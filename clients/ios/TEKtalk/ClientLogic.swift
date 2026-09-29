import Foundation

enum AuthMode: Equatable {
    case login
    case register
    case verify
}

enum AuthFlow {
    static func nextMode(for response: LoginResponse) -> AuthMode {
        response.tokens == nil ? .verify : .login
    }
}

enum InputValidator {
    static func auth(
        mode: AuthMode,
        phone: String,
        password: String,
        displayName: String = "",
        securityQuestion: String = "",
        securityAnswer: String = ""
    ) -> String? {
        if mode == .verify {
            return securityAnswer.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                ? "Câu trả lời bảo mật không được để trống" : nil
        }
        guard isE164(phone) else { return "Số điện thoại phải theo định dạng E.164" }
        guard password.count >= 10,
              password.contains(where: { $0.isUppercase }),
              password.contains(where: { $0.isNumber }) else {
            return "Mật khẩu cần ít nhất 10 ký tự, một chữ hoa và một chữ số"
        }
        if mode == .register {
            guard !displayName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
                return "Tên hiển thị không được để trống"
            }
            guard !securityQuestion.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                  !securityAnswer.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
                return "Câu hỏi và câu trả lời bảo mật không được để trống"
            }
        }
        return nil
    }

    static func chat(conversation: String, recipient: String, text: String) -> String? {
        guard UUID(uuidString: conversation) != nil else { return "Conversation UUID không hợp lệ" }
        guard UUID(uuidString: recipient) != nil else { return "Recipient UUID không hợp lệ" }
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return "Tin nhắn không được để trống"
        }
        guard text.count <= 16_384 else { return "Tin nhắn vượt quá 16384 ký tự" }
        return nil
    }

    static func isE164(_ value: String) -> Bool {
        value.range(of: #"^\+[1-9][0-9]{7,14}$"#, options: .regularExpression) != nil
    }
}
