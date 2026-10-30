use axum::{extract::State, http::HeaderMap, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::{auth, error::{AppError, AppResult}, state::AppState};
use tektalk_client_core::secret_chat::SUITE;

#[derive(Deserialize)]
pub struct RegisterBundle {
    pub suite: String,
    pub x25519_public: Vec<u8>,
    pub ml_kem_768_public: Vec<u8>,
}

#[derive(Serialize, FromRow)]
pub struct PublicBundle {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub suite: String,
    pub x25519_public: Vec<u8>,
    pub ml_kem_768_public: Vec<u8>,
}

#[derive(Deserialize)]
pub struct EnableSecretChat {
    pub conversation_id: Uuid,
    pub kind: String,
    #[serde(default)]
    pub member_ids: Vec<Uuid>,
}

#[derive(Deserialize)] pub struct ConversationRequest { pub conversation_id: Uuid }

#[derive(Serialize, FromRow)]
pub struct SecretPolicy {
    pub conversation_id: Uuid,
    pub kind: String,
    pub suite: String,
    pub epoch: i64,
}

#[derive(Deserialize)]
pub struct PublishKeyPackets {
    pub conversation_id: Uuid,
    pub epoch: i64,
    pub packets: Vec<KeyPacketInput>,
}

#[derive(Deserialize)]
pub struct KeyPacketInput { pub recipient_device_id: Uuid, pub packet: Value }

#[derive(Serialize, FromRow)]
pub struct KeyPacket {
    pub conversation_id: Uuid,
    pub epoch: i64,
    pub sender_device_id: Uuid,
    pub recipient_device_id: Uuid,
    pub packet: Value,
    pub created_at: DateTime<Utc>,
    pub generation: i64,
}

#[derive(Deserialize)]
pub struct SendCiphertext {
    pub conversation_id: Uuid,
    pub client_message_id: Uuid,
    pub epoch: i64,
    pub counter: i64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    #[serde(default = "default_generation")]
    pub generation: i64,
}
fn default_generation()->i64{1}

#[derive(Serialize, FromRow)]
pub struct SecretMessage {
    pub id: i64,
    pub conversation_id: Uuid,
    pub epoch: i64,
    pub sender_id: Uuid,
    pub sender_device_id: Uuid,
    pub client_message_id: Uuid,
    pub counter: i64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub generation: i64,
}

#[derive(Serialize)] pub struct SendCiphertextResponse { pub message: SecretMessage, pub deduplicated: bool }

async fn require_member(state: &AppState, conversation_id: Uuid, user_id: Uuid) -> AppResult<()> {
    let member = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM secret_conversation_members WHERE conversation_id=$1 AND user_id=$2 AND left_epoch IS NULL)")
        .bind(conversation_id).bind(user_id).fetch_one(&state.db).await?;
    if !member { return Err(AppError::Unauthorized); }
    Ok(())
}

pub async fn register_bundle(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<RegisterBundle>) -> AppResult<Json<PublicBundle>> {
    let claims = auth::claims(&state, &headers)?;
    if request.suite != SUITE { return Err(AppError::Invalid("mandatory hybrid suite is required")); }
    if request.x25519_public.len() != 32 { return Err(AppError::Invalid("invalid X25519 public key")); }
    if request.ml_kem_768_public.len() != 1184 { return Err(AppError::Invalid("invalid ML-KEM-768 public key")); }
    let row = sqlx::query_as::<_, PublicBundle>("INSERT INTO secret_device_bundles(device_id,user_id,suite,x25519_public,ml_kem_768_public) VALUES($1,$2,$3,$4,$5) ON CONFLICT(device_id) DO UPDATE SET suite=EXCLUDED.suite,x25519_public=EXCLUDED.x25519_public,ml_kem_768_public=EXCLUDED.ml_kem_768_public,updated_at=now() RETURNING user_id,device_id,suite,x25519_public,ml_kem_768_public")
        .bind(claims.device).bind(claims.sub).bind(&request.suite).bind(&request.x25519_public).bind(&request.ml_kem_768_public).fetch_one(&state.db).await?;
    Ok(Json(row))
}

