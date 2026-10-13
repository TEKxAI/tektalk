import XCTest
@testable import TEKtalk

final class EncryptedDatabaseTests: XCTestCase {
    func testEncryptedDatabaseCreatesConversationAndReopensWithKeychainKey() throws {
        let accountID = Int64.random(in: 1...Int64.max)

        do {
            let database = try EncryptedDatabase(accountID: accountID)
            try database.openConversation(101)
        }

        let reopened = try EncryptedDatabase(accountID: accountID)
        try reopened.openConversation(101)
        try reopened.openConversation(202)
    }

    func testEncryptedDatabaseRejectsInvalidIdentifiers() throws {
        XCTAssertThrowsError(try EncryptedDatabase(accountID: 0))
        let database = try EncryptedDatabase(accountID: Int64.random(in: 1...Int64.max))
        XCTAssertThrowsError(try database.openConversation(0))
    }
}
