use std::{env, net::SocketAddr};
use tektalk_contracts::v1::session_management_service_server::SessionManagementServiceServer;
use tektalk_platform_services::{init_tracing, shutdown, SessionManagementServiceImpl};
use tonic::transport::Server;

#[tokio::main]
async fn main()->anyhow::Result<()>{init_tracing();let address:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50054".into()).parse()?;tracing::info!(%address,"session management started");Server::builder().add_service(SessionManagementServiceServer::new(SessionManagementServiceImpl::default())).serve_with_shutdown(address,shutdown()).await?;Ok(())}
