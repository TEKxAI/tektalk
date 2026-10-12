import AppKit
import Foundation
import Security

struct DesktopPluginArtifact: Codable, Sendable {
    let pluginID: String
    let version: String
    let url: URL
    let sha256: Data
    let signature: Data
    let publicKey: Data
}

enum DesktopPluginError: Error {
    case invalidIdentifier, signatureRejected, platformSignatureRejected
    case loadFailed(String), symbolMissing(String), abiMismatch, identityMismatch, unhealthy
}

@MainActor final class LoadedDesktopPlugin {
    typealias CreateView = @convention(c) (UnsafeRawPointer?) -> UnsafeMutableRawPointer?
    typealias DestroyView = @convention(c) (UnsafeMutableRawPointer?) -> Void
    private let module: UnsafeMutableRawPointer
    private let create: CreateView
    private let destroy: DestroyView
    private var rawView: UnsafeMutableRawPointer?

    init(module: UnsafeMutableRawPointer, create: @escaping CreateView, destroy: @escaping DestroyView) { self.module = module; self.create = create; self.destroy = destroy }
    func makeView() -> NSView? {
        if let rawView { return Unmanaged<NSView>.fromOpaque(rawView).takeUnretainedValue() }
        guard let pointer = create(nil) else { return nil }
        rawView = pointer
        return Unmanaged<NSView>.fromOpaque(pointer).takeUnretainedValue()
    }
    deinit { if let rawView { destroy(rawView) }; dlclose(module) }
}

final class DesktopPluginLoader {
    typealias ABIVersion = @convention(c) () -> UInt32
    typealias PluginID = @convention(c) () -> UnsafePointer<CChar>?
    typealias Health = @convention(c) () -> Int32
    private let root: URL

    init() throws {
        let support = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
        root = support.appendingPathComponent("TEKtalk/Plugins", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }

    func downloadAndInstall(_ artifact: DesktopPluginArtifact) async throws -> URL {
        guard artifact.url.scheme?.lowercased() == "https" else { throw DesktopPluginError.invalidIdentifier }
        let (bytes, response) = try await URLSession.shared.data(from: artifact.url)
        guard let http = response as? HTTPURLResponse, 200..<300 ~= http.statusCode else {
            throw DesktopPluginError.loadFailed("plugin download failed")
        }
        return try install(bytes, artifact: artifact)
    }

    func install(_ bytes: Data, artifact: DesktopPluginArtifact) throws -> URL {
        guard artifact.pluginID.range(of: #"^[a-z][a-z0-9.-]{2,127}$"#, options: .regularExpression) != nil else { throw DesktopPluginError.invalidIdentifier }
        guard RustCore.shared.verifyArtifact(bytes, sha256: artifact.sha256, signature: artifact.signature, publicKey: artifact.publicKey) else { throw DesktopPluginError.signatureRejected }
        let directory = root.appendingPathComponent(artifact.pluginID, isDirectory: true).appendingPathComponent(artifact.version, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let staged = directory.appendingPathComponent("plugin.dylib.staged")
        let active = directory.appendingPathComponent("plugin.dylib")
        try bytes.write(to: staged, options: .atomic)
        guard Self.validPlatformSignature(staged) else { try? FileManager.default.removeItem(at: staged); throw DesktopPluginError.platformSignatureRejected }
        if FileManager.default.fileExists(atPath: active.path) {
            _ = try FileManager.default.replaceItemAt(active, withItemAt: staged, backupItemName: "plugin.dylib.previous", options: .usingNewMetadataOnly)
        } else {
            try FileManager.default.moveItem(at: staged, to: active)
        }
        return active
    }

    @MainActor func load(pluginID expectedID: String, from url: URL) throws -> LoadedDesktopPlugin {
        let canonicalRoot = root.standardizedFileURL.path
        guard url.standardizedFileURL.path.hasPrefix(canonicalRoot + "/") else { throw DesktopPluginError.invalidIdentifier }
        guard let module = dlopen(url.path, RTLD_NOW | RTLD_LOCAL) else { throw DesktopPluginError.loadFailed(String(cString: dlerror())) }
        func symbol<T>(_ name: String, _: T.Type) throws -> T { guard let pointer = dlsym(module, name) else { throw DesktopPluginError.symbolMissing(name) }; return unsafeBitCast(pointer, to: T.self) }
        do {
            let abi = try symbol("tektalk_plugin_abi_version", ABIVersion.self)
            let identity = try symbol("tektalk_plugin_id", PluginID.self)
            let health = try symbol("tektalk_plugin_health_check", Health.self)
            let create = try symbol("tektalk_plugin_create_view", LoadedDesktopPlugin.CreateView.self)
            let destroy = try symbol("tektalk_plugin_destroy_view", LoadedDesktopPlugin.DestroyView.self)
            guard abi() == 1 else { throw DesktopPluginError.abiMismatch }
            guard identity().map({ String(cString: $0) }) == expectedID else { throw DesktopPluginError.identityMismatch }
            guard health() == 1 else { throw DesktopPluginError.unhealthy }
            return LoadedDesktopPlugin(module: module, create: create, destroy: destroy)
        } catch { dlclose(module); throw error }
    }

    private static func validPlatformSignature(_ url: URL) -> Bool {
        var code: SecStaticCode?
        guard SecStaticCodeCreateWithPath(url as CFURL, [], &code) == errSecSuccess, let code else { return false }
        return SecStaticCodeCheckValidity(code, SecCSFlags(rawValue: kSecCSCheckAllArchitectures), nil) == errSecSuccess
    }
}
