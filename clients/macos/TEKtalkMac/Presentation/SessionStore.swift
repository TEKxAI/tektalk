import Foundation

@MainActor final class SessionStore: ObservableObject {
    @Published var tokens: Tokens?
    @Published var mode: AuthMode = .login
    @Published var challenge: UUID?
    @Published var challengeQuestion = ""
    @Published var error = ""
    @Published var busy = false
    let api = APIClient()

    func logout() { tokens = nil; challenge = nil; mode = .login }
}
