use std::{env, net::SocketAddr};

use tektalk_contracts::v1::plugin_registry_service_server::PluginRegistryServiceServer;
use tektalk_plugin_registry::Registry;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().with_env_filter(EnvFilter::from_default_env()).init();
    let address: SocketAddr = env::var("PLUGIN_REGISTRY_BIND").unwrap_or_else(|_| "0.0.0.0:50051".into()).parse()?;
    let registry = Registry::from_environment()?;
    tracing::info!(%address, signing_key_id = registry.signing_key_id(), "plugin registry started");
    Server::builder().add_service(PluginRegistryServiceServer::new(registry)).serve_with_shutdown(address, async { let _ = tokio::signal::ctrl_c().await; }).await?;
    Ok(())
}
