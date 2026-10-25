mod auth; mod chat_http; mod commerce; mod config; mod crypto; mod error; mod realtime; mod secret_http; mod state; mod token;
use axum::{extract::State,routing::{get,post},Router};
use dashmap::DashMap;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::{cors::CorsLayer,services::ServeDir,trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use crate::{config::Config,state::AppState};
use tektalk_client_core::id::SnowflakeGenerator;

async fn ready(State(state): State<AppState>) -> Result<&'static str, axum::http::StatusCode> {
    sqlx::query("SELECT 1").execute(&state.db).await.map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
    Ok("ready")
}

#[tokio::main]
async fn main()->anyhow::Result<()>{
    tracing_subscriber::fmt().json().with_env_filter(EnvFilter::from_default_env()).init();
    let config=Config::from_env()?;
    let db=PgPoolOptions::new().max_connections(30).connect(&config.database_url).await?;
    sqlx::migrate!("../infra/postgres").run(&db).await?;
    let redis=redis::Client::open(config.redis_url.clone())?;
    let business_channel=tonic::transport::Endpoint::from_shared(config.business_solutions_url.clone())?.connect_lazy();
    let message_ids=Arc::new(SnowflakeGenerator::new(config.snowflake_node_id)?);
    let state=AppState{config:config.clone(),db,redis,business_channel,online:Arc::new(DashMap::new()),tickets:Arc::new(DashMap::new()),message_ids};
    let tcp_listener=tokio::net::TcpListener::bind(config.realtime_tcp_bind).await?;
    tracing::info!(address=%config.realtime_tcp_bind,"raw TCP realtime listener started");
    tokio::spawn(realtime::serve_tcp(tcp_listener,state.clone()));
    let app=Router::new()
        .route("/healthz",get(||async{"ok"})).route("/readyz",get(ready))
        .route("/v1/auth/register",post(auth::register)).route("/v1/auth/login",post(auth::login)).route("/v1/auth/device/verify",post(auth::verify_device)).route("/v1/auth/password/reset/request",post(auth::request_reset)).route("/v1/auth/password/reset/confirm",post(auth::reset_password)).route("/v1/auth/password/change",post(auth::change_password))
        .route("/v1/chat/messages/send",post(chat_http::send)).route("/v1/chat/messages/history",post(chat_http::history))
        .route("/v1/secret/devices/register",post(secret_http::register_bundle))
        .route("/v1/secret/conversations/enable",post(secret_http::enable))
        .route("/v1/secret/conversations/policy",post(secret_http::policy))
        .route("/v1/secret/conversations/bundles",post(secret_http::bundles))
        .route("/v1/secret/keys/publish",post(secret_http::publish_keys))
        .route("/v1/secret/keys/inbox",post(secret_http::key_packets))
        .route("/v1/secret/messages/send",post(secret_http::send))
        .route("/v1/secret/messages/history",post(secret_http::history))
        .route("/v1/commerce/plans",get(commerce::list_plans)).route("/v1/commerce/subscription",post(commerce::activate).delete(commerce::cancel)).route("/v1/commerce/entitlements",get(commerce::entitlements))
        .route("/v1/realtime/bootstrap",post(realtime::bootstrap)).route("/v1/realtime/connect",get(realtime::socket))
        .fallback_service(ServeDir::new("clients/web").append_index_html_on_directories(true))
        .layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive()).with_state(state);
    let listener=tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address=%config.bind,"server started");axum::serve(listener,app).await?;Ok(())
}
