import SwiftUI

private func L(_ key: String) -> String { NSLocalizedString(key, comment: "") }

struct ContentView: View {
    @EnvironmentObject private var session: SessionStore
    var body: some View { Group { if let tokens = session.tokens { HostView(tokens: tokens) } else { AuthView() } }.frame(minWidth: 920, minHeight: 620).tint(TekDesign.primary).background(TekDesign.surface) }
}

private struct AuthView: View {
    @EnvironmentObject private var session: SessionStore
    @State private var phone = "+84"
    @State private var password = ""
    @State private var name = ""
    @State private var question = L("auth.default_security_question")
    @State private var answer = ""
    var body: some View {
        HStack(spacing: 0) {
            ZStack { LinearGradient(colors: [TekDesign.primary, TekDesign.primary], startPoint: .topLeading, endPoint: .bottomTrailing); VStack(alignment: .leading, spacing: 16) { Text("TEKtalk").font(.system(size: 52, weight: .bold)); Text(L("auth.hero")).font(.title2); Spacer(); Label(RustCore.shared.isAvailable ? L("core.connected") : L("core.fallback"), systemImage: "checkmark.shield") }.foregroundStyle(TekDesign.onPrimary).padding(52) }.frame(width: 410)
            Form {
                Text(session.mode == .register ? L("auth.create_account") : session.mode == .verify ? session.challengeQuestion : L("auth.sign_in")).font(.largeTitle.bold())
                if session.mode != .verify { TextField("Số điện thoại E.164", text: $phone).accessibilityIdentifier("auth.phone"); if session.mode == .register { TextField("Tên hiển thị", text: $name).accessibilityIdentifier("auth.name") }; SecureField("Mật khẩu", text: $password).accessibilityIdentifier("auth.password") }
                if session.mode == .register { TextField("Câu hỏi bảo mật", text: $question).accessibilityIdentifier("auth.question"); SecureField("Câu trả lời", text: $answer).accessibilityIdentifier("auth.answer") }
                if session.mode == .verify { SecureField("Câu trả lời bảo mật", text: $answer).accessibilityIdentifier("auth.answer") }
                if !session.error.isEmpty { Text(session.error).foregroundStyle(TekDesign.error) }
                Button(session.busy ? L("auth.working") : session.mode == .login ? L("auth.sign_in") : session.mode == .register ? L("auth.sign_up") : L("auth.verify_device")) { Task { await submit() } }.accessibilityIdentifier("auth.submit").buttonStyle(.borderedProminent).controlSize(.large).disabled(session.busy)
                if session.mode != .verify { Button(session.mode == .login ? L("auth.switch_to_register") : L("auth.switch_to_login")) { session.mode = session.mode == .login ? .register : .login; session.error = "" }.accessibilityIdentifier("auth.toggle").buttonStyle(.link) }
            }.formStyle(.grouped).padding(50).frame(maxWidth: .infinity)
        }
    }
    @MainActor private func submit() async {
        if let error = Validation.auth(mode: session.mode, phone: phone, password: password, name: name, question: question, answer: answer) { session.error = error; return }
        session.busy = true; session.error = ""; defer { session.busy = false }
        do {
            switch session.mode {
            case .register: session.tokens = try await session.api.register(phone: phone, name: name, password: password, question: question, answer: answer)
            case .verify: guard let challenge = session.challenge else { throw APIError.server(L("auth.invalid_verification_session")) }; session.tokens = try await session.api.verify(challenge: challenge, answer: answer)
            case .login:
                let response = try await session.api.login(phone: phone, password: password)
                if let tokens = response.tokens { session.tokens = tokens } else { session.challenge = response.challenge_id; session.challengeQuestion = response.question ?? L("auth.verify_device"); answer = ""; session.mode = .verify }
            }
        } catch { session.error = error.localizedDescription }
    }
}

private enum Destination: String, CaseIterable, Identifiable { case message = "Message", ai = "AI", me = "Me"; var id: Self { self }; var icon: String { switch self { case .message: "message.fill"; case .ai: "sparkles"; case .me: "person.crop.circle" } } }

