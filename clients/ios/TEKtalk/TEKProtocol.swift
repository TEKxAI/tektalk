import Foundation
import CryptoKit
struct TEKSessionKeys { let privateKey: Curve25519.KeyAgreement.PrivateKey; let publicKey: Data; let symmetricKey: SymmetricKey }
enum TEKProtocol {
    static func derive(serverPublic: Data) throws -> TEKSessionKeys {
        let privateKey=Curve25519.KeyAgreement.PrivateKey();let server=try Curve25519.KeyAgreement.PublicKey(rawRepresentation:serverPublic)
        let shared=try privateKey.sharedSecretFromKeyAgreement(with:server)
        let key=shared.hkdfDerivedSymmetricKey(using:SHA256.self,salt:Data("tektalk-v1".utf8),sharedInfo:Data("realtime-session".utf8),outputByteCount:32)
        return TEKSessionKeys(privateKey:privateKey,publicKey:privateKey.publicKey.rawRepresentation,symmetricKey:key)
    }
    static func encrypt(key:SymmetricKey,kind:UInt8,session:UInt64,message:UInt64,sequence:UInt64,payload:Data)throws->Data{
        var header=Data([1,kind,0,0]);header.append(session.bigEndianData);header.append(message.bigEndianData);header.append(sequence.bigEndianData);header.append(UInt32(payload.count).bigEndianData)
        var nonceData=Data();nonceData.append(sequence.bigEndianData);nonceData.append(UInt32(truncatingIfNeeded:message).bigEndianData)
        let box=try ChaChaPoly.seal(payload,using:key,nonce:try ChaChaPoly.Nonce(data:nonceData),authenticating:header)
        return header+box.ciphertext+box.tag
    }
}
private extension FixedWidthInteger { var bigEndianData:Data { withUnsafeBytes(of:self.bigEndian){Data($0)} } }
