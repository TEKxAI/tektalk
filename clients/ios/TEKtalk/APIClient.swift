import Foundation
import UIKit
final class APIClient {
    let baseURL: URL
    private let session: URLSession
    init(baseURL: URL = URL(string: "http://localhost:8080")!, session: URLSession = .shared) {
        self.baseURL = baseURL
        self.session = session
    }
    func makeRequest<I: Encodable>(_ path: String, body: I, token: String? = nil) throws -> URLRequest {
        var request = URLRequest(url: baseURL.appendingPathComponent(path)); request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type"); request.httpBody = try JSONEncoder().encode(body)
        if let token { request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization") }
        return request
    }
    func post<I: Encodable, O: Decodable>(_ path: String, body: I, token: String? = nil) async throws -> O {
        let request = try makeRequest(path, body: body, token: token)
        let (data,response)=try await session.data(for: request)
        guard let http=response as? HTTPURLResponse,(200..<300).contains(http.statusCode) else { throw URLError(.badServerResponse) }
        return try JSONDecoder().decode(O.self,from:data)
    }
    func login(phone:String,password:String,deviceId:UUID?) async throws -> LoginResponse { try await post("v1/auth/login",body:LoginRequest(phone:phone,password:password,device_id:deviceId,device_name:UIDevice.current.name)) }
    func register(phone:String,name:String,password:String,question:String,answer:String) async throws -> Tokens { try await post("v1/auth/register",body:RegisterRequest(phone:phone,display_name:name,password:password,security_question:question,security_answer:answer,device_name:UIDevice.current.name)) }
    func verify(challenge:UUID,answer:String) async throws -> Tokens { try await post("v1/auth/device/verify",body:VerifyDeviceRequest(challenge_id:challenge,answer:answer)) }
    func history(token:String,conversation:UUID) async throws -> [ChatMessage] { try await post("v1/chat/messages/history",body:HistoryRequest(conversation_id:conversation,before_message_id:nil,limit:50),token:token) }
    func send(token:String,conversation:UUID,recipient:UUID,text:String) async throws -> SendMessageResponse { try await post("v1/chat/messages/send",body:SendMessageRequest(conversation_id:conversation,recipient_id:recipient,client_message_id:UUID(),text:text),token:token) }
}
import UIKit
