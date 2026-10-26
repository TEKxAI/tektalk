import SwiftUI
import CoreImage.CIFilterBuiltins

private func L(_ key: String) -> String { NSLocalizedString(key, comment: "") }

struct ContentView: View {
    @EnvironmentObject private var session: SessionStore
    var body: some View { Group { if let tokens = session.tokens { HostView(tokens: tokens) } else { AuthView() } }.frame(minWidth: 920, minHeight: 620).tint(TekDesign.primary).foregroundStyle(TekDesign.text).background(TekDesign.surface) }
}

private struct AuthView: View {
    @EnvironmentObject private var session: SessionStore
    @State private var phone = "+84"
    @State private var password = ""
    @State private var name = ""
    @State private var question = L("auth.default_security_question")
    @State private var answer = ""
    @State private var deviceLink: DeviceLinkStarted?
    var body: some View {
        HStack(spacing: 0) {
            ZStack { LinearGradient(colors: [TekDesign.primary, TekDesign.primary], startPoint: .topLeading, endPoint: .bottomTrailing); VStack(alignment: .leading, spacing: 16) { Text("TEKtalk").font(.system(size: 52, weight: .bold)); Text(L("auth.hero")).font(.title2); Spacer(); Label(RustCore.shared.isAvailable ? L("core.connected") : L("core.fallback"), systemImage: "checkmark.shield") }.foregroundStyle(TekDesign.onPrimary).padding(52) }.frame(width: 410)
            Form {
                Text(session.mode == .register ? L("auth.create_account") : session.mode == .verify ? session.challengeQuestion : L("auth.sign_in")).font(.largeTitle.bold())
                if let link=deviceLink { if let image=qrImage(link.qr_payload){Image(nsImage:image).interpolation(.none).resizable().frame(width:260,height:260).accessibilityIdentifier("auth.qr")};Text("Scan with TEKtalk on your phone, verify this Mac, then approve.").foregroundStyle(TekDesign.muted);Button("Cancel"){deviceLink=nil}.buttonStyle(.bordered) }
                else if session.mode != .verify { TextField(L("auth.phone_e164"), text: $phone).accessibilityIdentifier("auth.phone"); if session.mode == .register { TextField(L("auth.display_name"), text: $name).accessibilityIdentifier("auth.name") }; SecureField(L("auth.password"), text: $password).accessibilityIdentifier("auth.password") }
                if session.mode == .register { TextField(L("auth.security_question"), text: $question).accessibilityIdentifier("auth.question"); SecureField(L("auth.answer"), text: $answer).accessibilityIdentifier("auth.answer") }
                if session.mode == .verify { SecureField(L("auth.security_answer"), text: $answer).accessibilityIdentifier("auth.answer") }
                if !session.error.isEmpty { Text(session.error).foregroundStyle(TekDesign.error) }
                if deviceLink == nil { Button { Task { await submit() } } label: { Text(session.busy ? L("auth.working") : session.mode == .login ? L("auth.sign_in") : session.mode == .register ? L("auth.sign_up") : L("auth.verify_device")).frame(maxWidth: .infinity) }.accessibilityIdentifier("auth.submit").buttonStyle(TekPrimaryButtonStyle()).disabled(session.busy) }
                if session.mode == .login && deviceLink == nil { Button("Sign in with QR") { Task { await startQr() } }.accessibilityIdentifier("auth.qr.start").buttonStyle(.bordered) }
                if session.mode != .verify && deviceLink == nil { Button(session.mode == .login ? L("auth.switch_to_register") : L("auth.switch_to_login")) { session.mode = session.mode == .login ? .register : .login; session.error = "" }.accessibilityIdentifier("auth.toggle").buttonStyle(.link) }
            }.formStyle(.grouped).scrollContentBackground(.hidden).background(TekDesign.surface).padding(50).frame(maxWidth: .infinity)
        }
    }
    private func qrImage(_ payload:String)->NSImage?{let filter=CIFilter.qrCodeGenerator();filter.message=Data(payload.utf8);filter.correctionLevel="M";guard let output=filter.outputImage?.transformed(by:CGAffineTransform(scaleX:10,y:10)) else{return nil};let rep=NSCIImageRep(ciImage:output);let image=NSImage(size:rep.size);image.addRepresentation(rep);return image}
    @MainActor private func startQr() async { session.error="";session.busy=true;defer{session.busy=false};do{let link=try await session.api.startDeviceLink();deviceLink=link;while deviceLink?.link_id==link.link_id{let result=try await session.api.pollDeviceLink(link);if let tokens=result.tokens{deviceLink=nil;session.tokens=tokens;return};try await Task.sleep(for:.seconds(1.5))}}catch{deviceLink=nil;session.error=error.localizedDescription} }
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
    @StateObject private var chat = DesktopChatState()
    var body: some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 0) {
                Text("TEKtalk").font(.title2.bold()).padding(TekDesign.Spacing.xl)
                ScrollView {
                    VStack(alignment: .leading, spacing: TekDesign.Spacing.md) {
                        Text(L("tab.\(selection.rawValue.lowercased())")).font(.title.bold())
                        switch selection {
                        case .message:
                            ForEach(chat.recent) { entry in
                                Button { chat.open(entry) } label: {
                                    VStack(alignment: .leading, spacing: TekDesign.Spacing.xs) {
                                        Text(L("message.direct_conversation")).font(.headline)
                                        Text(entry.preview.isEmpty ? entry.id : entry.preview).font(.caption).lineLimit(2).foregroundStyle(TekDesign.muted)
                                    }.frame(maxWidth: .infinity, alignment: .leading).padding(TekDesign.Spacing.md)
                                        .background(chat.conversation == entry.id ? TekDesign.primaryContainer : TekDesign.surfaceVariant, in: RoundedRectangle(cornerRadius: TekDesign.Radius.control))
                                }.buttonStyle(.plain).accessibilityIdentifier("history.\(entry.id)")
                            }
                            if chat.recent.isEmpty { Text(L("message.start_hint")).font(.callout).foregroundStyle(TekDesign.muted) }
                        case .ai: Text(L("ai.plugin_description")).foregroundStyle(TekDesign.muted)
                        case .me: Text(tokens.user_id.uuidString).font(.caption).foregroundStyle(TekDesign.muted).textSelection(.enabled)
                        }
                    }.padding(.horizontal, TekDesign.Spacing.lg).padding(.bottom, TekDesign.Spacing.lg)
                }.frame(maxWidth: .infinity, maxHeight: .infinity).accessibilityIdentifier("sidebar.content")
                Divider()
                HStack(spacing: TekDesign.Spacing.xs) {
                    ForEach(Destination.allCases) { item in
                        Button { selection = item } label: {
                            VStack(spacing: TekDesign.Spacing.xs) {
                                Image(systemName: item.icon).font(.system(size: 18, weight: .semibold))
                                Text(L("tab.\(item.rawValue.lowercased())")).font(.caption2).lineLimit(1)
                            }
                            .frame(maxWidth: .infinity, minHeight: TekDesign.Layout.touchTarget)
                            .contentShape(Rectangle())
                            .foregroundStyle(selection == item ? TekDesign.onPrimaryContainer : TekDesign.muted)
                            .background(selection == item ? TekDesign.primaryContainer : Color.clear, in: RoundedRectangle(cornerRadius: TekDesign.Radius.control))
                        }
                        .frame(maxWidth: .infinity, minHeight: TekDesign.Layout.touchTarget)
                        .contentShape(Rectangle())
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("tab.\(item.rawValue.lowercased())")
                        .accessibilityAddTraits(selection == item ? .isSelected : [])
                    }
                }.padding(TekDesign.Spacing.sm).accessibilityElement(children: .contain).accessibilityIdentifier("navigation.bottom")
            }
            .frame(width: TekDesign.Layout.sidebar)
            .background(TekDesign.surface)
            Divider()
            Group {
                switch selection {
                case .message: ChatView(tokens: tokens, chat: chat)
                case .ai: AIView()
                case .me: MeView(tokens: tokens)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(TekDesign.background)
        }
    }
}