private struct HostView: View {
    let tokens: Tokens
    @State private var selection: Destination = .message
    var body: some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: TekDesign.Spacing.lg) {
                Text("TEKtalk").font(.title2.bold()).padding(.horizontal, TekDesign.Spacing.md)
                HStack(spacing: TekDesign.Spacing.xs) {
                    ForEach(Destination.allCases) { item in
                        Button { selection = item } label: {
                            VStack(spacing: TekDesign.Spacing.xs) {
                                Image(systemName: item.icon).font(.system(size: 18, weight: .semibold))
                                Text(L("tab.\(item.rawValue.lowercased())")).font(.caption2).lineLimit(1)
                            }
                            .frame(maxWidth: .infinity, minHeight: TekDesign.Layout.touchTarget)
                            .foregroundStyle(selection == item ? TekDesign.onPrimaryContainer : TekDesign.muted)
                            .background(selection == item ? TekDesign.primaryContainer : Color.clear, in: RoundedRectangle(cornerRadius: TekDesign.Radius.control))
                        }
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("tab.\(item.rawValue.lowercased())")
                        .accessibilityAddTraits(selection == item ? .isSelected : [])
                    }
                }
                Spacer()
            }
            .padding(TekDesign.Spacing.sm)
            .frame(width: TekDesign.Layout.sidebar)
            .background(TekDesign.surface)
            Divider()
            Group {
                switch selection {
                case .message: ChatView(tokens: tokens)
                case .ai: AIView()
                case .me: MeView(tokens: tokens)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(TekDesign.background)
        }
    }
}

private struct ChatView: View {
    let tokens: Tokens
    @EnvironmentObject private var session: SessionStore
    @State private var conversation = ""
    @State private var recipient = ""
    @State private var draft = ""
    @State private var status = L("message.start_hint")
    @State private var messages: [ChatMessage] = []
    var body: some View { VStack(spacing: 0) { HStack { VStack(alignment: .leading) { Text(L("tab.message")).font(.largeTitle.bold()); Text(status).foregroundStyle(TekDesign.muted) }; Spacer(); Button(L("message.sync")) { Task { await load() } }.accessibilityIdentifier("chat.sync") }.padding(); Divider(); ScrollView { LazyVStack(spacing: 10) { ForEach(messages) { message in HStack { if message.sender_id == tokens.user_id { Spacer() }; Text(message.body).padding(12).foregroundStyle(message.sender_id == tokens.user_id ? TekDesign.onPrimary : TekDesign.text).background(message.sender_id == tokens.user_id ? TekDesign.primary : TekDesign.surfaceVariant, in: RoundedRectangle(cornerRadius: TekDesign.Radius.bubble)); if message.sender_id != tokens.user_id { Spacer() } } } }.padding() }; Divider(); VStack { HStack { TextField(L("message.conversation_uuid"), text: $conversation).accessibilityIdentifier("chat.conversation"); TextField(L("message.recipient_uuid"), text: $recipient).accessibilityIdentifier("chat.recipient") }; HStack { TextField(L("message.compose"), text: $draft).accessibilityIdentifier("chat.draft"); Button(L("message.send")) { Task { await send() } }.accessibilityIdentifier("chat.send").keyboardShortcut(.return, modifiers: .command) } }.padding() }.navigationTitle(L("tab.message")) }
    @MainActor private func load() async { guard let id = UUID(uuidString: conversation) else { status = L("message.invalid_conversation"); return }; do { messages = try await session.api.history(token: tokens.access_token, conversation: id); status = String(format:L("message.synced_count"),Int64(messages.count)) } catch { status = error.localizedDescription } }
    @MainActor private func send() async { if let error = Validation.chat(conversation: conversation, recipient: recipient, text: draft) { status = error; return }; guard let cid = UUID(uuidString: conversation), let rid = UUID(uuidString: recipient) else { return }; let text = draft; draft = ""; do { let response = try await session.api.send(token: tokens.access_token, conversation: cid, recipient: rid, text: text); let unique = Dictionary(uniqueKeysWithValues: (messages + [response.message]).map { ($0.id, $0) }); messages = unique.values.sorted { $0.id < $1.id }; status = String(format:L("message.sent_id"),response.message.id) } catch { status = error.localizedDescription } }
}

private struct AIView: View { var body: some View { ContentUnavailableView("TEK AI", systemImage: "sparkles", description: Text(L("ai.plugin_description"))) } }
private struct MeView: View { @EnvironmentObject private var session: SessionStore; let tokens: Tokens; var body: some View { Form { Section("Core Platform") { LabeledContent(L("profile.user"), value: tokens.user_id.uuidString); LabeledContent(L("profile.device"), value: tokens.device_id.uuidString); LabeledContent(L("profile.rust_core"), value: RustCore.shared.isAvailable ? L("profile.connected") : L("profile.fallback")) }; Button(L("auth.sign_out"), role: .destructive) { session.logout() }.accessibilityIdentifier("auth.logout") }.formStyle(.grouped).navigationTitle(L("tab.me")) } }
