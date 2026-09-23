import Foundation
import CryptoKit
enum MTProto2KDF {
    static func authKeyId(_ key: Data) -> Data { precondition(key.count == 256); return Data(Insecure.SHA1.hash(data: key)).suffix(8) }
    static func keyAndIv(authKey: Data, messageKey: Data, clientToServer: Bool) -> (Data, Data) {
        let x = clientToServer ? 0 : 8
        let a = Data(SHA256.hash(data: messageKey + authKey[x..<(x + 36)]))
        let b = Data(SHA256.hash(data: authKey[(40 + x)..<(76 + x)] + messageKey))
        return (Data(a[0..<8] + b[8..<24] + a[24..<32]), Data(b[0..<8] + a[8..<24] + b[24..<32]))
    }
}
