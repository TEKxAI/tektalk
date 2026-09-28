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
    let minimumHostVersion: String
    let capabilities: Set<String>
    let tab: PluginTab
}

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
