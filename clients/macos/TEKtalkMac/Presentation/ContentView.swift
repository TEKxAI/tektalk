import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var session: SessionStore
    var body: some View { Group { if let tokens = session.tokens { HostView(tokens: tokens) } else { AuthView() } }.frame(minWidth: 920, minHeight: 620) }
}

private struct AuthView: View {
    @EnvironmentObject private var session: SessionStore
    @State private var phone = "+84"
    @State private var password = ""
    @State private var name = ""
    @State private var question = "Tên trường tiểu học của bạn?"
    @State private var answer = ""
    var body: some View {
        HStack(spacing: 0) {
            ZStack { LinearGradient(colors: [.blue, Color(red: 0.22, green: 0.48, blue: 0.96)], startPoint: .topLeading, endPoint: .bottomTrailing); VStack(alignment: .leading, spacing: 16) { Text("TEKtalk").font(.system(size: 52, weight: .bold)); Text("Native messaging for macOS\nPowered by shared Rust Core").font(.title2); Spacer(); Label(RustCore.shared.isAvailable ? "Rust Core connected" : "Rust Core development fallback", systemImage: "checkmark.shield") }.foregroundStyle(.white).padding(52) }.frame(width: 410)
            Form {
                Text(session.mode == .register ? "Tạo tài khoản" : session.mode == .verify ? session.challengeQuestion : "Đăng nhập").font(.largeTitle.bold())
                if session.mode != .verify { TextField("Số điện thoại E.164", text: $phone); if session.mode == .register { TextField("Tên hiển thị", text: $name) }; SecureField("Mật khẩu", text: $password) }
                if session.mode == .register { TextField("Câu hỏi bảo mật", text: $question); SecureField("Câu trả lời", text: $answer) }
                if session.mode == .verify { SecureField("Câu trả lời bảo mật", text: $answer) }
                if !session.error.isEmpty { Text(session.error).foregroundStyle(.red) }
                Button(session.busy ? "Đang xử lý…" : session.mode == .login ? "Đăng nhập" : session.mode == .register ? "Đăng ký" : "Xác minh thiết bị") { Task { await submit() } }.buttonStyle(.borderedProminent).controlSize(.large).disabled(session.busy)
                if session.mode != .verify { Button(session.mode == .login ? "Chưa có tài khoản? Đăng ký" : "Đã có tài khoản? Đăng nhập") { session.mode = session.mode == .login ? .register : .login; session.error = "" }.buttonStyle(.link) }
            }.formStyle(.grouped).padding(50).frame(maxWidth: .infinity)
        }
    }
    @MainActor private func submit() async {
        if let error = Validation.auth(mode: session.mode, phone: phone, password: password, name: name, question: question, answer: answer) { session.error = error; return }
        session.busy = true; session.error = ""; defer { session.busy = false }
        do {
            switch session.mode {
            case .register: session.tokens = try await session.api.register(phone: phone, name: name, password: password, question: question, answer: answer)
            case .verify: guard let challenge = session.challenge else { throw APIError.server("Phiên xác minh không hợp lệ") }; session.tokens = try await session.api.verify(challenge: challenge, answer: answer)
            case .login:
                let response = try await session.api.login(phone: phone, password: password)
                if let tokens = response.tokens { session.tokens = tokens } else { session.challenge = response.challenge_id; session.challengeQuestion = response.question ?? "Xác minh thiết bị"; answer = ""; session.mode = .verify }
            }
        } catch { session.error = error.localizedDescription }
    }
}

private enum Destination: String, CaseIterable, Identifiable { case message = "Message", ai = "AI", me = "Me"; var id: Self { self }; var icon: String { switch self { case .message: "message.fill"; case .ai: "sparkles"; case .me: "person.crop.circle" } } }

private struct HostView: View {
    let tokens: Tokens
    @State private var selection: Destination? = .message
    var body: some View { NavigationSplitView { List(Destination.allCases, selection: $selection) { item in Label(item.rawValue, systemImage: item.icon).tag(item) }.navigationTitle("TEKtalk") } detail: { switch selection ?? .message { case .message: ChatView(tokens: tokens); case .ai: AIView(); case .me: MeView(tokens: tokens) } } }
}

private struct ChatView: View {
    let tokens: Tokens
    @EnvironmentObject private var session: SessionStore
    @State private var conversation = ""
    @State private var recipient = ""
    @State private var draft = ""
    @State private var status = "Nhập conversation và recipient UUID"
    @State private var messages: [ChatMessage] = []
    var body: some View { VStack(spacing: 0) { HStack { VStack(alignment: .leading) { Text("Tin nhắn").font(.largeTitle.bold()); Text(status).foregroundStyle(.secondary) }; Spacer(); Button("Đồng bộ") { Task { await load() } } }.padding(); Divider(); ScrollView { LazyVStack(spacing: 10) { ForEach(messages) { message in HStack { if message.sender_id == tokens.user_id { Spacer() }; Text(message.body).padding(12).foregroundStyle(message.sender_id == tokens.user_id ? .white : .primary).background(message.sender_id == tokens.user_id ? Color.blue : Color.secondary.opacity(0.12), in: RoundedRectangle(cornerRadius: 14)); if message.sender_id != tokens.user_id { Spacer() } } } }.padding() }; Divider(); VStack { HStack { TextField("Conversation UUID", text: $conversation); TextField("Recipient UUID", text: $recipient) }; HStack { TextField("Tin nhắn", text: $draft); Button("Gửi") { Task { await send() } }.keyboardShortcut(.return, modifiers: .command) } }.padding() }.navigationTitle("Message") }
    @MainActor private func load() async { guard let id = UUID(uuidString: conversation) else { status = "Conversation UUID không hợp lệ"; return }; do { messages = try await session.api.history(token: tokens.access_token, conversation: id); status = "Đã đồng bộ \(messages.count) tin" } catch { status = error.localizedDescription } }
    @MainActor private func send() async { if let error = Validation.chat(conversation: conversation, recipient: recipient, text: draft) { status = error; return }; guard let cid = UUID(uuidString: conversation), let rid = UUID(uuidString: recipient) else { return }; let text = draft; draft = ""; do { let response = try await session.api.send(token: tokens.access_token, conversation: cid, recipient: rid, text: text); let unique = Dictionary(uniqueKeysWithValues: (messages + [response.message]).map { ($0.id, $0) }); messages = unique.values.sorted { $0.id < $1.id }; status = "Đã gửi • ID \(response.message.id)" } catch { status = error.localizedDescription } }
}

private struct AIView: View { var body: some View { ContentUnavailableView("TEK AI", systemImage: "sparkles", description: Text("Plugin AI đã có contract; orchestrator sẽ được nối ở milestone tiếp theo.")) } }
private struct MeView: View { @EnvironmentObject private var session: SessionStore; let tokens: Tokens; var body: some View { Form { Section("Core Platform") { LabeledContent("User", value: tokens.user_id.uuidString); LabeledContent("Device", value: tokens.device_id.uuidString); LabeledContent("Rust Core", value: RustCore.shared.isAvailable ? "Connected" : "Fallback") }; Button("Đăng xuất", role: .destructive) { session.logout() } }.formStyle(.grouped).navigationTitle("Me") } }