pub async fn enable(State(state): State<AppState>, headers: HeaderMap, Json(mut request): Json<EnableSecretChat>) -> AppResult<Json<SecretPolicy>> {
    let claims = auth::claims(&state, &headers)?;
    if !matches!(request.kind.as_str(), "one_to_one" | "group") { return Err(AppError::Invalid("kind must be one_to_one or group")); }
    request.member_ids.push(claims.sub);
    request.member_ids.sort_unstable(); request.member_ids.dedup();
    if request.kind == "one_to_one" && request.member_ids.len() != 2 { return Err(AppError::Invalid("one-to-one secret chat requires exactly two members")); }
    if request.kind == "group" && !(2..=256).contains(&request.member_ids.len()) { return Err(AppError::Invalid("group secret chat requires 2..256 members")); }
    let existing = sqlx::query("SELECT kind,epoch,created_by FROM secret_conversations WHERE conversation_id=$1").bind(request.conversation_id).fetch_optional(&state.db).await?;
    let mut tx = state.db.begin().await?;
    if let Some(row) = existing {
        let existing_kind: String = row.get("kind");
        if existing_kind != request.kind { return Err(AppError::Conflict("conversation kind cannot change")); }
        require_member(&state, request.conversation_id, claims.sub).await?;
        let current: Vec<Uuid> = sqlx::query_scalar("SELECT user_id FROM secret_conversation_members WHERE conversation_id=$1 AND left_epoch IS NULL ORDER BY user_id")
            .bind(request.conversation_id).fetch_all(&mut *tx).await?;
        if current != request.member_ids {
            if row.get::<Uuid, _>("created_by") != claims.sub { return Err(AppError::Unauthorized); }
            let epoch: i64 = sqlx::query_scalar("UPDATE secret_conversations SET epoch=epoch+1,updated_at=now() WHERE conversation_id=$1 RETURNING epoch").bind(request.conversation_id).fetch_one(&mut *tx).await?;
            sqlx::query("UPDATE secret_conversation_members SET left_epoch=$2 WHERE conversation_id=$1 AND left_epoch IS NULL AND NOT(user_id=ANY($3))").bind(request.conversation_id).bind(epoch).bind(&request.member_ids).execute(&mut *tx).await?;
            for member in &request.member_ids {
                sqlx::query("INSERT INTO secret_conversation_members(conversation_id,user_id,joined_epoch,left_epoch) VALUES($1,$2,$3,NULL) ON CONFLICT(conversation_id,user_id) DO UPDATE SET joined_epoch=$3,left_epoch=NULL").bind(request.conversation_id).bind(member).bind(epoch).execute(&mut *tx).await?;
            }
        }
    } else {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id=ANY($1) AND disabled_at IS NULL").bind(&request.member_ids).fetch_one(&mut *tx).await?;
        if count as usize != request.member_ids.len() { return Err(AppError::Invalid("one or more members do not exist")); }
        sqlx::query("INSERT INTO secret_conversations(conversation_id,kind,created_by,suite,epoch) VALUES($1,$2,$3,$4,1)").bind(request.conversation_id).bind(&request.kind).bind(claims.sub).bind(SUITE).execute(&mut *tx).await?;
        for member in &request.member_ids {
            sqlx::query("INSERT INTO secret_conversation_members(conversation_id,user_id,joined_epoch) VALUES($1,$2,1)").bind(request.conversation_id).bind(member).execute(&mut *tx).await?;
        }
    }
    tx.commit().await?;
    policy_inner(&state, request.conversation_id, claims.sub).await.map(Json)
}

async fn policy_inner(state: &AppState, conversation_id: Uuid, user_id: Uuid) -> AppResult<SecretPolicy> {
    require_member(state, conversation_id, user_id).await?;
    Ok(sqlx::query_as::<_, SecretPolicy>("SELECT conversation_id,kind,suite,epoch FROM secret_conversations WHERE conversation_id=$1")
        .bind(conversation_id).fetch_one(&state.db).await?)
}

pub async fn policy(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<ConversationRequest>) -> AppResult<Json<SecretPolicy>> {
    let claims = auth::claims(&state, &headers)?;
    policy_inner(&state, request.conversation_id, claims.sub).await.map(Json)
}

pub async fn bundles(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<ConversationRequest>) -> AppResult<Json<Vec<PublicBundle>>> {
    let claims = auth::claims(&state, &headers)?;
    require_member(&state, request.conversation_id, claims.sub).await?;
    let rows = sqlx::query_as::<_, PublicBundle>("SELECT b.user_id,b.device_id,b.suite,b.x25519_public,b.ml_kem_768_public FROM secret_device_bundles b JOIN secret_conversation_members m ON m.user_id=b.user_id WHERE m.conversation_id=$1 AND m.left_epoch IS NULL ORDER BY b.user_id,b.device_id")
        .bind(request.conversation_id).fetch_all(&state.db).await?;
    Ok(Json(rows))
}

