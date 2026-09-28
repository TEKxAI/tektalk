use std::{env, net::SocketAddr};

use tektalk_contracts::v1::{
    plugin_registry_service_server::{PluginRegistryService, PluginRegistryServiceServer},
    PluginDescriptor, PluginHealthReport, ReportPluginHealthResponse, ResolvePluginSetRequest,
    ResolvePluginSetResponse,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::EnvFilter;

#[derive(Default)]
struct Registry;

#[tonic::async_trait]
impl PluginRegistryService for Registry {
    async fn resolve_plugin_set(
        &self,
        request: Request<ResolvePluginSetRequest>,
    ) -> Result<Response<ResolvePluginSetResponse>, Status> {
        let request = request.into_inner();
        if request.host_version.is_empty() || request.platform.is_empty() {
            return Err(Status::invalid_argument("platform and host_version are required"));
        }
        let plugins = [
            ("tektalk.message", "MessagePlugin", 0, true),
            ("tektalk.ai", "AIPlugin", 1, false),
            ("tektalk.me", "MePlugin", 2, true),
        ]
        .into_iter()
        .map(|(id, entry_point, tab_order, required)| PluginDescriptor {
            id: id.into(),
            version: "0.1.0".into(),
            entry_point: entry_point.into(),
            module_url: format!("https://plugins.tektalk.vn/{id}/0.1.0/module.valdimodule"),
            sha256: Vec::new(),
            signature: Vec::new(),
            minimum_host_version: "0.1.0".into(),
            capabilities: Vec::new(),
            tab_order,
            required,
        })
        .collect();
        Ok(Response::new(ResolvePluginSetResponse {
            catalog_version: "development".into(),
            plugins,
            catalog_signature: Vec::new(),
            expires_at_unix_ms: 0,
        }))
    }

    async fn report_plugin_health(
        &self,
        request: Request<PluginHealthReport>,
    ) -> Result<Response<ReportPluginHealthResponse>, Status> {
        let report = request.into_inner();
        tracing::info!(plugin_id = %report.plugin_id, state = %report.state, "plugin health");
        Ok(Response::new(ReportPluginHealthResponse { accepted: true }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let address: SocketAddr = env::var("PLUGIN_REGISTRY_BIND")
        .unwrap_or_else(|_| "0.0.0.0:50051".into())
        .parse()?;
    tracing::info!(%address, "plugin registry started");
    Server::builder()
        .add_service(PluginRegistryServiceServer::new(Registry))
        .serve_with_shutdown(address, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
