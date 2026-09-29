import XCTest
@testable import TEKtalk

final class APIClientTests: XCTestCase {
    override func tearDown() {
        URLProtocolStub.handler = nil
        super.tearDown()
    }

    func testRequestContainsJSONAndBearerToken() throws {
        let client = APIClient(baseURL: URL(string: "https://api.tektalk.test")!)
        let body = HistoryRequest(conversation_id: UUID(), before_message_id: nil, limit: 50)
        let request = try client.makeRequest("v1/chat/messages/history", body: body, token: "access-token")

        XCTAssertEqual(request.httpMethod, "POST")
        XCTAssertEqual(request.url?.absoluteString, "https://api.tektalk.test/v1/chat/messages/history")
        XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer access-token")
        XCTAssertEqual(request.value(forHTTPHeaderField: "Content-Type"), "application/json")
        let json = try XCTUnwrap(request.httpBody).jsonObject
        XCTAssertEqual(json["limit"] as? Int, 50)
    }

    func testPostDecodesSuccessfulResponse() async throws {
        let user = UUID(), device = UUID()
        URLProtocolStub.handler = { request in
            XCTAssertEqual(request.url?.path, "/v1/auth/register")
            let payload = #"{"access_token":"a","refresh_token":"r","user_id":"\#(user.uuidString)","device_id":"\#(device.uuidString)"}"#.data(using: .utf8)!
            return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, payload)
        }
        let client = APIClient(baseURL: URL(string: "https://api.tektalk.test")!, session: stubSession())
        let tokens = try await client.register(phone: "+84901234567", name: "TEK", password: "Password123", question: "School?", answer: "TEK")
        XCTAssertEqual(tokens.user_id, user)
        XCTAssertEqual(tokens.device_id, device)
    }

    func testPostRejectsNonSuccessStatus() async {
        URLProtocolStub.handler = { request in
            (HTTPURLResponse(url: request.url!, statusCode: 401, httpVersion: nil, headerFields: nil)!, Data())
        }
        let client = APIClient(baseURL: URL(string: "https://api.tektalk.test")!, session: stubSession())
        do {
            _ = try await client.login(phone: "+84901234567", password: "wrong-pass", deviceId: nil)
            XCTFail("Expected bad server response")
        } catch {
            XCTAssertEqual((error as? URLError)?.code, .badServerResponse)
        }
    }

    private func stubSession() -> URLSession {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [URLProtocolStub.self]
        return URLSession(configuration: configuration)
    }
}

private final class URLProtocolStub: URLProtocol {
    static var handler: ((URLRequest) throws -> (HTTPURLResponse, Data))?
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        do {
            let (response, data) = try XCTUnwrap(Self.handler)(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch { client?.urlProtocol(self, didFailWithError: error) }
    }
    override func stopLoading() {}
}

private extension Data {
    var jsonObject: [String: Any] {
        get throws { try XCTUnwrap(JSONSerialization.jsonObject(with: self) as? [String: Any]) }
    }
}
