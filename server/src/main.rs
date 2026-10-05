mod auth; mod chat_http; mod config; mod crypto; mod error; mod realtime; mod state; mod token;
use axum::{routing::{get,post},Router};
use dashmap::DashMap;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::{cors::CorsLayer,trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use crate::{config::Config,state::AppState};
use tektalk_client_core::id::SnowflakeGenerator;

#[tokio::main]
async fn main()->anyhow::Result<()>{tracing_subscriber::fmt().json().with_env_filter(EnvFilter::from_default_env()).init();let config=Config::from_env()?;let db=PgPoolOptions::new().max_connections(30).connect(&config.database_url).await?;sqlx::migrate!("../infra/postgres").run(&db).await?;let redis=redis::Client::open(config.redis_url.clone())?;let message_ids=Arc::new(SnowflakeGenerator::new(config.snowflake_node_id)?);let state=AppState{config:config.clone(),db,redis,online:Arc::new(DashMap::new()),tickets:Arc::new(DashMap::new()),message_ids};let app=Router::new().route("/healthz",get(||async{"ok"})).route("/v1/auth/register",post(auth::register)).route("/v1/auth/login",post(auth::login)).route("/v1/auth/device/verify",post(auth::verify_device)).route("/v1/auth/password/reset/request",post(auth::request_reset)).route("/v1/auth/password/reset/confirm",post(auth::reset_password)).route("/v1/auth/password/change",post(auth::change_password)).route("/v1/chat/messages/send",post(chat_http::send)).route("/v1/chat/messages/history",post(chat_http::history)).route("/v1/realtime/bootstrap",post(realtime::bootstrap)).route("/v1/realtime/connect",get(realtime::socket)).layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive()).with_state(state);let listener=tokio::net::TcpListener::bind(config.bind).await?;tracing::info!(address=%config.bind,"server started");axum::serve(listener,app).await?;Ok(())}
