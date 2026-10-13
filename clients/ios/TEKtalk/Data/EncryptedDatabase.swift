import Foundation
import Security

@_silgen_name("tektalk_database_open")
private func databaseOpen(
    _ root: UnsafePointer<CChar>,
    _ key: UnsafePointer<UInt8>,
    _ keyLength: Int,
    _ maxOpen: UInt32
) -> UnsafeMutableRawPointer?
@_silgen_name("tektalk_database_close")
private func databaseClose(_ handle: UnsafeMutableRawPointer?)
@_silgen_name("tektalk_database_open_conversation")
private func databaseOpenConversation(_ handle: UnsafeMutableRawPointer?, _ conversationID: Int64) -> Int32

/// Native SQLCipher store. The database key is random and persisted only in Keychain.
final class EncryptedDatabase {
    private var handle: UnsafeMutableRawPointer?

    init(accountID: Int64) throws {
        guard accountID > 0 else { throw DatabaseError.invalidAccount }
        let root = try Self.accountRoot(accountID: accountID)
        var key = try KeychainDatabaseKey.getOrCreate(accountID: accountID)
        defer { key.resetBytes(in: 0..<key.count) }
        handle = root.path.withCString { path in
            key.withUnsafeBytes { bytes in
                databaseOpen(path, bytes.bindMemory(to: UInt8.self).baseAddress!, key.count, 8)
            }
        }
        guard handle != nil else { throw DatabaseError.cipherUnavailableOrInvalidKey }
    }

    deinit { databaseClose(handle) }

    func openConversation(_ conversationID: Int64) throws {
        guard conversationID > 0, databaseOpenConversation(handle, conversationID) == 1 else {
            throw DatabaseError.openConversationFailed
        }
    }

    private static func accountRoot(accountID: Int64) throws -> URL {
        let base = try FileManager.default.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )
        let root = base.appending(path: "TEKtalk/accounts/\(accountID)", directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        return root
    }
}

enum DatabaseError: Error {
    case invalidAccount
    case keychain(OSStatus)
    case invalidStoredKey
    case cipherUnavailableOrInvalidKey
    case openConversationFailed
}

private enum KeychainDatabaseKey {
    static func getOrCreate(accountID: Int64) throws -> Data {
        let account = "account.\(accountID)"
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "vn.tektalk.database-key",
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        if status == errSecSuccess {
            guard let key = result as? Data, key.count == 32 else { throw DatabaseError.invalidStoredKey }
            return key
        }
        guard status == errSecItemNotFound else { throw DatabaseError.keychain(status) }
        var key = Data(count: 32)
        let randomStatus = key.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, 32, $0.baseAddress!) }
        guard randomStatus == errSecSuccess else { throw DatabaseError.keychain(randomStatus) }
        var insert = query
        insert.removeValue(forKey: kSecReturnData as String)
        insert.removeValue(forKey: kSecMatchLimit as String)
        insert[kSecValueData as String] = key
        insert[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        let addStatus = SecItemAdd(insert as CFDictionary, nil)
        guard addStatus == errSecSuccess else { throw DatabaseError.keychain(addStatus) }
        return key
    }
}