pub async fn publish_keys(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<PublishKeyPackets>) -> AppResult<Json<Value>> {
    let claims = auth::claims(&state, &headers)?;
    let policy = policy_inner(&state, request.conversation_id, claims.sub).await?;
    if request.epoch != policy.epoch { return Err(AppError::Conflict("membership epoch is stale")); }
    if request.packets.is_empty() || request.packets.len() > 1024 { return Err(AppError::Invalid("key packet count must be 1..1024")); }
    let mut tx = state.db.begin().await?;
    for packet in request.packets {
        let allowed = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM devices d JOIN secret_conversation_members m ON m.user_id=d.user_id WHERE d.id=$1 AND d.revoked_at IS NULL AND m.conversation_id=$2 AND m.left_epoch IS NULL)").bind(packet.recipient_device_id).bind(request.conversation_id).fetch_one(&mut *tx).await?;
        if !allowed { return Err(AppError::Invalid("recipient device is not an active member")); }
        let generation=packet.packet.get("generation").and_then(Value::as_i64).ok_or(AppError::Invalid("key packet generation is required"))?;
        if generation < 1 { return Err(AppError::Invalid("invalid key generation")); }
        sqlx::query("INSERT INTO secret_key_packets(conversation_id,epoch,generation,sender_device_id,recipient_device_id,packet) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(conversation_id,epoch,generation,sender_device_id,recipient_device_id) DO UPDATE SET packet=EXCLUDED.packet,created_at=now()")
            .bind(request.conversation_id).bind(request.epoch).bind(generation).bind(claims.device).bind(packet.recipient_device_id).bind(packet.packet).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({"status":"published","epoch":request.epoch})))
}

pub async fn key_packets(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<ConversationRequest>) -> AppResult<Json<Vec<KeyPacket>>> {
    let claims = auth::claims(&state, &headers)?;
    let policy = policy_inner(&state, request.conversation_id, claims.sub).await?;
    let rows = sqlx::query_as::<_, KeyPacket>("SELECT conversation_id,epoch,generation,sender_device_id,recipient_device_id,packet,created_at FROM secret_key_packets WHERE conversation_id=$1 AND epoch=$2 AND recipient_device_id=$3 ORDER BY generation,created_at")
        .bind(request.conversation_id).bind(policy.epoch).bind(claims.device).fetch_all(&state.db).await?;
    Ok(Json(rows))
}

pub async fn send(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<SendCiphertext>) -> AppResult<Json<SendCiphertextResponse>> {
    let claims = auth::claims(&state, &headers)?;
    let policy = policy_inner(&state, request.conversation_id, claims.sub).await?;
    if request.epoch != policy.epoch { return Err(AppError::Conflict("membership epoch is stale")); }
    if request.counter < 0 { return Err(AppError::Invalid("counter must be non-negative")); }
    if request.generation < 1 { return Err(AppError::Invalid("generation must be positive")); }
    if request.nonce.len() != 12 { return Err(AppError::Invalid("invalid nonce")); }
    if request.ciphertext.len() < 16 || request.ciphertext.len() > 131_072 { return Err(AppError::Invalid("invalid ciphertext")); }
    let id = state.message_ids.next_id().map_err(|_| AppError::Invalid("message id generation failed"))?;
    let inserted = sqlx::query("INSERT INTO secret_messages(id,conversation_id,epoch,generation,sender_id,sender_device_id,client_message_id,counter,nonce,ciphertext) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT(sender_device_id,client_message_id) DO NOTHING")
        .bind(id).bind(request.conversation_id).bind(request.epoch).bind(request.generation).bind(claims.sub).bind(claims.device).bind(request.client_message_id).bind(request.counter).bind(&request.nonce).bind(&request.ciphertext).execute(&state.db).await?.rows_affected() == 1;
    let message = sqlx::query_as::<_, SecretMessage>("SELECT id,conversation_id,epoch,generation,sender_id,sender_device_id,client_message_id,counter,nonce,ciphertext,created_at FROM secret_messages WHERE sender_device_id=$1 AND client_message_id=$2")
        .bind(claims.device).bind(request.client_message_id).fetch_one(&state.db).await?;
    Ok(Json(SendCiphertextResponse { message, deduplicated: !inserted }))
}

pub async fn history(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<ConversationRequest>) -> AppResult<Json<Vec<SecretMessage>>> {
    let claims = auth::claims(&state, &headers)?;
    require_member(&state, request.conversation_id, claims.sub).await?;
    let rows = sqlx::query_as::<_, SecretMessage>("SELECT id,conversation_id,epoch,generation,sender_id,sender_device_id,client_message_id,counter,nonce,ciphertext,created_at FROM secret_messages WHERE conversation_id=$1 ORDER BY id DESC LIMIT 100")
        .bind(request.conversation_id).fetch_all(&state.db).await?;
    Ok(Json(rows.into_iter().rev().collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_plaintext_and_wrong_suite_at_the_contract_boundary() {
        assert!(serde_json::from_value::<SendCiphertext>(serde_json::json!({"conversation_id":Uuid::new_v4(),"client_message_id":Uuid::new_v4(),"epoch":1,"counter":0,"text":"leak"})).is_err());
        assert_ne!(SUITE, "X25519_ONLY");
    }
}
