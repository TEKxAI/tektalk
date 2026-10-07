import XCTest
@testable import TEKtalk

final class ClientLogicTests: XCTestCase {
    func testPluginUIEnvironmentCarriesLocaleThemeAndTabletLayout() {
        let environment = PluginUIEnvironment(locale: "vi-VN", colorScheme: .dark, layoutClass: .expanded, contentSizeCategory: "accessibility")
        XCTAssertEqual(environment.locale, "vi-VN")
        XCTAssertEqual(environment.colorScheme, .dark)
        XCTAssertEqual(environment.layoutClass, .expanded)
    }
    func testE164ValidationAcceptsInternationalNumber() {
        XCTAssertTrue(InputValidator.isE164("+84901234567"))
        XCTAssertFalse(InputValidator.isE164("0901234567"))
        XCTAssertFalse(InputValidator.isE164("+012345678"))
    }

    func testLoginRejectsWeakPassword() {
        XCTAssertEqual(
            InputValidator.auth(mode: .login, phone: "+84901234567", password: "short"),
            "Mật khẩu cần ít nhất 10 ký tự, một chữ hoa và một chữ số"
        )
    }

    func testRegistrationRequiresProfileAndSecurityFields() {
        XCTAssertEqual(
            InputValidator.auth(mode: .register, phone: "+84901234567", password: "Password123"),
            "Tên hiển thị không được để trống"
        )
        XCTAssertNil(InputValidator.auth(
            mode: .register,
            phone: "+84901234567",
            password: "Password123",
            displayName: "TEK User",
            securityQuestion: "First school?",
            securityAnswer: "TEK"
        ))
    }

    func testVerificationRequiresAnswerOnly() {
        XCTAssertNotNil(InputValidator.auth(mode: .verify, phone: "", password: "", securityAnswer: " "))
        XCTAssertNil(InputValidator.auth(mode: .verify, phone: "", password: "", securityAnswer: "answer"))
    }

    func testChatRequiresValidIdentifiersAndNonBlankMessage() {
        let id = UUID().uuidString
        XCTAssertEqual(InputValidator.chat(conversation: "bad", recipient: id, text: "Hello"), "Conversation UUID không hợp lệ")
        XCTAssertEqual(InputValidator.chat(conversation: id, recipient: id, text: "  "), "Tin nhắn không được để trống")
        XCTAssertNil(InputValidator.chat(conversation: id, recipient: id, text: "Hello"))
    }

    func testUnknownDeviceTransitionsToVerification() {
        let response = LoginResponse(status: "challenge", tokens: nil, challenge_id: UUID(), question: "Question")
        XCTAssertEqual(AuthFlow.nextMode(for: response), .verify)
    }

    func testMTProtoKDFProducesStableDirectionalKeyMaterial() {
        let authKey = Data((0..<256).map { UInt8($0) })
        let messageKey = Data((0..<16).map { UInt8($0) })
        let client = MTProto2KDF.keyAndIv(authKey: authKey, messageKey: messageKey, clientToServer: true)
        let server = MTProto2KDF.keyAndIv(authKey: authKey, messageKey: messageKey, clientToServer: false)
        XCTAssertEqual(client.0.count, 32)
        XCTAssertEqual(client.1.count, 32)
        XCTAssertNotEqual(client.0, server.0)
        XCTAssertEqual(MTProto2KDF.authKeyId(authKey).count, 8)
    }

    func testPluginLifecyclePromotesAndRollsBack() throws {
        var lifecycle = PluginLifecycle()
        try lifecycle.transition(to: .downloaded)
        try lifecycle.transition(to: .verified)
        try lifecycle.transition(to: .staged)
        try lifecycle.transition(to: .active)
        try lifecycle.transition(to: .degraded)
        try lifecycle.transition(to: .rolledBack)
        XCTAssertEqual(lifecycle.state, .rolledBack)
    }

    func testPluginLifecycleRejectsUnverifiedActivation() {
        var lifecycle = PluginLifecycle()
        XCTAssertThrowsError(try lifecycle.transition(to: .active))
    }
}
