use anyhow::Context;
use std::{env, net::SocketAddr};
use tektalk_business_solutions::CommerceService;
use tektalk_contracts::v1::{
    product_catalog_service_server::ProductCatalogServiceServer,
    subscription_service_server::SubscriptionServiceServer,
};
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let bind: SocketAddr = env::var("SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:50056".to_owned())
        .parse()
        .context("invalid SERVICE_BIND")?;
    let service = CommerceService::default();
    info!(%bind, "business solutions gRPC service listening");
    Server::builder()
        .add_service(ProductCatalogServiceServer::new(service.clone()))
        .add_service(SubscriptionServiceServer::new(service))
        .serve_with_shutdown(bind, async { let _ = tokio::signal::ctrl_c().await; })
        .await?;
    Ok(())
}

