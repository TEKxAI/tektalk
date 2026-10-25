import XCTest
@testable import TEKtalk

/// The transport is deterministic, but every bundle, wrap and message passes
/// through the actual linked Rust cryptography, including the peer's reply.
@MainActor final class SecretChatTests: XCTestCase {
    func testOptInSendReceiveReloadAndTamperRejection() async throws {
        let user = UUID(), device = UUID(), peerID = UUID(), peerDeviceID = UUID(), conversation = UUID()
        let tokens = Tokens(access_token: "secret-test-token", refresh_token: "r", user_id: user, device_id: device)
        let peer = try XCTUnwrap(SecretDevice(userID: peerID, deviceID: peerDeviceID))
        let encoder = JSONEncoder(), decoder = JSONDecoder()
        let peerBundle = try decoder.decode(CoreBundle.self, from: Data(try XCTUnwrap(peer.publicBundleJSON).utf8))
        var own: CoreBundle?
        var rows: [[String: Any]] = []
        var corrupt = false
        SecretProtocol.handler = { request in
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer secret-test-token")
            let input = try XCTUnwrap(JSONSerialization.jsonObject(with: request.httpBody ?? request.httpBodyStreamData()) as? [String: Any])
            let output: Any
            switch request.url!.path {
            case "/v1/secret/devices/register":
                own = CoreBundle(user_id: user.uuidString.lowercased(), device_id: device.uuidString.lowercased(), x25519_public: try XCTUnwrap(input["x25519_public"] as? [UInt8]), ml_kem_768_public: try XCTUnwrap(input["ml_kem_768_public"] as? [UInt8]), suites: [try XCTUnwrap(input["suite"] as? String)])
                output = ["user_id": user.uuidString, "device_id": device.uuidString, "suite": own!.suites[0], "x25519_public": own!.x25519_public, "ml_kem_768_public": own!.ml_kem_768_public]
            case "/v1/secret/conversations/enable", "/v1/secret/conversations/policy":
                output = ["conversation_id": conversation.uuidString, "kind": "group", "suite": peerBundle.suites[0], "epoch": 1]
            case "/v1/secret/conversations/bundles":
                output = [own!, peerBundle].map { ["user_id": $0.user_id, "device_id": $0.device_id, "suite": $0.suites[0], "x25519_public": $0.x25519_public, "ml_kem_768_public": $0.ml_kem_768_public] as [String: Any] }
            case "/v1/secret/keys/publish":
                let packets = try XCTUnwrap(input["packets"] as? [[String: Any]])
                for packet in packets where (packet["recipient_device_id"] as? String)?.lowercased() == peerDeviceID.uuidString.lowercased() {
                    let wrapped = String(decoding: try JSONSerialization.data(withJSONObject: packet["packet"]!), as: UTF8.self)
                    XCTAssertTrue(peer.accept(conversation: conversation.uuidString.lowercased(), sender: device.uuidString.lowercased(), kind: "group", epoch: 1, wrapped: wrapped))
                }
                output = ["status": "published", "epoch": 1]
            case "/v1/secret/keys/inbox":
                let bundles = String(decoding: try encoder.encode([own!]), as: UTF8.self)
                let packets = try decoder.decode([WrappedSenderKey].self, from: Data(try XCTUnwrap(peer.begin(conversation: conversation.uuidString.lowercased(), kind: "group", epoch: 1, bundles: bundles)).utf8))
                output = [["conversation_id": conversation.uuidString, "epoch": 1, "sender_device_id": peerDeviceID.uuidString, "recipient_device_id": device.uuidString, "packet": try JSONSerialization.jsonObject(with: encoder.encode(packets[0])), "created_at": "now"]]
            case "/v1/secret/messages/send":
                XCTAssertNil(input["body"]); XCTAssertNil(input["text"])
                var wire = input; wire["sender_device_id"] = device.uuidString.lowercased()
                let value = String(decoding: try JSONSerialization.data(withJSONObject: wire), as: UTF8.self)
                XCTAssertEqual(peer.decrypt(message: value), "outgoing secret")
                var row = wire; row["id"] = 2; row["sender_id"] = user.uuidString; row["created_at"] = "now"
                rows.append(row)
                output = ["message": row, "deduplicated": false]
            case "/v1/secret/messages/history":
                if rows.isEmpty {
                    let value = try XCTUnwrap(peer.encrypt(conversation: conversation.uuidString.lowercased(), epoch: 1, plaintext: "incoming secret"))
                    var row = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(value.utf8)) as? [String: Any])
                    row["id"] = 1; row["sender_id"] = peerID.uuidString; row["client_message_id"] = UUID().uuidString; row["created_at"] = "now"; rows.append(row)
                }
                if corrupt { rows[0]["ciphertext"] = [UInt8](repeating: 0, count: 32) }
                output = rows
            default: throw URLError(.unsupportedURL)
            }
            return try JSONSerialization.data(withJSONObject: output)
        }
        defer { SecretProtocol.handler = nil }
        let config = URLSessionConfiguration.ephemeral; config.protocolClasses = [SecretProtocol.self]
        let api = APIClient(baseURL: URL(string: "https://secret.test")!, session: URLSession(configuration: config))
        let storage = MemorySecretStorage()
        let service = try XCTUnwrap(SecretChatService(tokens: tokens, api: api, storage: storage))
        try await service.enable(conversation: conversation, kind: "group", members: [peerID])
        let first = try await service.history(conversation: conversation)
        XCTAssertEqual(first.map(\.body), ["incoming secret"])
        let sent = try await service.send(conversation: conversation, text: "outgoing secret")
        XCTAssertEqual(sent.body, "outgoing secret")
        try await service.enable(conversation: conversation, kind: "group", members: [peerID])
        let second = try await service.history(conversation: conversation)
        let reload = try await service.history(conversation: conversation)
        XCTAssertEqual(second.map(\.body), ["incoming secret", "outgoing secret"])
        XCTAssertEqual(reload.map(\.body), second.map(\.body))
        let reopened = try XCTUnwrap(SecretChatService(tokens: tokens, api: api, storage: storage))
        let restored = try await reopened.history(conversation: conversation)
        XCTAssertEqual(restored.map(\.body), second.map(\.body))
        corrupt = true
        do { _ = try await service.history(conversation: conversation); XCTFail("Changed ciphertext must fail") }
        catch { XCTAssertEqual((error as? URLError)?.code, .cannotDecodeContentData) }
    }
}

private final class MemorySecretStorage: SecretStateStorage {
    var values: [String: Data] = [:]
    func load(account: String) throws -> Data? { values[account] }
    func save(account: String, data: Data) throws { values[account] = data }
}

private final class SecretProtocol: URLProtocol {
    static var handler: ((URLRequest) throws -> Data)?
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        do {
            let data = try XCTUnwrap(Self.handler)(request)
            client?.urlProtocol(self, didReceive: HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data); client?.urlProtocolDidFinishLoading(self)
        } catch { client?.urlProtocol(self, didFailWithError: error) }
    }
    override func stopLoading() {}
}
private extension URLRequest {
    func httpBodyStreamData() throws -> Data {
        let stream = try XCTUnwrap(httpBodyStream); stream.open(); defer { stream.close() }
        var result = Data(), buffer = [UInt8](repeating: 0, count: 4096)
        while stream.hasBytesAvailable { let n = stream.read(&buffer, maxLength: buffer.count); guard n >= 0 else { throw URLError(.cannotDecodeContentData) }; if n == 0 { break }; result.append(contentsOf: buffer.prefix(n)) }
        return result
    }
}
