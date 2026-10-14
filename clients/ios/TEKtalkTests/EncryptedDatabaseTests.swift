import XCTest
@testable import TEKtalk

final class EncryptedDatabaseTests: XCTestCase {
    private let testKeyProvider: (Int64) throws -> Data = { accountID in
        var value = accountID.bigEndian
        let seed = withUnsafeBytes(of: &value) { Data($0) }
        return Data(Array(repeating: seed, count: 4).joined())
    }

    func testEncryptedDatabaseCreatesConversationAndReopensWithStableKey() throws {
        let accountID = Int64.random(in: 1...Int64.max)

        do {
            let database = try EncryptedDatabase(accountID: accountID, keyProvider: testKeyProvider)
            try database.openConversation(101)
        }

        let reopened = try EncryptedDatabase(accountID: accountID, keyProvider: testKeyProvider)
        try reopened.openConversation(101)
        try reopened.openConversation(202)
    }

    func testEncryptedDatabaseRejectsInvalidIdentifiers() throws {
        XCTAssertThrowsError(try EncryptedDatabase(accountID: 0))
        let database = try EncryptedDatabase(
            accountID: Int64.random(in: 1...Int64.max),
            keyProvider: testKeyProvider
        )
        XCTAssertThrowsError(try database.openConversation(0))
    }
}
