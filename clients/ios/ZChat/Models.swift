import Foundation
struct RegisterRequest: Codable { let phone, display_name, password, security_question, security_answer, device_name: String }
struct LoginRequest: Codable { let phone: String; let password: String; let device_id: UUID?; let device_name: String }
struct Tokens: Codable { let access_token, refresh_token: String; let user_id, device_id: UUID }
struct LoginResponse: Codable { let status: String; let tokens: Tokens?; let challenge_id: UUID?; let question: String? }
struct BootstrapRequest: Codable { let access_token: String }
struct Bootstrap: Codable { let ticket, server_public_key: String; let expires_in_seconds, protocol_version: Int }

