use std::{env, net::SocketAddr};
use tektalk_contracts::v1::account_service_server::AccountServiceServer;
use tektalk_platform_services::{init_tracing, shutdown, AccountServiceImpl};
use tonic::transport::Server;

#[tokio::main]
async fn main()->anyhow::Result<()>{init_tracing();let address:SocketAddr=env::var("SERVICE_BIND").unwrap_or_else(|_|"0.0.0.0:50052".into()).parse()?;tracing::info!(%address,"signin-signup started");Server::builder().add_service(AccountServiceServer::new(AccountServiceImpl::default())).serve_with_shutdown(address,shutdown()).await?;Ok(())}
