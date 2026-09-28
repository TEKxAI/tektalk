use std::{env, net::SocketAddr};
use tektalk_contracts::v1::chat_service_server::ChatServiceServer;
use tektalk_platform_services::{init_tracing, shutdown, ChatServiceImpl};
use tonic::transport::Server;

#[tokio::main]
async fn main()->anyhow::Result<()>{init_tracing();let address:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50053".into()).parse()?;tracing::info!(%address,"chat service started");Server::builder().add_service(ChatServiceServer::new(ChatServiceImpl::default())).serve_with_shutdown(address,shutdown()).await?;Ok(())}
