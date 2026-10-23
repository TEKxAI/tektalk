import Foundation
struct RegisterRequest: Codable { let phone, display_name, password, security_question, security_answer, device_name: String }
struct LoginRequest: Codable { let phone: String; let password: String; let device_id: UUID?; let device_name: String }
struct Tokens: Codable { let access_token, refresh_token: String; let user_id, device_id: UUID }
struct LoginResponse: Codable { let status: String; let tokens: Tokens?; let challenge_id: UUID?; let question: String? }
struct VerifyDeviceRequest: Codable { let challenge_id: UUID; let answer: String }
struct BootstrapRequest: Codable { let access_token: String }
struct Bootstrap: Codable { let ticket, server_public_key: String; let expires_in_seconds, protocol_version: Int }
struct SendMessageRequest: Codable { let conversation_id, recipient_id, client_message_id: UUID; let text: String }
struct HistoryRequest: Codable { let conversation_id: UUID; let before_message_id: Int64?; let limit: Int }
struct ChatMessage: Codable, Identifiable { let id: Int64; let conversation_id, sender_id, recipient_id, client_message_id: UUID; let body, created_at: String }
struct SendMessageResponse: Codable { let message: ChatMessage; let deduplicated: Bool }
struct SecretConversationRequest:Codable{let conversation_id:UUID}
struct EnableSecretRequest:Codable{let conversation_id:UUID;let kind:String;let member_ids:[UUID]}
struct SecretPolicy:Codable{let conversation_id:UUID;let kind,suite:String;let epoch:Int64}
struct CoreBundle:Codable{let user_id,device_id:String;let x25519_public,ml_kem_768_public:[UInt8];let suites:[String]}
struct RegisterSecretBundle:Codable{let suite:String;let x25519_public,ml_kem_768_public:[UInt8]}
struct ServerBundle:Codable{let user_id,device_id:UUID;let suite:String;let x25519_public,ml_kem_768_public:[UInt8]}
struct SessionEnvelope:Codable{let suite,recipient_device_id:String;let ephemeral_x25519_public,ml_kem_ciphertext,transcript_hash:[UInt8]}
struct WrappedSenderKey:Codable{let recipient_device_id:String;let envelope:SessionEnvelope;let nonce,ciphertext:[UInt8];let epoch:UInt64}
struct PublishKeyPacket:Codable{let recipient_device_id:UUID;let packet:WrappedSenderKey}
struct PublishKeysRequest:Codable{let conversation_id:UUID;let epoch:Int64;let packets:[PublishKeyPacket]}
struct KeyPacket:Codable{let conversation_id:UUID;let epoch:Int64;let sender_device_id,recipient_device_id:UUID;let packet:WrappedSenderKey;let created_at:String}
struct CoreEncryptedMessage:Codable{let conversation_id:String;let epoch:UInt64;let sender_device_id:String;let counter:UInt64;let nonce,ciphertext:[UInt8]}
struct SendSecretRequest:Codable{let conversation_id,client_message_id:UUID;let epoch,counter:Int64;let nonce,ciphertext:[UInt8]}
struct SecretMessage:Codable{let id:Int64;let conversation_id:UUID;let epoch:Int64;let sender_id,sender_device_id,client_message_id:UUID;let counter:Int64;let nonce,ciphertext:[UInt8];let created_at:String}
struct SendSecretResponse:Codable{let message:SecretMessage;let deduplicated:Bool}
struct StatusResponse:Codable{let status:String;let epoch:Int64}
