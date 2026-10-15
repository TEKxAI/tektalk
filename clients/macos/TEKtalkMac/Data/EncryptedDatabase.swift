import Foundation
import Security

final class EncryptedDatabase {
    private let core: RustCore
    private var handle: UnsafeMutableRawPointer?

    init(accountID: Int64, core: RustCore = .shared) throws {
        guard accountID > 0 else { throw MacDatabaseError.invalidAccount }
        self.core = core
        let base = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
        let root = base.appending(path: "TEKtalk/accounts/\(accountID)", directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        var key = try MacDatabaseKey.getOrCreate(accountID: accountID)
        defer { key.resetBytes(in: 0..<key.count) }
        handle = core.openDatabase(root: root.path, key: key)
        guard handle != nil else { throw MacDatabaseError.cipherUnavailableOrInvalidKey }
    }

    deinit { core.closeDatabase(handle) }

    func openConversation(_ conversationID: Int64) throws {
        guard conversationID > 0, core.openConversation(handle, id: conversationID) else {
            throw MacDatabaseError.openConversationFailed
        }
    }
}

enum MacDatabaseError: Error { case invalidAccount, keychain(OSStatus), invalidStoredKey, cipherUnavailableOrInvalidKey, openConversationFailed }

private enum MacDatabaseKey {
    static func getOrCreate(accountID: Int64) throws -> Data {
        let account = "account.\(accountID)"
        let base: [String: Any] = [kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: "vn.tektalk.database-key", kSecAttrAccount as String: account]
        var query = base; query[kSecReturnData as String] = true; query[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        if status == errSecSuccess {
            guard let key = result as? Data, key.count == 32 else { throw MacDatabaseError.invalidStoredKey }
            return key
        }
        guard status == errSecItemNotFound else { throw MacDatabaseError.keychain(status) }
        var key = Data(count: 32)
        let randomStatus = key.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, 32, $0.baseAddress!) }
        guard randomStatus == errSecSuccess else { throw MacDatabaseError.keychain(randomStatus) }
        var insert = base; insert[kSecValueData as String] = key
        let addStatus = SecItemAdd(insert as CFDictionary, nil)
        guard addStatus == errSecSuccess else { throw MacDatabaseError.keychain(addStatus) }
        return key
    }
}
