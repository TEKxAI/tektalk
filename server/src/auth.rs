use axum::{extract::State, http::HeaderMap, Json};
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{crypto, error::{AppError, AppResult}, state::AppState, token};

#[derive(Deserialize)] pub struct Register { pub phone: String, pub display_name: String, pub password: String, pub security_question: String, pub security_answer: String, pub device_name: String }
#[derive(Deserialize)] pub struct Login { pub phone: String, pub password: String, pub device_id: Option<Uuid>, pub device_name: String }
#[derive(Deserialize)] pub struct DeviceChallenge { pub challenge_id: Uuid, pub answer: String }
#[derive(Deserialize)] pub struct Phone { pub phone: String }
#[derive(Deserialize)] pub struct ResetPassword { pub phone: String, pub otp: String, pub new_password: String }
#[derive(Deserialize)] pub struct ChangePassword { pub current_password: String, pub new_password: String }
#[derive(Serialize)] pub struct AuthTokens { pub access_token: String, pub refresh_token: String, pub user_id: Uuid, pub device_id: Uuid }
#[derive(Serialize)] #[serde(tag = "status", rename_all = "snake_case")] pub enum LoginResult { Authenticated { tokens: AuthTokens }, DeviceVerificationRequired { challenge_id: Uuid, question: String } }

pub async fn register(State(s): State<AppState>, Json(r): Json<Register>) -> AppResult<Json<AuthTokens>> {
    let phone = normalize_phone(&r.phone)?;
    let pass = crypto::hash_password(&r.password)?;
    if r.security_answer.trim().len() < 3 { return Err(AppError::Invalid("security answer is too short")); }
    let answer = crypto::hash_secret(&crypto::normalize_answer(&r.security_answer))?;
    let mut tx = s.db.begin().await?;
    let user_id = Uuid::new_v4(); let device_id = Uuid::new_v4();
    let inserted = sqlx::query("INSERT INTO users(id, phone_e164, display_name, password_hash, security_question, security_answer_hash) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(user_id).bind(&phone).bind(r.display_name.trim()).bind(pass).bind(r.security_question.trim()).bind(answer).execute(&mut *tx).await;
    if let Err(sqlx::Error::Database(e)) = &inserted { if e.is_unique_violation() { return Err(AppError::Conflict("phone already registered")); } }
    inserted?;
    sqlx::query("INSERT INTO devices(id,user_id,name,trusted_at,last_seen_at) VALUES($1,$2,$3,now(),now())").bind(device_id).bind(user_id).bind(r.device_name).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(make_tokens(&s, user_id, device_id).await?))
}

pub async fn login(State(s): State<AppState>, Json(r): Json<Login>) -> AppResult<Json<LoginResult>> {
    let phone = normalize_phone(&r.phone)?;
    let row = sqlx::query("SELECT id,password_hash,security_question FROM users WHERE phone_e164=$1 AND disabled_at IS NULL").bind(phone).fetch_optional(&s.db).await?.ok_or(AppError::Unauthorized)?;
    let user_id: Uuid = row.get("id");
    if !crypto::verify_password(row.get("password_hash"), &r.password) { return Err(AppError::Unauthorized); }
    if let Some(device_id) = r.device_id {
        let trusted = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM devices WHERE id=$1 AND user_id=$2 AND revoked_at IS NULL)").bind(device_id).bind(user_id).fetch_one(&s.db).await?;
        if trusted { return Ok(Json(LoginResult::Authenticated { tokens: make_tokens(&s, user_id, device_id).await? })); }
    }
    let challenge_id = Uuid::new_v4();
    sqlx::query("INSERT INTO device_challenges(id,user_id,device_name,expires_at) VALUES($1,$2,$3,$4)").bind(challenge_id).bind(user_id).bind(r.device_name).bind(Utc::now()+Duration::minutes(5)).execute(&s.db).await?;
    Ok(Json(LoginResult::DeviceVerificationRequired { challenge_id, question: row.get("security_question") }))
}

pub async fn verify_device(State(s): State<AppState>, Json(r): Json<DeviceChallenge>) -> AppResult<Json<AuthTokens>> {
    let row = sqlx::query("SELECT c.user_id,c.device_name,u.security_answer_hash FROM device_challenges c JOIN users u ON u.id=c.user_id WHERE c.id=$1 AND c.used_at IS NULL AND c.expires_at>now() FOR UPDATE").bind(r.challenge_id).fetch_optional(&s.db).await?.ok_or(AppError::Unauthorized)?;
    if !crypto::verify_password(row.get("security_answer_hash"), &crypto::normalize_answer(&r.answer)) { return Err(AppError::Unauthorized); }
    let user_id: Uuid=row.get("user_id"); let device_id=Uuid::new_v4(); let mut tx=s.db.begin().await?;
    let claimed=sqlx::query("UPDATE device_challenges SET used_at=now() WHERE id=$1 AND used_at IS NULL AND expires_at>now()").bind(r.challenge_id).execute(&mut *tx).await?.rows_affected();
    if claimed!=1{return Err(AppError::Unauthorized)}
    sqlx::query("INSERT INTO devices(id,user_id,name,trusted_at,last_seen_at) VALUES($1,$2,$3,now(),now())").bind(device_id).bind(user_id).bind(row.get::<String,_>("device_name")).execute(&mut *tx).await?;
    tx.commit().await?; Ok(Json(make_tokens(&s,user_id,device_id).await?))
}

