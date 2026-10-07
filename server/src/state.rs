use dashmap::DashMap;
use sqlx::PgPool;
use std::{sync::Arc, time::Instant};
use tonic::transport::Channel;
use uuid::Uuid;
use tektalk_client_core::id::SnowflakeGenerator;

use crate::{config::Config, realtime::ClientHandle};
use x25519_dalek::StaticSecret;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub redis: redis::Client,
    pub business_channel: Channel,
    pub online: Arc<DashMap<Uuid, Vec<ClientHandle>>>,
    pub tickets: Arc<DashMap<String, Ticket>>,
    pub message_ids: Arc<SnowflakeGenerator>,
}

#[derive(Clone)]
pub struct Ticket { pub user_id: Uuid, pub device_id: Uuid, pub expires_at: Instant, pub server_secret: StaticSecret }