private struct RecentConversation: Identifiable {
    let id: String
    let recipient: String
    let messages: [ChatMessage]
    let draft: String
    var preview: String { messages.last?.body ?? "" }
}

// Owned by the host so switching tabs never discards history or an unsent draft.
private final class DesktopChatState: ObservableObject {
    @Published var conversation = ""
    @Published var recipient = ""
    @Published var draft = ""
    @Published var status = L("message.start_hint")
    @Published var messages: [ChatMessage] = []
    @Published var recent: [RecentConversation] = []
    func remember() {
        guard UUID(uuidString: conversation) != nil else { return }
        let entry = RecentConversation(id: conversation, recipient: recipient, messages: messages, draft: draft)
        recent.removeAll { $0.id == entry.id }
        recent.insert(entry, at: 0)
    }
    func open(_ entry: RecentConversation) {
        guard entry.id != conversation else { return }
        remember()
        conversation = entry.id; recipient = entry.recipient; messages = entry.messages; draft = entry.draft
        status = L("message.direct_conversation")
    }
}

private struct ChatView: View {
    let tokens: Tokens
    @EnvironmentObject private var session: SessionStore
    @ObservedObject var chat: DesktopChatState
    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading) { Text(L("tab.message")).font(.title.bold()); Text(chat.status).font(.caption).foregroundStyle(TekDesign.muted) }
                Spacer()
                Button(L("message.sync")) { Task { await load() } }.accessibilityIdentifier("chat.sync").buttonStyle(.bordered)
            }.padding().background(TekDesign.surface)
            Divider()
            HStack {
                TextField(L("message.conversation_uuid"), text: $chat.conversation).accessibilityIdentifier("chat.conversation")
                TextField(L("message.recipient_uuid"), text: $chat.recipient).accessibilityIdentifier("chat.recipient")
            }.textFieldStyle(.roundedBorder).padding().background(TekDesign.surface)
            ScrollView {
                LazyVStack(spacing: 10) {
                    ForEach(chat.messages) { message in
                        HStack {
                            if message.sender_id == tokens.user_id { Spacer(minLength: 60) }
                            Text(message.body).padding(12)
                                .foregroundStyle(message.sender_id == tokens.user_id ? TekDesign.onPrimary : TekDesign.text)
                                .background(message.sender_id == tokens.user_id ? TekDesign.primary : TekDesign.surface, in: RoundedRectangle(cornerRadius: TekDesign.Radius.bubble))
                            if message.sender_id != tokens.user_id { Spacer(minLength: 60) }
                        }
                    }
                }.padding()
            }.frame(maxHeight: .infinity).accessibilityIdentifier("chat.messages")
            Divider()
            HStack {
                TextField(L("message.compose"), text: $chat.draft).textFieldStyle(.roundedBorder).accessibilityIdentifier("chat.draft")
                Button(L("message.send")) { Task { await send() } }.accessibilityIdentifier("chat.send")
                    .buttonStyle(TekPrimaryButtonStyle()).keyboardShortcut(.return, modifiers: .command)
                    .disabled(chat.draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }.padding().background(TekDesign.surface)
        }.onDisappear { chat.remember() }
    }
    @MainActor private func load() async {
        guard let id = UUID(uuidString: chat.conversation) else { chat.status = L("message.invalid_conversation"); return }
        do {
            let rows = try await session.api.history(token: tokens.access_token, conversation: id)
            guard UUID(uuidString: chat.conversation) == id else { return }
            chat.messages = rows; chat.status = String(format: L("message.synced_count"), Int64(rows.count)); chat.remember()
        } catch { chat.status = error.localizedDescription }
    }
    @MainActor private func send() async {
        if let error = Validation.chat(conversation: chat.conversation, recipient: chat.recipient, text: chat.draft) { chat.status = error; return }
        guard let cid = UUID(uuidString: chat.conversation), let rid = UUID(uuidString: chat.recipient) else { return }
        let text = chat.draft; chat.draft = ""
        do {
            let response = try await session.api.send(token: tokens.access_token, conversation: cid, recipient: rid, text: text)
            guard UUID(uuidString: chat.conversation) == cid else { return }
            var unique = Dictionary(chat.messages.map { ($0.id, $0) }, uniquingKeysWith: { _, last in last })
            unique[response.message.id] = response.message
            chat.messages = unique.values.sorted { $0.id < $1.id }
            chat.status = String(format: L("message.sent_id"), response.message.id); chat.remember()
        } catch { chat.draft = text; chat.status = error.localizedDescription }
    }
}

private struct AIView: View { var body: some View { ContentUnavailableView("TEK AI", systemImage: "sparkles", description: Text(L("ai.plugin_description"))) } }
private struct MeView: View { @EnvironmentObject private var session: SessionStore; let tokens: Tokens; var body: some View { Form { Section("Core Platform") { LabeledContent(L("profile.user"), value: tokens.user_id.uuidString); LabeledContent(L("profile.device"), value: tokens.device_id.uuidString); LabeledContent(L("profile.rust_core"), value: RustCore.shared.isAvailable ? L("profile.connected") : L("profile.fallback")) }; Button(L("auth.sign_out"), role: .destructive) { session.logout() }.accessibilityIdentifier("auth.logout") }.formStyle(.grouped).navigationTitle(L("tab.me")) } }
