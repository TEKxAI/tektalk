use axum::{extract::{ws::{Message,WebSocket,WebSocketUpgrade},Query,State},response::Response,Json};
use base64::{engine::general_purpose::STANDARD_NO_PAD,Engine};
use chrono::Utc;
use futures_util::{SinkExt,StreamExt};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use serde::{Deserialize,Serialize};
use sha2::Sha256;
use std::time::{Duration,Instant};
use tokio::sync::mpsc;
use uuid::Uuid;
use x25519_dalek::{PublicKey,StaticSecret};
use crate::{error::{AppError,AppResult},mtproto::{self,Direction,Message as MtMessage,AUTH_KEY_LEN},state::{AppState,Ticket}};

#[derive(Clone)] pub struct ClientHandle{pub device_id:Uuid,pub tx:mpsc::Sender<Vec<u8>>}
#[derive(Deserialize)] pub struct BootstrapRequest{pub access_token:String}
#[derive(Serialize)] pub struct BootstrapResponse{pub ticket:String,pub server_public_key:String,pub expires_in_seconds:u32,pub protocol_version:u8}
#[derive(Deserialize)] pub struct WsQuery{pub ticket:String}
#[derive(Deserialize,Serialize)] struct SendMessage{conversation_id:Uuid,client_message_id:Uuid,recipient_id:Uuid,text:String}
#[derive(Serialize)] struct Ack{client_message_id:Uuid,server_message_id:i64,server_timestamp:i64}

pub async fn bootstrap(State(s):State<AppState>,Json(r):Json<BootstrapRequest>)->AppResult<Json<BootstrapResponse>>{let c=crate::token::verify(&s.config.jwt_secret,&r.access_token)?;let secret=StaticSecret::random_from_rng(OsRng);let public=PublicKey::from(&secret);let ticket=Uuid::new_v4().to_string();s.tickets.insert(ticket.clone(),Ticket{user_id:c.sub,device_id:c.device,expires_at:Instant::now()+Duration::from_secs(60),server_secret:secret});Ok(Json(BootstrapResponse{ticket,server_public_key:STANDARD_NO_PAD.encode(public.as_bytes()),expires_in_seconds:60,protocol_version:1}))}

pub async fn socket(State(s):State<AppState>,Query(q):Query<WsQuery>,ws:WebSocketUpgrade)->AppResult<Response>{let ticket=s.tickets.remove(&q.ticket).map(|(_,v)|v).ok_or(AppError::Unauthorized)?;if ticket.expires_at<Instant::now(){return Err(AppError::Unauthorized)}Ok(ws.on_upgrade(move|sock|serve(s,ticket,sock)))}

async fn serve(s:AppState,ticket:Ticket,mut socket:WebSocket){let client_key=match socket.recv().await{Some(Ok(Message::Binary(b)))if b.len()==32=>{let mut a=[0u8;32];a.copy_from_slice(&b);PublicKey::from(a)},_=>return};let shared=ticket.server_secret.diffie_hellman(&client_key);let hk=Hkdf::<Sha256>::new(Some(b"tektalk-mtproto-bootstrap-v1"),shared.as_bytes());let mut auth_key=[0u8;AUTH_KEY_LEN];if hk.expand(b"mtproto-auth-key",&mut auth_key).is_err(){return}let(tx,mut rx)=mpsc::channel::<Vec<u8>>(256);let handle=ClientHandle{device_id:ticket.device_id,tx};s.online.entry(ticket.user_id).or_default().push(handle);let(user_tx,mut user_rx)=socket.split();let writer=tokio::spawn(async move{let mut out=user_tx;while let Some(frame)=rx.recv().await{if out.send(Message::Binary(frame)).await.is_err(){break}}});let mut last_message_id=0i64;while let Some(Ok(Message::Binary(frame)))=user_rx.next().await{let Ok(incoming)=mtproto::decrypt(&auth_key,&frame,Direction::ClientToServer)else{break};if incoming.message_id<=last_message_id{break}last_message_id=incoming.message_id;let Ok(cmd)=serde_json::from_slice::<SendMessage>(&incoming.body)else{continue};if cmd.text.len()>16_384{continue}let server_id=snowflake();let inserted=sqlx::query("INSERT INTO messages(id,conversation_id,sender_id,recipient_id,client_message_id,body,created_at) VALUES($1,$2,$3,$4,$5,$6,now()) ON CONFLICT(sender_id,client_message_id) DO NOTHING").bind(server_id).bind(cmd.conversation_id).bind(ticket.user_id).bind(cmd.recipient_id).bind(cmd.client_message_id).bind(&cmd.text).execute(&s.db).await;if inserted.is_err(){continue}let ack=serde_json::to_vec(&Ack{client_message_id:cmd.client_message_id,server_message_id:server_id,server_timestamp:Utc::now().timestamp_millis()}).unwrap();let outbound=MtMessage{server_salt:incoming.server_salt,session_id:incoming.session_id,message_id:server_mtproto_message_id(),sequence_no:1,body:ack};if let Ok(f)=mtproto::encrypt(&auth_key,&outbound,Direction::ServerToClient){let _=s.online.get(&ticket.user_id).and_then(|v|v.iter().find(|x|x.device_id==ticket.device_id).map(|x|x.tx.clone())).map(|t|t.try_send(f));}}if let Some(mut list)=s.online.get_mut(&ticket.user_id){list.retain(|h|h.device_id!=ticket.device_id);}writer.abort();}
pub(crate) fn snowflake()->i64{let epoch=1_704_067_200_000i64;let ms=Utc::now().timestamp_millis()-epoch;(ms<<22)|(rand::random::<u32>()as i64&0x3fffff)}
fn server_mtproto_message_id()->i64{let now=Utc::now();let raw=(now.timestamp()<<32)|(((now.timestamp_subsec_nanos()as i64)<<2)/1_000_000_000);(raw&!3)|1}
