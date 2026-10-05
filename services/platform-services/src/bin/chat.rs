use std::{env, net::SocketAddr};
use tektalk_contracts::v1::chat_service_server::ChatServiceServer;
use tektalk_platform_services::{init_tracing, shutdown, ChatServiceImpl};
use tonic::transport::Server;

#[tokio::main]
async fn main()->anyhow::Result<()>{init_tracing();let address:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50053".into()).parse()?;let node_id=env::var("SNOWFLAKE_NODE_ID").unwrap_or_else(|_|"1".into()).parse()?;let service=ChatServiceImpl::new(node_id)?;tracing::info!(%address,node_id,"chat service started");Server::builder().add_service(ChatServiceServer::new(service)).serve_with_shutdown(address,shutdown()).await?;Ok(())}
