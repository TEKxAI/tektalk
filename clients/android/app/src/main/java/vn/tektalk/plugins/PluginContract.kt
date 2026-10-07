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
    val pluginType: String,
    val minimumHostVersion: String,
    val minimumCoreAbi: Int,
    val valdiRuntime: String,
    val capabilities: Set<String>,
    val networkAllowlist: Set<String>,
    val storageNamespace: String,
    val trustTier: String,
    val required: Boolean,
    val rolloutPercentage: Int,
    val tab: PluginTab,
)

enum class PluginState { DISCOVERED, DOWNLOADED, VERIFIED, STAGED, ACTIVE, DEGRADED, ROLLED_BACK }

class PluginLifecycle {
    var state: PluginState = PluginState.DISCOVERED
        private set

    fun transition(next: PluginState) {
        val allowed = when (state) {
            PluginState.DISCOVERED -> setOf(PluginState.DOWNLOADED)
            PluginState.DOWNLOADED -> setOf(PluginState.VERIFIED)
            PluginState.VERIFIED -> setOf(PluginState.STAGED)
            PluginState.STAGED -> setOf(PluginState.ACTIVE, PluginState.ROLLED_BACK)
            PluginState.ACTIVE -> setOf(PluginState.DEGRADED)
            PluginState.DEGRADED -> setOf(PluginState.ROLLED_BACK, PluginState.ACTIVE)
            PluginState.ROLLED_BACK -> emptySet()
        }
        require(next in allowed) { "invalid plugin transition: $state -> $next" }
        state = next
    }
}

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
    fun updateUIEnvironment(environment: PluginUIEnvironment)
    val id: String
    suspend fun healthCheck(): Boolean
    fun unload()
}

interface HostCapabilityBroker {
    suspend fun invoke(pluginId: String, capability: String, request: ByteArray): ByteArray
}
enum class PluginColorScheme { LIGHT, DARK }
enum class PluginLayoutClass { COMPACT, EXPANDED }
data class PluginUIEnvironment(
    val locale: String,
    val colorScheme: PluginColorScheme,
    val layoutClass: PluginLayoutClass,
    val fontScale: Float,
)
