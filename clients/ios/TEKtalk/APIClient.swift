import Foundation
final class APIClient {
    let baseURL: URL
    init(baseURL: URL = URL(string: "http://localhost:8080")!) { self.baseURL = baseURL }
    func post<I: Encodable, O: Decodable>(_ path: String, body: I) async throws -> O {
        var request = URLRequest(url: baseURL.appendingPathComponent(path)); request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type"); request.httpBody = try JSONEncoder().encode(body)
        let (data,response)=try await URLSession.shared.data(for: request)
        guard let http=response as? HTTPURLResponse,(200..<300).contains(http.statusCode) else { throw URLError(.badServerResponse) }
        return try JSONDecoder().decode(O.self,from:data)
    }
    func login(phone:String,password:String,deviceId:UUID?) async throws -> LoginResponse { try await post("v1/auth/login",body:LoginRequest(phone:phone,password:password,device_id:deviceId,device_name:UIDevice.current.name)) }
}
import UIKit
