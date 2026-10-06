import Foundation

@_silgen_name("tektalk_snowflake_create") private func tektalkSnowflakeCreate(_ nodeID: UInt16) -> UnsafeMutableRawPointer?
@_silgen_name("tektalk_snowflake_next") private func tektalkSnowflakeNext(_ handle: UnsafeMutableRawPointer?) -> Int64
@_silgen_name("tektalk_snowflake_destroy") private func tektalkSnowflakeDestroy(_ handle: UnsafeMutableRawPointer?)

final class NativeCore {
    static let shared = NativeCore()
    private let generator = tektalkSnowflakeCreate(17)
    private init() {}
    deinit { tektalkSnowflakeDestroy(generator) }
    var isAvailable: Bool { generator != nil }
    func snowflake() -> Int64? { let value = tektalkSnowflakeNext(generator); return value >= 0 ? value : nil }
}
