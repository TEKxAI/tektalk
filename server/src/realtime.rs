use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
    Json,
};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{Duration, Instant};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use uuid::Uuid;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    error::{AppError, AppResult},
    state::{AppState, Ticket},
};
use tektalk_client_core::mtproto::{
    self, Direction, Message as MtMessage, MessageIdGenerator, AUTH_KEY_LEN,
};

const TCP_PREFACE: u8 = 0xef;
const MAX_TICKET_LEN: usize = 256;
const MAX_ENCRYPTED_FRAME: usize = 16 * 1024 * 1024;

#[derive(Clone)]
pub struct ClientHandle {
    pub device_id: Uuid,
    pub tx: mpsc::Sender<Vec<u8>>,
}

#[derive(Deserialize)]
pub struct BootstrapRequest {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct BootstrapResponse {
    pub ticket: String,
    pub server_public_key: String,
    pub expires_in_seconds: u32,
    pub protocol_version: u8,
    pub tcp_endpoint: String,
    pub wss_endpoint: String,
    pub transport_order: [&'static str; 2],
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub ticket: String,
}

#[derive(Deserialize, Serialize)]
struct SendMessage {
    conversation_id: Uuid,
    client_message_id: Uuid,
    recipient_id: Uuid,
    text: String,
}

#[derive(Serialize)]
struct Ack {
    client_message_id: Uuid,
    server_message_id: i64,
    server_timestamp: i64,
}

pub async fn bootstrap(
    State(state): State<AppState>,
    Json(request): Json<BootstrapRequest>,
) -> AppResult<Json<BootstrapResponse>> {
    let claims = crate::token::verify(&state.config.jwt_secret, &request.access_token)?;
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    let ticket = Uuid::new_v4().to_string();
    state.tickets.insert(
        ticket.clone(),
        Ticket {
            user_id: claims.sub,
            device_id: claims.device,
            expires_at: Instant::now() + Duration::from_secs(60),
            server_secret: secret,
        },
    );
    Ok(Json(BootstrapResponse {
        ticket,
        server_public_key: STANDARD_NO_PAD.encode(public.as_bytes()),
        expires_in_seconds: 60,
        protocol_version: 2,
        tcp_endpoint: state.config.realtime_tcp_public.clone(),
        wss_endpoint: state.config.realtime_wss_public.clone(),
        transport_order: ["tcp", "wss"],
    }))
}

pub async fn socket(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let ticket = take_ticket(&state, &query.ticket)?;
    Ok(ws.on_upgrade(move |socket| serve_websocket(state, ticket, socket)))
}

pub async fn serve_tcp(listener: TcpListener, state: AppState) {
    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(error) = serve_tcp_connection(state, stream).await {
                        tracing::debug!(%peer, %error, "raw TCP realtime connection closed");
                    }
                });
            }
            Err(error) => tracing::warn!(%error, "raw TCP accept failed"),
        }
    }
}

async fn serve_websocket(state: AppState, ticket: Ticket, mut socket: WebSocket) {
    let client_public = match socket.recv().await {
        Some(Ok(Message::Binary(bytes))) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            PublicKey::from(key)
        }
        _ => return,
    };
    let Some(auth_key) = derive_auth_key(&ticket, &client_public) else {
        return;
    };
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<Vec<u8>>(256);
    register_online(&state, &ticket, outbound_tx);
    let (mut writer, mut reader) = socket.split();
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = outbound_rx.recv().await {
            if writer.send(Message::Binary(frame)).await.is_err() {
                break;
            }
        }
    });

    let outbound_ids = MessageIdGenerator::new(Direction::ServerToClient);
    let mut last_message_id = 0i64;
    while let Some(Ok(Message::Binary(frame))) = reader.next().await {
        let Some(reply) = process_frame(
            &state,
            &ticket,
            &auth_key,
            &frame,
            &outbound_ids,
            &mut last_message_id,
        )
        .await
        else {
            break;
        };
        if let Some(reply) = reply {
            send_to_device(&state, &ticket, reply);
        }
    }
    unregister_online(&state, &ticket);
    writer_task.abort();
}

async fn serve_tcp_connection(state: AppState, mut stream: TcpStream) -> anyhow::Result<()> {
    let mut preface = [0u8; 1];
    stream.read_exact(&mut preface).await?;
    anyhow::ensure!(preface[0] == TCP_PREFACE, "invalid TCP transport preface");

    let ticket_len = stream.read_u16_le().await? as usize;
    anyhow::ensure!(
        (1..=MAX_TICKET_LEN).contains(&ticket_len),
        "invalid realtime ticket length"
    );
    let mut ticket_bytes = vec![0u8; ticket_len];
    stream.read_exact(&mut ticket_bytes).await?;
    let ticket_text = std::str::from_utf8(&ticket_bytes)?;
    let ticket = take_ticket(&state, ticket_text).map_err(|_| anyhow::anyhow!("invalid ticket"))?;

    let mut client_key = [0u8; 32];
    stream.read_exact(&mut client_key).await?;
    let auth_key = derive_auth_key(&ticket, &PublicKey::from(client_key))
        .ok_or_else(|| anyhow::anyhow!("auth key derivation failed"))?;

    let (mut reader, mut writer) = stream.into_split();
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<Vec<u8>>(256);
    register_online(&state, &ticket, outbound_tx);
    let writer_task = tokio::spawn(async move {
        while let Some(frame) = outbound_rx.recv().await {
            let Ok(encoded) = mtproto::abridged_encode(&frame) else {
                break;
            };
            if writer.write_all(&encoded).await.is_err() {
                break;
            }
        }
    });

    let outbound_ids = MessageIdGenerator::new(Direction::ServerToClient);
    let mut last_message_id = 0i64;
    loop {
        let frame = match read_abridged_frame(&mut reader).await {
            Ok(frame) => frame,
            Err(error) => {
                unregister_online(&state, &ticket);
                writer_task.abort();
                return Err(error);
            }
        };
        let Some(reply) = process_frame(
            &state,
            &ticket,
            &auth_key,
            &frame,
            &outbound_ids,
            &mut last_message_id,
        )
        .await
        else {
            break;
        };
        if let Some(reply) = reply {
            send_to_device(&state, &ticket, reply);
        }
    }
    unregister_online(&state, &ticket);
    writer_task.abort();
    Ok(())
}

