import Foundation

struct RegisterRequest: Codable { let phone, display_name, password, security_question, security_answer, device_name: String }
struct LoginRequest: Codable { let phone, password: String; let device_id: UUID?; let device_name: String }
struct VerifyDeviceRequest: Codable { let challenge_id: UUID; let answer: String }
struct Tokens: Codable { let access_token, refresh_token: String; let user_id, device_id: UUID }
struct LoginResponse: Codable { let status: String; let tokens: Tokens?; let challenge_id: UUID?; let question: String? }
struct HistoryRequest: Codable { let conversation_id: UUID; let before_message_id: Int64?; let limit: Int }
struct SendMessageRequest: Codable { let conversation_id, recipient_id, client_message_id: UUID; let text: String }
struct ChatMessage: Codable, Identifiable, Equatable { let id: Int64; let conversation_id, sender_id, recipient_id, client_message_id: UUID; let body, created_at: String }
struct SendMessageResponse: Codable { let message: ChatMessage; let deduplicated: Bool }

enum AuthMode { case login, register, verify }

enum Validation {
    static func auth(mode: AuthMode, phone: String, password: String, name: String, question: String, answer: String) -> String? {
        if mode == .verify { return answer.trimmingCharacters(in: .whitespacesAndNewlines).count < 2 ? "Vui lòng nhập câu trả lời bảo mật" : nil }
        guard phone.range(of: #"^\+[1-9][0-9]{7,14}$"#, options: .regularExpression) != nil else { return "Số điện thoại phải theo chuẩn E.164" }
        guard password.count >= 10, password.contains(where: { $0.isUppercase }), password.contains(where: { $0.isNumber }) else { return "Mật khẩu cần ít nhất 10 ký tự, một chữ hoa và một chữ số" }
        if mode == .register {
            guard name.trimmingCharacters(in: .whitespaces).count >= 2 else { return "Tên hiển thị quá ngắn" }
            guard question.count >= 5, answer.count >= 2 else { return "Câu hỏi hoặc câu trả lời bảo mật quá ngắn" }
        }
        return nil
    }

    static func chat(conversation: String, recipient: String, text: String) -> String? {
        guard UUID(uuidString: conversation) != nil else { return "Conversation UUID không hợp lệ" }
        guard UUID(uuidString: recipient) != nil else { return "Recipient UUID không hợp lệ" }
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return "Tin nhắn không được để trống" }
        return text.count > 16_384 ? "Tin nhắn vượt quá 16384 ký tự" : nil
    }
}
