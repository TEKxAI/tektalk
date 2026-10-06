import XCTest
@testable import TEKtalkMac

final class ClientLogicTests: XCTestCase {
    func testAuthRejectsInvalidE164AndWeakPassword() {
        XCTAssertNotNil(Validation.auth(mode: .login, phone: "0912", password: "weak", name: "", question: "", answer: ""))
        XCTAssertNil(Validation.auth(mode: .login, phone: "+84912345678", password: "StrongPass1", name: "", question: "", answer: ""))
    }
    func testChatValidatesUUIDAndBody() {
        let id = UUID().uuidString
        XCTAssertNil(Validation.chat(conversation: id, recipient: UUID().uuidString, text: "hello"))
        XCTAssertNotNil(Validation.chat(conversation: id, recipient: "bad", text: "hello"))
    }
}
