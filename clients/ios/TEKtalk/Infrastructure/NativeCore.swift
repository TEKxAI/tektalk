import Foundation

@_silgen_name("tektalk_snowflake_create") private func tektalkSnowflakeCreate(_ nodeID: UInt16) -> UnsafeMutableRawPointer?
@_silgen_name("tektalk_snowflake_next") private func tektalkSnowflakeNext(_ handle: UnsafeMutableRawPointer?) -> Int64
@_silgen_name("tektalk_snowflake_destroy") private func tektalkSnowflakeDestroy(_ handle: UnsafeMutableRawPointer?)
@_silgen_name("tektalk_secret_device_create") private func tektalkSecretCreate(_ user: UnsafePointer<CChar>, _ device: UnsafePointer<CChar>) -> UnsafeMutableRawPointer?
@_silgen_name("tektalk_secret_device_destroy") private func tektalkSecretDestroy(_ handle: UnsafeMutableRawPointer?)
@_silgen_name("tektalk_secret_public_bundle_json") private func tektalkSecretBundle(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?
@_silgen_name("tektalk_secret_snapshot_json") private func tektalkSecretSnapshot(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?
@_silgen_name("tektalk_secret_restore_json") private func tektalkSecretRestore(_ snapshot: UnsafePointer<CChar>) -> UnsafeMutableRawPointer?
@_silgen_name("tektalk_secret_begin_epoch_json") private func tektalkSecretBegin(_ handle: UnsafeMutableRawPointer?, _ conversation: UnsafePointer<CChar>, _ kind: UnsafePointer<CChar>, _ epoch: UInt64, _ bundles: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("tektalk_secret_accept_key_json") private func tektalkSecretAccept(_ handle: UnsafeMutableRawPointer?, _ conversation: UnsafePointer<CChar>, _ sender: UnsafePointer<CChar>, _ kind: UnsafePointer<CChar>, _ epoch: UInt64, _ wrapped: UnsafePointer<CChar>) -> Int32
@_silgen_name("tektalk_secret_encrypt_json") private func tektalkSecretEncrypt(_ handle: UnsafeMutableRawPointer?, _ conversation: UnsafePointer<CChar>, _ epoch: UInt64, _ plaintext: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("tektalk_secret_decrypt_json") private func tektalkSecretDecrypt(_ handle: UnsafeMutableRawPointer?, _ message: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("tektalk_string_destroy") private func tektalkStringDestroy(_ value: UnsafeMutablePointer<CChar>?)

final class NativeCore {
    static let shared = NativeCore()
    private let generator = tektalkSnowflakeCreate(17)
    private init() {}
    deinit { tektalkSnowflakeDestroy(generator) }
    var isAvailable: Bool { generator != nil }
    func snowflake() -> Int64? { let value = tektalkSnowflakeNext(generator); return value >= 0 ? value : nil }
    func secretDevice(userID:UUID,deviceID:UUID)->SecretDevice?{SecretDevice(userID:userID,deviceID:deviceID)}
}

final class SecretDevice {
    private var handle:UnsafeMutableRawPointer?
    init?(userID:UUID,deviceID:UUID){handle=userID.uuidString.lowercased().withCString{u in deviceID.uuidString.lowercased().withCString{d in tektalkSecretCreate(u,d)}};if handle==nil{return nil}}
    init?(snapshot:String){handle=snapshot.withCString{tektalkSecretRestore($0)};if handle==nil{return nil}}
    deinit{tektalkSecretDestroy(handle)}
    private func take(_ value:UnsafeMutablePointer<CChar>?)->String?{guard let value else{return nil};defer{tektalkStringDestroy(value)};return String(cString:value)}
    var publicBundleJSON:String?{take(tektalkSecretBundle(handle))}
    var snapshotJSON:String?{take(tektalkSecretSnapshot(handle))}
    func begin(conversation:String,kind:String,epoch:UInt64,bundles:String)->String?{conversation.withCString{c in kind.withCString{k in bundles.withCString{b in take(tektalkSecretBegin(handle,c,k,epoch,b))}}}}
    func accept(conversation:String,sender:String,kind:String,epoch:UInt64,wrapped:String)->Bool{conversation.withCString{c in sender.withCString{s in kind.withCString{k in wrapped.withCString{w in tektalkSecretAccept(handle,c,s,k,epoch,w)==1}}}}}
    func encrypt(conversation:String,epoch:UInt64,plaintext:String)->String?{conversation.withCString{c in plaintext.withCString{p in take(tektalkSecretEncrypt(handle,c,epoch,p))}}}
    func decrypt(message:String)->String?{message.withCString{m in take(tektalkSecretDecrypt(handle,m))}}
}
