use axum::{extract::State, http::HeaderMap, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)] pub struct SendMessage { pub conversation_id: Uuid, pub recipient_id: Uuid, pub client_message_id: Uuid, pub text: String }
#[derive(Deserialize)] pub struct MessageHistory { pub conversation_id: Uuid, pub before_message_id: Option<i64>, pub limit: Option<i64> }
#[derive(Serialize, FromRow)] pub struct ChatMessage { pub id:i64, pub conversation_id:Uuid, pub sender_id:Uuid, pub recipient_id:Uuid, pub client_message_id:Uuid, pub body:String, pub created_at:DateTime<Utc> }
#[derive(Serialize)] pub struct SendMessageResponse { pub message: ChatMessage, pub deduplicated: bool }

pub async fn send(State(state):State<AppState>,headers:HeaderMap,Json(request):Json<SendMessage>)->AppResult<Json<SendMessageResponse>>{
    let claims=auth::claims(&state,&headers)?;let text=request.text.trim();
    if text.is_empty()||text.len()>16_384{return Err(AppError::Invalid("message must contain 1..16384 characters"));}
    if request.recipient_id==claims.sub{return Err(AppError::Invalid("recipient must be another account"));}
    let exists=sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND disabled_at IS NULL)").bind(request.recipient_id).fetch_one(&state.db).await?;
    if !exists{return Err(AppError::Invalid("recipient does not exist"));}
    let id=crate::realtime::snowflake();
    let inserted=sqlx::query("INSERT INTO messages(id,conversation_id,sender_id,recipient_id,client_message_id,body,created_at) VALUES($1,$2,$3,$4,$5,$6,now()) ON CONFLICT(sender_id,client_message_id) DO NOTHING").bind(id).bind(request.conversation_id).bind(claims.sub).bind(request.recipient_id).bind(request.client_message_id).bind(text).execute(&state.db).await?.rows_affected()==1;
    let message=sqlx::query_as::<_,ChatMessage>("SELECT id,conversation_id,sender_id,recipient_id,client_message_id,body,created_at FROM messages WHERE sender_id=$1 AND client_message_id=$2").bind(claims.sub).bind(request.client_message_id).fetch_one(&state.db).await?;
    Ok(Json(SendMessageResponse{message,deduplicated:!inserted}))
}

pub async fn history(State(state):State<AppState>,headers:HeaderMap,Json(request):Json<MessageHistory>)->AppResult<Json<Vec<ChatMessage>>>{
    let claims=auth::claims(&state,&headers)?;let limit=request.limit.unwrap_or(50).clamp(1,100);let before=request.before_message_id.unwrap_or(i64::MAX);
    let messages=sqlx::query_as::<_,ChatMessage>("SELECT id,conversation_id,sender_id,recipient_id,client_message_id,body,created_at FROM messages WHERE conversation_id=$1 AND id<$2 AND (sender_id=$3 OR recipient_id=$3) ORDER BY id DESC LIMIT $4").bind(request.conversation_id).bind(before).bind(claims.sub).bind(limit).fetch_all(&state.db).await?;
    Ok(Json(messages.into_iter().rev().collect()))
}