async fn read_abridged_frame<R: AsyncRead + Unpin>(reader: &mut R) -> anyhow::Result<Vec<u8>> {
    let first = reader.read_u8().await?;
    let words = if first < 0x7f {
        usize::from(first)
    } else {
        let mut length = [0u8; 4];
        reader.read_exact(&mut length[..3]).await?;
        u32::from_le_bytes(length) as usize
    };
    anyhow::ensure!(words > 0, "empty abridged frame");
    let bytes = words
        .checked_mul(4)
        .ok_or_else(|| anyhow::anyhow!("abridged frame overflow"))?;
    anyhow::ensure!(bytes <= MAX_ENCRYPTED_FRAME, "abridged frame too large");
    let mut frame = vec![0u8; bytes];
    reader.read_exact(&mut frame).await?;
    Ok(frame)
}

async fn process_frame(
    state: &AppState,
    ticket: &Ticket,
    auth_key: &[u8; AUTH_KEY_LEN],
    frame: &[u8],
    outbound_ids: &MessageIdGenerator,
    last_message_id: &mut i64,
) -> Option<Option<Vec<u8>>> {
    let incoming = mtproto::decrypt(auth_key, frame, Direction::ClientToServer).ok()?;
    if incoming.message_id <= *last_message_id {
        return None;
    }
    *last_message_id = incoming.message_id;

    let command = match serde_json::from_slice::<SendMessage>(&incoming.body) {
        Ok(command) => command,
        Err(_) => return Some(None),
    };
    if command.text.len() > 16_384 {
        return Some(None);
    }
    let server_id = match state.message_ids.next_id() {
        Ok(id) => id,
        Err(_) => return Some(None),
    };
    if sqlx::query(
        "INSERT INTO messages(id,conversation_id,sender_id,recipient_id,client_message_id,body,created_at) \
         VALUES($1,$2,$3,$4,$5,$6,now()) \
         ON CONFLICT(sender_id,client_message_id) DO NOTHING",
    )
    .bind(server_id)
    .bind(command.conversation_id)
    .bind(ticket.user_id)
    .bind(command.recipient_id)
    .bind(command.client_message_id)
    .bind(&command.text)
    .execute(&state.db)
    .await
    .is_err()
    {
        return Some(None);
    }

    let body = serde_json::to_vec(&Ack {
        client_message_id: command.client_message_id,
        server_message_id: server_id,
        server_timestamp: Utc::now().timestamp_millis(),
    })
    .ok()?;
    let message_id = outbound_ids.next_id().ok()?;
    let outgoing = MtMessage {
        server_salt: incoming.server_salt,
        session_id: incoming.session_id,
        message_id,
        sequence_no: 1,
        body,
    };
    Some(mtproto::encrypt(auth_key, &outgoing, Direction::ServerToClient).ok())
}

fn take_ticket(state: &AppState, value: &str) -> AppResult<Ticket> {
    let ticket = state
        .tickets
        .remove(value)
        .map(|(_, ticket)| ticket)
        .ok_or(AppError::Unauthorized)?;
    if ticket.expires_at < Instant::now() {
        return Err(AppError::Unauthorized);
    }
    Ok(ticket)
}

fn derive_auth_key(
    ticket: &Ticket,
    client_public: &PublicKey,
) -> Option<[u8; AUTH_KEY_LEN]> {
    let shared = ticket.server_secret.diffie_hellman(client_public);
    let hkdf = Hkdf::<Sha256>::new(
        Some(b"tektalk-mtproto-bootstrap-v1"),
        shared.as_bytes(),
    );
    let mut auth_key = [0u8; AUTH_KEY_LEN];
    hkdf.expand(b"mtproto-auth-key", &mut auth_key).ok()?;
    Some(auth_key)
}

fn register_online(state: &AppState, ticket: &Ticket, tx: mpsc::Sender<Vec<u8>>) {
    state
        .online
        .entry(ticket.user_id)
        .or_default()
        .push(ClientHandle {
            device_id: ticket.device_id,
            tx,
        });
}

fn send_to_device(state: &AppState, ticket: &Ticket, frame: Vec<u8>) {
    if let Some(sender) = state.online.get(&ticket.user_id).and_then(|handles| {
        handles
            .iter()
            .find(|handle| handle.device_id == ticket.device_id)
            .map(|handle| handle.tx.clone())
    }) {
        let _ = sender.try_send(frame);
    }
}

fn unregister_online(state: &AppState, ticket: &Ticket) {
    if let Some(mut handles) = state.online.get_mut(&ticket.user_id) {
        handles.retain(|handle| handle.device_id != ticket.device_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn reads_short_abridged_frame() {
        let (mut client, mut server) = tokio::io::duplex(64);
        client.write_all(&[2, 1, 2, 3, 4, 5, 6, 7, 8]).await.unwrap();
        let frame = read_abridged_frame(&mut server).await.unwrap();
        assert_eq!(frame, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[tokio::test]
    async fn rejects_empty_abridged_frame() {
        let (mut client, mut server) = tokio::io::duplex(8);
        client.write_all(&[0]).await.unwrap();
        assert!(read_abridged_frame(&mut server).await.is_err());
    }
}
