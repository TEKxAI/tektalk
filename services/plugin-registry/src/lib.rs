use std::{collections::HashMap, env, sync::Arc};

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tektalk_contracts::v1::{
    plugin_registry_service_server::PluginRegistryService, PluginDescriptor,
    ReportPluginHealthRequest, ReportPluginHealthResponse, ResolveMiniAppsRequest,
    ResolveMiniAppsResponse, ResolvePluginSetRequest, ResolvePluginSetResponse,
};
use tonic::{Request, Response, Status};

const MESSAGE_MANIFEST: &str = include_str!("../../../plugins/message/manifest.json");
const AI_MANIFEST: &str = include_str!("../../../plugins/ai/manifest.json");
const ME_MANIFEST: &str = include_str!("../../../plugins/me/manifest.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    id: String,
    version: String,
    entry_point: String,
    plugin_type: String,
    minimum_host_version: String,
    minimum_core_abi: u32,
    valdi_runtime: String,
    capabilities: Vec<String>,
    network_allowlist: Vec<String>,
    storage_namespace: String,
    required: bool,
    rollout_percentage: u8,
    tab: Tab,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Tab { order: u32 }

#[derive(Clone, Debug, Deserialize)]
struct ArtifactMetadata {
    id: String,
    version: String,
    sha256_base64: String,
    signature_base64: String,
}

#[derive(Clone)]
pub struct Registry {
    manifests: Arc<Vec<Manifest>>,
    signing_key: Arc<SigningKey>,
    key_id: Arc<String>,
    artifact_base_url: Arc<String>,
    artifacts: Arc<HashMap<(String, String), ArtifactMetadata>>,
}

impl Registry {
    pub fn from_environment() -> anyhow::Result<Self> {
        let seed = env::var("PLUGIN_SIGNING_SEED").unwrap_or_else(|_| "tektalk-local-development-signing-key".into());
        let digest: [u8; 32] = Sha256::digest(seed.as_bytes()).into();
        let artifacts_json = env::var("PLUGIN_ARTIFACT_METADATA_JSON").unwrap_or_else(|_| "[]".into());
        let artifacts: Vec<ArtifactMetadata> = serde_json::from_str(&artifacts_json)?;
        let require_signed = env::var("PLUGIN_REQUIRE_SIGNED_ARTIFACTS").as_deref() == Ok("true");
        let registry = Self::new(
            SigningKey::from_bytes(&digest),
            env::var("PLUGIN_SIGNING_KEY_ID").unwrap_or_else(|_| "local-development".into()),
            env::var("PLUGIN_ARTIFACT_BASE_URL").unwrap_or_else(|_| "https://plugins.tektalk.vn".into()),
            artifacts,
        )?;
        if require_signed && registry.manifests.iter().any(|manifest| !registry.artifacts.contains_key(&(manifest.id.clone(), manifest.version.clone()))) {
            anyhow::bail!("production mode requires signed metadata for every plugin artifact");
        }
        Ok(registry)
    }

    fn new(signing_key: SigningKey, key_id: String, artifact_base_url: String, artifacts: Vec<ArtifactMetadata>) -> anyhow::Result<Self> {
        let manifests = [MESSAGE_MANIFEST, AI_MANIFEST, ME_MANIFEST].into_iter().map(serde_json::from_str).collect::<Result<Vec<Manifest>, _>>()?;
        let artifacts = artifacts.into_iter().map(|artifact| ((artifact.id.clone(), artifact.version.clone()), artifact)).collect();
        Ok(Self { manifests: Arc::new(manifests), signing_key: Arc::new(signing_key), key_id: Arc::new(key_id), artifact_base_url: Arc::new(artifact_base_url), artifacts: Arc::new(artifacts) })
    }

    pub fn signing_key_id(&self) -> &str { self.key_id.as_str() }

    fn compatible(&self, manifest: &Manifest, host_version: &str, core_abi: u32, runtime: &str) -> bool {
        version_at_least(host_version, &manifest.minimum_host_version) && core_abi >= manifest.minimum_core_abi && runtime == manifest.valdi_runtime
    }

    fn selected(&self, manifest: &Manifest, subject: &str) -> bool {
        manifest.required || cohort(subject, &manifest.id) < u32::from(manifest.rollout_percentage)
    }

    fn descriptor(&self, manifest: &Manifest) -> PluginDescriptor {
        let artifact = self.artifacts.get(&(manifest.id.clone(), manifest.version.clone()));
        PluginDescriptor {
            id: manifest.id.clone(), version: manifest.version.clone(), entry_point: manifest.entry_point.clone(),
            module_url: format!("{}/{}/{}/module.valdimodule", self.artifact_base_url, manifest.id, manifest.version),
            sha256: artifact.and_then(|value| STANDARD.decode(&value.sha256_base64).ok()).unwrap_or_default(),
            signature: artifact.and_then(|value| STANDARD.decode(&value.signature_base64).ok()).unwrap_or_default(),
            minimum_host_version: manifest.minimum_host_version.clone(), capabilities: manifest.capabilities.clone(),
            tab_order: manifest.tab.order, required: manifest.required, minimum_core_abi: manifest.minimum_core_abi, valdi_runtime: manifest.valdi_runtime.clone(),
            plugin_type: manifest.plugin_type.clone(), storage_namespace: manifest.storage_namespace.clone(), network_allowlist: manifest.network_allowlist.clone(), signing_key_id: self.key_id.to_string(),
        }
    }

    fn sign_catalog(&self, descriptors: &[PluginDescriptor]) -> Result<(Vec<u8>, Vec<u8>), Status> {
        let plugins: Vec<_> = descriptors.iter().map(|plugin| serde_json::json!({
            "id": plugin.id, "version": plugin.version, "entryPoint": plugin.entry_point,
            "moduleUrl": plugin.module_url, "sha256": STANDARD.encode(&plugin.sha256),
            "signature": STANDARD.encode(&plugin.signature), "minimumHostVersion": plugin.minimum_host_version,
            "minimumCoreAbi": plugin.minimum_core_abi, "valdiRuntime": plugin.valdi_runtime,
            "pluginType": plugin.plugin_type, "capabilities": plugin.capabilities,
            "storageNamespace": plugin.storage_namespace, "networkAllowlist": plugin.network_allowlist,
            "tabOrder": plugin.tab_order, "required": plugin.required, "signingKeyId": plugin.signing_key_id
        })).collect();
        let payload = serde_json::to_vec(&serde_json::json!({"version": "2", "keyId": &*self.key_id, "plugins": plugins})).map_err(|_| Status::internal("catalog serialization failed"))?;
        let signature = self.signing_key.sign(&payload).to_bytes().to_vec();
        Ok((payload, signature))
    }
}

#[tonic::async_trait]
impl PluginRegistryService for Registry {
    async fn resolve_plugin_set(&self, request: Request<ResolvePluginSetRequest>) -> Result<Response<ResolvePluginSetResponse>, Status> {
        let request = request.into_inner();
        if request.host_version.is_empty() || request.platform.is_empty() || request.device_id.is_empty() { return Err(Status::invalid_argument("platform, host_version and device_id are required")); }
        let plugins: Vec<_> = self.manifests.iter().filter(|manifest| manifest.plugin_type == "first_party_tab" && self.compatible(manifest, &request.host_version, request.core_abi, &request.valdi_runtime) && self.selected(manifest, &request.device_id)).map(|manifest| self.descriptor(manifest)).collect();
        if !plugins.iter().any(|plugin| plugin.id == "tektalk.message") || !plugins.iter().any(|plugin| plugin.id == "tektalk.me") { return Err(Status::failed_precondition("host is incompatible with required plugins")); }
        let (signed_catalog, catalog_signature) = self.sign_catalog(&plugins)?;
        Ok(Response::new(ResolvePluginSetResponse { catalog_version: "2".into(), plugins, catalog_signature, expires_at_unix_ms: (Utc::now() + Duration::hours(6)).timestamp_millis(), signed_catalog, signing_key_id: self.key_id.to_string() }))
    }

    async fn report_plugin_health(&self, request: Request<ReportPluginHealthRequest>) -> Result<Response<ReportPluginHealthResponse>, Status> {
        let report = request.into_inner();
        if !["staged", "active", "degraded", "rolled_back"].contains(&report.state.as_str()) { return Err(Status::invalid_argument("unknown plugin health state")); }
        tracing::info!(plugin_id = %report.plugin_id, version = %report.plugin_version, state = %report.state, failure_code = %report.failure_code, "plugin health");
        Ok(Response::new(ReportPluginHealthResponse { accepted: true }))
    }

    async fn resolve_mini_apps(&self, _request: Request<ResolveMiniAppsRequest>) -> Result<Response<ResolveMiniAppsResponse>, Status> {
        let plugins: Vec<_> = self.manifests.iter().filter(|manifest| manifest.plugin_type == "mini_app").map(|manifest| self.descriptor(manifest)).collect();
        let (signed_catalog, catalog_signature) = self.sign_catalog(&plugins)?;
        Ok(Response::new(ResolveMiniAppsResponse { mini_apps: plugins, catalog_version: "2".into(), signed_catalog, catalog_signature, signing_key_id: self.key_id.to_string(), expires_at_unix_ms: (Utc::now() + Duration::hours(1)).timestamp_millis() }))
    }
}

fn version_at_least(actual: &str, minimum: &str) -> bool {
    fn parse(value: &str) -> Option<(u32, u32, u32)> { let mut parts = value.split('.').map(str::parse); Some((parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?)) }
    matches!((parse(actual), parse(minimum)), (Some(actual), Some(minimum)) if actual >= minimum)
}

fn cohort(subject: &str, plugin_id: &str) -> u32 {
    let digest = Sha256::digest(format!("{subject}:{plugin_id}").as_bytes());
    u32::from_be_bytes(digest[..4].try_into().expect("fixed hash range")) % 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use tektalk_contracts::v1::RequestContext;

    fn registry() -> Registry { Registry::new(SigningKey::from_bytes(&[9; 32]), "test-key".into(), "https://cdn.example".into(), Vec::new()).unwrap() }
    fn request() -> ResolvePluginSetRequest { ResolvePluginSetRequest { context: Some(RequestContext::default()), platform: "ios".into(), architecture: "arm64".into(), host_version: "0.2.0".into(), locale: "vi-VN".into(), core_abi: 2, valdi_runtime: "1.1".into(), device_id: "device-1".into() } }

    #[tokio::test]
    async fn resolves_required_signed_tabs() {
        let response = registry().resolve_plugin_set(Request::new(request())).await.unwrap().into_inner();
        assert!(response.plugins.iter().any(|plugin| plugin.id == "tektalk.message"));
        assert!(response.plugins.iter().any(|plugin| plugin.id == "tektalk.me"));
        assert!(!response.catalog_signature.is_empty());
    }

    #[tokio::test]
    async fn rejects_incompatible_host() {
        let mut request = request(); request.core_abi = 1;
        assert!(registry().resolve_plugin_set(Request::new(request)).await.is_err());
    }
}
