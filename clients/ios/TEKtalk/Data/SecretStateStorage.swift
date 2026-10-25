import Foundation
import Security

protocol SecretStateStorage {
    func load(account: String) throws -> Data?
    func save(account: String, data: Data) throws
}

/// Device-only Keychain data: never synchronized to iCloud or sent to the API.
struct KeychainSecretStateStorage: SecretStateStorage {
    private func query(_ account: String) -> [String: Any] {
        [kSecClass as String: kSecClassGenericPassword,
         kSecAttrService as String: "vn.tektalk.secret-chat-state.v1",
         kSecAttrAccount as String: account]
    }
    func load(account: String) throws -> Data? {
        var q = query(account); q[kSecReturnData as String] = true; q[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: CFTypeRef?
        let status = SecItemCopyMatching(q as CFDictionary, &result)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess, let data = result as? Data else { throw DatabaseError.keychain(status) }
        return data
    }
    func save(account: String, data: Data) throws {
        let q = query(account)
        let status = SecItemUpdate(q as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if status == errSecItemNotFound {
            var insert = q; insert[kSecValueData as String] = data
            insert[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            let added = SecItemAdd(insert as CFDictionary, nil)
            guard added == errSecSuccess else { throw DatabaseError.keychain(added) }
        } else if status != errSecSuccess { throw DatabaseError.keychain(status) }
    }
}
