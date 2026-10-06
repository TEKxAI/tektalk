import Foundation

struct PluginTab: Codable, Sendable {
    let titleKey: String
    let icon: String
    let order: Int
}

struct PluginManifest: Codable, Sendable {
    let schemaVersion: Int
    let id: String
    let version: String
    let entryPoint: String
    let pluginType: String
    let minimumHostVersion: String
    let minimumCoreAbi: Int
    let valdiRuntime: String
    let capabilities: Set<String>
    let networkAllowlist: Set<String>
    let storageNamespace: String
    let trustTier: String
    let required: Bool
    let rolloutPercentage: Int
    let tab: PluginTab
}

enum PluginState: String, Sendable {
    case discovered, downloaded, verified, staged, active, degraded, rolledBack
}

struct PluginLifecycle: Sendable {
    private(set) var state: PluginState = .discovered

    mutating func transition(to next: PluginState) throws {
        let allowed: Set<PluginState>
        switch state {
        case .discovered: allowed = [.downloaded]
        case .downloaded: allowed = [.verified]
        case .verified: allowed = [.staged]
        case .staged: allowed = [.active, .rolledBack]
        case .active: allowed = [.degraded]
        case .degraded: allowed = [.rolledBack, .active]
        case .rolledBack: allowed = []
        }
        guard allowed.contains(next) else { throw PluginLifecycleError.invalidTransition(state, next) }
        state = next
    }
}

enum PluginLifecycleError: Error, Equatable { case invalidTransition(PluginState, PluginState) }

protocol PluginArtifactStore: Sendable {
    func stage(pluginID: String, version: String, bytes: Data) async throws -> URL
    func activate(pluginID: String, stagedURL: URL) async throws
    func rollback(pluginID: String) async throws
}

protocol PluginVerifier: Sendable {
    func verifyCatalog(_ catalog: Data, signature: Data) -> Bool
    func verifyArtifact(_ bytes: Data, sha256: Data, signature: Data) -> Bool
}

protocol ValdiPluginRuntime: Sendable {
    func load(manifest: PluginManifest, artifactURL: URL) async throws -> any PluginHandle
}

protocol PluginHandle: AnyObject, Sendable {
    var id: String { get }
    func healthCheck() async -> Bool
    func unload()
}

protocol HostCapabilityBroker: Sendable {
    func invoke(pluginID: String, capability: String, request: Data) async throws -> Data
}
