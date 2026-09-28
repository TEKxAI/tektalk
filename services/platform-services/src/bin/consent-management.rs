use std::{env, net::SocketAddr};
use tektalk_contracts::v1::consent_management_service_server::ConsentManagementServiceServer;
use tektalk_platform_services::{init_tracing, shutdown, ConsentManagementServiceImpl};
use tonic::transport::Server;

#[tokio::main]
async fn main()->anyhow::Result<()>{init_tracing();let address:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50055".into()).parse()?;tracing::info!(%address,"consent management started");Server::builder().add_service(ConsentManagementServiceServer::new(ConsentManagementServiceImpl::default())).serve_with_shutdown(address,shutdown()).await?;Ok(())}
