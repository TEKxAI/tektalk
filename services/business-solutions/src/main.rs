use anyhow::{Context,Result};
use sqlx::postgres::PgPoolOptions;
use std::{env,net::SocketAddr};
use tektalk_business_solutions::CommerceService;
use tektalk_contracts::v1::{product_catalog_service_server::ProductCatalogServiceServer,subscription_service_server::SubscriptionServiceServer};
use tonic::transport::Server;

#[tokio::main]
async fn main()->Result<()>{tracing_subscriber::fmt().json().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();let bind:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50056".into()).parse().context("invalid SERVICE_BIND")?;let database_url=env::var("DATABASE_URL").context("DATABASE_URL is required")?;let pool=PgPoolOptions::new().max_connections(15).connect(&database_url).await?;sqlx::migrate!("../../infra/postgres").run(&pool).await?;let service=CommerceService::with_postgres(pool);tracing::info!(%bind,"business solutions gRPC service listening");Server::builder().add_service(ProductCatalogServiceServer::new(service.clone())).add_service(SubscriptionServiceServer::new(service)).serve_with_shutdown(bind,async{let _=tokio::signal::ctrl_c().await;}).await?;Ok(())}
