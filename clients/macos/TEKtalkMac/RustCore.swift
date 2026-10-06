import Darwin
import Foundation

/// Runtime C-ABI bridge. The app remains launchable in Xcode previews without
/// the library; release packaging copies libtektalk_client_core.dylib into Frameworks.
final class RustCore {
    static let shared = RustCore()
    private typealias Create = @convention(c) (UInt16) -> UnsafeMutableRawPointer?
    private typealias Next = @convention(c) (UnsafeMutableRawPointer?) -> Int64
    private typealias Destroy = @convention(c) (UnsafeMutableRawPointer?) -> Void
    private let handle: UnsafeMutableRawPointer?
    private let create: Create?
    private let next: Next?
    private let destroy: Destroy?
    private var generator: UnsafeMutableRawPointer?

    private init() {
        let embedded = Bundle.main.privateFrameworksPath.map { $0 + "/libtektalk_client_core.dylib" }
        handle = embedded.flatMap { dlopen($0, RTLD_NOW | RTLD_LOCAL) } ?? dlopen("libtektalk_client_core.dylib", RTLD_NOW | RTLD_LOCAL)
        create = handle.flatMap { dlsym($0, "tektalk_snowflake_create") }.map { unsafeBitCast($0, to: Create.self) }
        next = handle.flatMap { dlsym($0, "tektalk_snowflake_next") }.map { unsafeBitCast($0, to: Next.self) }
        destroy = handle.flatMap { dlsym($0, "tektalk_snowflake_destroy") }.map { unsafeBitCast($0, to: Destroy.self) }
        generator = create?(17)
    }
    deinit { destroy?(generator); if let handle { dlclose(handle) } }
    var isAvailable: Bool { generator != nil }
    func nextSnowflake() -> Int64? { guard let value = next?(generator), value >= 0 else { return nil }; return value }
}
