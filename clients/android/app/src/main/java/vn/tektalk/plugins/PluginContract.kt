package vn.tektalk.plugins

import kotlinx.serialization.Serializable

@Serializable
data class PluginTab(
    val titleKey: String,
    val icon: String,
    val order: Int,
)

@Serializable
data class PluginManifest(
    val schemaVersion: Int,
    val id: String,
    val version: String,
    val entryPoint: String,
    val minimumHostVersion: String,
    val capabilities: Set<String>,
    val tab: PluginTab,
)

interface PluginArtifactStore {
    suspend fun stage(pluginId: String, version: String, bytes: ByteArray): String
    suspend fun activate(pluginId: String, stagedPath: String)
    suspend fun rollback(pluginId: String)
}

interface PluginVerifier {
    fun verifyCatalog(catalog: ByteArray, signature: ByteArray): Boolean
    fun verifyArtifact(bytes: ByteArray, sha256: ByteArray, signature: ByteArray): Boolean
}

interface ValdiPluginRuntime {
    suspend fun load(manifest: PluginManifest, artifactPath: String): PluginHandle
}

interface PluginHandle {
    val id: String
    suspend fun healthCheck(): Boolean
    fun unload()
}

interface HostCapabilityBroker {
    suspend fun invoke(pluginId: String, capability: String, request: ByteArray): ByteArray
}
