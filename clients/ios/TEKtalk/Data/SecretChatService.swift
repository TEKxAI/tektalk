import Foundation

@MainActor final class SecretChatService {
    private let api: APIClient
    private let tokens: Tokens
    private let core: SecretDevice
    private let storage: SecretStateStorage?
    private let account: String
    private struct Cached: Codable { let wire: SecretMessage; let body: String }
    private struct Stored: Codable { let core: String; let messages: [Cached] }
    private var decrypted: [Int64: (SecretMessage, ChatMessage)] = [:]
    private let encoder = JSONEncoder(), decoder = JSONDecoder()

    init(tokens: Tokens, api: APIClient = APIClient(), storage: SecretStateStorage? = KeychainSecretStateStorage()) throws {
        self.tokens = tokens; self.api = api; self.storage = storage
        account = tokens.user_id.uuidString.lowercased() + ":" + tokens.device_id.uuidString.lowercased()
        do {
            if let data = try storage?.load(account: account) {
                let stored = try JSONDecoder().decode(Stored.self, from: data)
                guard let restored = SecretDevice(snapshot: stored.core), let bundleJSON = restored.publicBundleJSON else { throw URLError(.cannotDecodeContentData) }
                let bundle = try JSONDecoder().decode(CoreBundle.self, from: Data(bundleJSON.utf8))
                guard bundle.user_id == tokens.user_id.uuidString.lowercased(), bundle.device_id == tokens.device_id.uuidString.lowercased() else { throw URLError(.cannotDecodeContentData) }
                core = restored
                for entry in stored.messages {
                    let row = entry.wire
                    decrypted[row.id] = (row, ChatMessage(id: row.id, conversation_id: row.conversation_id, sender_id: row.sender_id, recipient_id: row.sender_id, client_message_id: row.client_message_id, body: entry.body, created_at: row.created_at))
                }
            } else {
                guard let created = NativeCore.shared.secretDevice(userID: tokens.user_id, deviceID: tokens.device_id) else { throw URLError(.cannotDecodeContentData) }
                core = created
            }
        } catch { throw error } // Never replace a corrupted/unavailable identity silently.
    }
    private func save() throws {
        guard let storage else { return }
        guard let snapshot = core.snapshotJSON else { throw URLError(.cannotDecodeContentData) }
        let rows = decrypted.values.map { Cached(wire: $0.0, body: $0.1.body) }
        try storage.save(account: account, data: encoder.encode(Stored(core: snapshot, messages: rows)))
    }
    private func json<T: Encodable>(_ value: T) throws -> String {
        String(decoding: try encoder.encode(value), as: UTF8.self)
    }
    func registerDevice() async throws {
        try save()
        guard let value = core.publicBundleJSON else { throw URLError(.cannotDecodeContentData) }
        let bundle = try decoder.decode(CoreBundle.self, from: Data(value.utf8))
        let _: ServerBundle = try await api.post("v1/secret/devices/register", body: RegisterSecretBundle(suite: bundle.suites[0], x25519_public: bundle.x25519_public, ml_kem_768_public: bundle.ml_kem_768_public), token: tokens.access_token)
    }
    func enable(conversation: UUID, kind: String, members: [UUID]) async throws {
        try await registerDevice()
        let policy: SecretPolicy = try await api.post("v1/secret/conversations/enable", body: EnableSecretRequest(conversation_id: conversation, kind: kind, member_ids: members), token: tokens.access_token)
        try await publish(conversation, policy)
    }
    private func publish(_ conversation: UUID, _ policy: SecretPolicy) async throws {
        let bundles: [ServerBundle] = try await api.post("v1/secret/conversations/bundles", body: SecretConversationRequest(conversation_id: conversation), token: tokens.access_token)
        guard bundles.contains(where: { $0.user_id != tokens.user_id }) else { throw URLError(.resourceUnavailable) }
        let coreBundles = bundles.map { CoreBundle(user_id: $0.user_id.uuidString.lowercased(), device_id: $0.device_id.uuidString.lowercased(), x25519_public: $0.x25519_public, ml_kem_768_public: $0.ml_kem_768_public, suites: [$0.suite]) }
        guard let wrappedJSON = core.begin(conversation: conversation.uuidString.lowercased(), kind: policy.kind, epoch: UInt64(policy.epoch), bundles: try json(coreBundles)) else { throw URLError(.cannotDecodeContentData) }
        try save()
        let wrapped = try decoder.decode([WrappedSenderKey].self, from: Data(wrappedJSON.utf8))
        let packets = try wrapped.map { packet -> PublishKeyPacket in
            guard let id = UUID(uuidString: packet.recipient_device_id) else { throw URLError(.cannotDecodeContentData) }
            return PublishKeyPacket(recipient_device_id: id, packet: packet)
        }
        let _: StatusResponse = try await api.post("v1/secret/keys/publish", body: PublishKeysRequest(conversation_id: conversation, epoch: policy.epoch, packets: packets), token: tokens.access_token)
    }
    private func prepare(_ conversation: UUID) async throws -> SecretPolicy {
        try await registerDevice()
        let policy: SecretPolicy = try await api.post("v1/secret/conversations/policy", body: SecretConversationRequest(conversation_id: conversation), token: tokens.access_token)
        let packets: [KeyPacket] = try await api.post("v1/secret/keys/inbox", body: SecretConversationRequest(conversation_id: conversation), token: tokens.access_token)
        for packet in packets {
            guard core.accept(conversation: conversation.uuidString.lowercased(), sender: packet.sender_device_id.uuidString.lowercased(), kind: policy.kind, epoch: UInt64(packet.epoch), wrapped: try json(packet.packet)) else { throw URLError(.cannotDecodeContentData) }
        }
        // A receiving device also needs its own sender key before replying.
        try await publish(conversation, policy)
        return policy
    }
    func history(conversation: UUID) async throws -> [ChatMessage] {
        _ = try await prepare(conversation)
        let rows: [SecretMessage] = try await api.post("v1/secret/messages/history", body: SecretConversationRequest(conversation_id: conversation), token: tokens.access_token)
        return try rows.map { row in
            if let (wire, message) = decrypted[row.id] {
                guard wire == row else { throw URLError(.cannotDecodeContentData) }
                return message
            }
            let wire = CoreEncryptedMessage(conversation_id: row.conversation_id.uuidString.lowercased(), epoch: UInt64(row.epoch), sender_device_id: row.sender_device_id.uuidString.lowercased(), counter: UInt64(row.counter), nonce: row.nonce, ciphertext: row.ciphertext, generation: UInt64(row.generation))
            guard let body = core.decrypt(message: try json(wire)) else { throw URLError(.cannotDecodeContentData) }
            let message = ChatMessage(id: row.id, conversation_id: row.conversation_id, sender_id: row.sender_id, recipient_id: row.sender_id, client_message_id: row.client_message_id, body: body, created_at: row.created_at)
            decrypted[row.id] = (row, message)
            try save()
            return message
        }
    }
    func send(conversation: UUID, text: String) async throws -> ChatMessage {
        let policy = try await prepare(conversation)
        guard let value = core.encrypt(conversation: conversation.uuidString.lowercased(), epoch: UInt64(policy.epoch), plaintext: text) else { throw URLError(.cannotDecodeContentData) }
        try save() // Persist the advanced counter before handing ciphertext to the network.
        let wire = try decoder.decode(CoreEncryptedMessage.self, from: Data(value.utf8))
        let response: SendSecretResponse = try await api.post("v1/secret/messages/send", body: SendSecretRequest(conversation_id: conversation, client_message_id: UUID(), epoch: Int64(wire.epoch), counter: Int64(wire.counter), nonce: wire.nonce, ciphertext: wire.ciphertext, generation: Int64(wire.generation)), token: tokens.access_token)
        let row = response.message
        return ChatMessage(id: row.id, conversation_id: row.conversation_id, sender_id: row.sender_id, recipient_id: row.sender_id, client_message_id: row.client_message_id, body: text, created_at: row.created_at)
    }
}
