import Darwin
import Foundation

/// Runtime C-ABI bridge. The app remains launchable in Xcode previews without
/// the library; release packaging copies libtektalk_client_core.dylib into Frameworks.
final class RustCore {
    static let shared = RustCore()
    private typealias Create = @convention(c) (UInt16) -> UnsafeMutableRawPointer?
    private typealias Next = @convention(c) (UnsafeMutableRawPointer?) -> Int64
    private typealias Destroy = @convention(c) (UnsafeMutableRawPointer?) -> Void
    private typealias Verify = @convention(c) (UnsafePointer<UInt8>?, Int, UnsafePointer<UInt8>?, UnsafePointer<UInt8>?, UnsafePointer<UInt8>?) -> Int32
    private typealias DatabaseOpen = @convention(c) (UnsafePointer<CChar>?, UnsafePointer<UInt8>?, Int, UInt32) -> UnsafeMutableRawPointer?
    private typealias DatabaseClose = @convention(c) (UnsafeMutableRawPointer?) -> Void
    private typealias ConversationOpen = @convention(c) (UnsafeMutableRawPointer?, Int64) -> Int32
    private let handle: UnsafeMutableRawPointer?
    private let create: Create?
    private let next: Next?
    private let destroy: Destroy?
    private let verify: Verify?
    private let databaseOpen: DatabaseOpen?
    private let databaseClose: DatabaseClose?
    private let conversationOpen: ConversationOpen?
    private var generator: UnsafeMutableRawPointer?

    private init() {
        let embedded = Bundle.main.privateFrameworksPath.map { $0 + "/libtektalk_client_core.dylib" }
        handle = embedded.flatMap { dlopen($0, RTLD_NOW | RTLD_LOCAL) } ?? dlopen("libtektalk_client_core.dylib", RTLD_NOW | RTLD_LOCAL)
        create = handle.flatMap { dlsym($0, "tektalk_snowflake_create") }.map { unsafeBitCast($0, to: Create.self) }
        next = handle.flatMap { dlsym($0, "tektalk_snowflake_next") }.map { unsafeBitCast($0, to: Next.self) }
        destroy = handle.flatMap { dlsym($0, "tektalk_snowflake_destroy") }.map { unsafeBitCast($0, to: Destroy.self) }
        verify = handle.flatMap { dlsym($0, "tektalk_plugin_verify_artifact") }.map { unsafeBitCast($0, to: Verify.self) }
        databaseOpen = handle.flatMap { dlsym($0, "tektalk_database_open") }.map { unsafeBitCast($0, to: DatabaseOpen.self) }
        databaseClose = handle.flatMap { dlsym($0, "tektalk_database_close") }.map { unsafeBitCast($0, to: DatabaseClose.self) }
        conversationOpen = handle.flatMap { dlsym($0, "tektalk_database_open_conversation") }.map { unsafeBitCast($0, to: ConversationOpen.self) }
        generator = create?(17)
    }
    deinit { destroy?(generator); if let handle { dlclose(handle) } }
    var isAvailable: Bool { generator != nil }
    func nextSnowflake() -> Int64? { guard let value = next?(generator), value >= 0 else { return nil }; return value }
    func verifyArtifact(_ data: Data, sha256: Data, signature: Data, publicKey: Data) -> Bool {
        guard let verify, sha256.count == 32, signature.count == 64, publicKey.count == 32 else { return false }
        return data.withUnsafeBytes { bytes in sha256.withUnsafeBytes { digest in signature.withUnsafeBytes { sig in publicKey.withUnsafeBytes { key in
            verify(bytes.bindMemory(to: UInt8.self).baseAddress, data.count, digest.bindMemory(to: UInt8.self).baseAddress, sig.bindMemory(to: UInt8.self).baseAddress, key.bindMemory(to: UInt8.self).baseAddress) == 1
        }}}}
    }
    func openDatabase(root: String, key: Data, maxOpen: UInt32 = 12) -> UnsafeMutableRawPointer? {
        guard key.count == 32, let databaseOpen else { return nil }
        return root.withCString { path in key.withUnsafeBytes { bytes in
            databaseOpen(path, bytes.bindMemory(to: UInt8.self).baseAddress, key.count, maxOpen)
        }}
    }
    func closeDatabase(_ database: UnsafeMutableRawPointer?) { databaseClose?(database) }
    func openConversation(_ database: UnsafeMutableRawPointer?, id: Int64) -> Bool {
        conversationOpen?(database, id) == 1
    }
}