pub async fn request_reset(State(s): State<AppState>, Json(r): Json<Phone>) -> AppResult<Json<serde_json::Value>> {
    let phone=normalize_phone(&r.phone)?; let otp=format!("{:06}",rand::thread_rng().gen_range(0..1_000_000));
    let digest=crypto::hmac_hex(s.config.otp_hmac_secret.as_bytes(),&format!("{phone}:{otp}"));
    sqlx::query("INSERT INTO password_reset_otps(phone_e164,otp_digest,expires_at) VALUES($1,$2,now()+interval '5 minutes')").bind(&phone).bind(digest).execute(&s.db).await?;
    tracing::info!(phone_suffix=%phone.chars().rev().take(4).collect::<String>(), "OTP queued for provider");
    let mut response=serde_json::json!({"status":"accepted"});
    if s.config.otp_dev_echo { response["development_otp"]=serde_json::Value::String(otp); }
    Ok(Json(response))
}

pub async fn reset_password(State(s): State<AppState>, Json(r): Json<ResetPassword>) -> AppResult<Json<serde_json::Value>> {
    let phone=normalize_phone(&r.phone)?; let digest=crypto::hmac_hex(s.config.otp_hmac_secret.as_bytes(),&format!("{phone}:{}",r.otp)); let pass=crypto::hash_password(&r.new_password)?;
    let mut tx=s.db.begin().await?;
    let ok=sqlx::query("UPDATE password_reset_otps SET used_at=now() WHERE id=(SELECT id FROM password_reset_otps WHERE phone_e164=$1 AND otp_digest=$2 AND used_at IS NULL AND expires_at>now() AND attempts<5 ORDER BY created_at DESC LIMIT 1) AND used_at IS NULL").bind(&phone).bind(digest).execute(&mut *tx).await?.rows_affected()==1;
    if !ok { return Err(AppError::Unauthorized); }
    sqlx::query("UPDATE users SET password_hash=$1,password_changed_at=now() WHERE phone_e164=$2").bind(pass).bind(&phone).execute(&mut *tx).await?;
    sqlx::query("UPDATE refresh_tokens SET revoked_at=now() WHERE user_id=(SELECT id FROM users WHERE phone_e164=$1)").bind(&phone).execute(&mut *tx).await?;
    tx.commit().await?; Ok(Json(serde_json::json!({"status":"changed"})))
}

pub async fn change_password(State(s): State<AppState>, headers: HeaderMap, Json(r): Json<ChangePassword>) -> AppResult<Json<serde_json::Value>> {
    let claims=claims(&s,&headers)?; let hash=sqlx::query_scalar::<_,String>("SELECT password_hash FROM users WHERE id=$1").bind(claims.sub).fetch_one(&s.db).await?;
    if !crypto::verify_password(&hash,&r.current_password) { return Err(AppError::Unauthorized); }
    let pass=crypto::hash_password(&r.new_password)?;
    sqlx::query("UPDATE users SET password_hash=$1,password_changed_at=now() WHERE id=$2").bind(pass).bind(claims.sub).execute(&s.db).await?;
    Ok(Json(serde_json::json!({"status":"changed"})))
}

pub fn claims(s:&AppState, headers:&HeaderMap)->AppResult<token::Claims>{ let h=headers.get("authorization").and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).ok_or(AppError::Unauthorized)?; token::verify(&s.config.jwt_secret,h) }

async fn make_tokens(s:&AppState,user_id:Uuid,device_id:Uuid)->AppResult<AuthTokens>{ let access_token=token::issue(&s.config.jwt_secret,user_id,device_id)?; let refresh_token=Uuid::new_v4().to_string()+&Uuid::new_v4().to_string(); let digest=crypto::hmac_hex(s.config.jwt_secret.as_bytes(),&refresh_token); sqlx::query("INSERT INTO refresh_tokens(user_id,device_id,token_digest,expires_at) VALUES($1,$2,$3,now()+interval '30 days')").bind(user_id).bind(device_id).bind(digest).execute(&s.db).await?; Ok(AuthTokens{access_token,refresh_token,user_id,device_id}) }

fn normalize_phone(v:&str)->AppResult<String>{ let clean:String=v.chars().filter(|c|c.is_ascii_digit()||*c=='+').collect(); if !clean.starts_with('+')||clean.len()<9||clean.len()>16{return Err(AppError::Invalid("phone must be E.164"));} Ok(clean) }
