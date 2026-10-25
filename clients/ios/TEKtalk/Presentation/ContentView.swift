import SwiftUI

private func L(_ key: String) -> String { NSLocalizedString(key, comment: "") }

struct ContentView: View {
    @State private var tokens: Tokens?
    var body: some View { Group { if let tokens { HostTabs(tokens: tokens) { self.tokens=nil } } else { AuthView { tokens=$0 } } }.tint(TekDesign.primary).foregroundStyle(TekDesign.text).background(TekDesign.surface) }
}

private struct AuthView: View {
    let authenticated:(Tokens)->Void
    @State private var mode=AuthMode.login;@State private var phone="+84";@State private var password="";@State private var name="";@State private var question=L("auth.default_security_question");@State private var answer="";@State private var challenge:UUID?;@State private var serverQuestion="";@State private var error="";@State private var busy=false
    var body: some View { NavigationStack { ScrollView { VStack(alignment:.leading,spacing:14){
        Spacer(minLength:60);Text("TEKtalk").font(.system(size:42,weight:.bold));Text(mode == .verify ? serverQuestion : mode == .register ? L("auth.create_core_account") : L("auth.tagline")).foregroundStyle(TekDesign.muted)
        if mode != .verify { TextField(L("auth.phone_e164"),text:$phone).accessibilityIdentifier("auth.phone").textContentType(.telephoneNumber).textFieldStyle(.roundedBorder);if mode == .register { TextField(L("auth.display_name"),text:$name).accessibilityIdentifier("auth.name").textFieldStyle(.roundedBorder) };SecureField(L("auth.password"),text:$password).accessibilityIdentifier("auth.password").textFieldStyle(.roundedBorder);if mode == .register { TextField(L("auth.security_question"),text:$question).accessibilityIdentifier("auth.question").textFieldStyle(.roundedBorder);SecureField(L("auth.answer"),text:$answer).accessibilityIdentifier("auth.answer").textFieldStyle(.roundedBorder) } } else { SecureField(L("auth.security_answer"),text:$answer).accessibilityIdentifier("auth.answer").textFieldStyle(.roundedBorder) }
        if !error.isEmpty { Text(error).font(.footnote).foregroundStyle(TekDesign.error) }
        Button { Task { await submit() } } label: { Text(busy ? L("auth.working") : mode == .login ? L("auth.sign_in") : mode == .register ? L("auth.sign_up") : L("auth.verify_device")).frame(maxWidth: .infinity) }.accessibilityIdentifier("auth.submit").buttonStyle(TekPrimaryButtonStyle()).disabled(busy)
        if mode != .verify { Button(mode == .login ? L("auth.switch_to_register") : L("auth.switch_to_login")){mode = mode == .login ? .register : .login;error=""}.accessibilityIdentifier("auth.toggle").frame(maxWidth:.infinity) }
    }.padding(28) } } }
    @MainActor private func submit() async { if let validation=InputValidator.auth(mode:mode,phone:phone,password:password,displayName:name,securityQuestion:question,securityAnswer:answer){error=validation;return};busy=true;error="";defer{busy=false};do{let api=APIClient();switch mode{case .register:authenticated(try await api.register(phone:phone,name:name,password:password,question:question,answer:answer));case .verify:guard let challenge else{error=L("auth.invalid_verification_session");return};authenticated(try await api.verify(challenge:challenge,answer:answer));case .login:let response=try await api.login(phone:phone,password:password,deviceId:nil);if let tokens=response.tokens{authenticated(tokens)}else{challenge=response.challenge_id;serverQuestion=response.question ?? L("auth.verify_device");answer="";mode=AuthFlow.nextMode(for:response)}}}catch{self.error=error.localizedDescription}}
}

private enum HostDestination: Int, CaseIterable, Identifiable {
    case message, ai, me
    var id: Self { self }
    var title: String { L(self == .message ? "tab.message" : self == .ai ? "tab.ai" : "tab.me") }
    var symbol: String { self == .message ? "message.fill" : self == .ai ? "sparkles" : "person.crop.circle" }
}

private struct HostTabs: View {
    let tokens: Tokens
    let logout: () -> Void
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var selection: HostDestination = .message
    @ViewBuilder private var selectedContent: some View {
        switch selection {
        case .message: MessageHome(tokens: tokens)
        case .ai: Placeholder(title: L("tab.ai"), detail: String(localized: "ai_plugin_description"))
        case .me: ProfileView(tokens: tokens, logout: logout)
        }
    }
    var body: some View {
        if horizontalSizeClass == .regular {
            NavigationSplitView {
                List(HostDestination.allCases) { item in
                    Button { selection = item } label: {
                        Label(item.title, systemImage: item.symbol)
                            .foregroundStyle(selection == item ? TekDesign.primary : TekDesign.text)
                            .frame(minHeight: TekDesign.Layout.touchTarget)
                    }
                    .accessibilityIdentifier("tab.\(item == .message ? "message" : item == .ai ? "ai" : "me")")
                    .listRowBackground(selection == item ? TekDesign.primaryContainer : TekDesign.surface)
                }
                .navigationTitle("TEKtalk")
                .navigationSplitViewColumnWidth(min: 200, ideal: TekDesign.Layout.sidebar, max: 280)
            } detail: { selectedContent }
        } else {
            TabView(selection: $selection) {
                MessageHome(tokens: tokens)
                    .tabItem { Label(L("tab.message"), systemImage: "message.fill") }.tag(HostDestination.message)
                Placeholder(title: L("tab.ai"), detail: String(localized: "ai_plugin_description"))
                    .tabItem { Label(L("tab.ai"), systemImage: "sparkles") }.tag(HostDestination.ai)
                ProfileView(tokens: tokens, logout: logout)
                    .tabItem { Label(L("tab.me"), systemImage: "person.crop.circle") }.tag(HostDestination.me)
            }
        }
    }
}

private struct MessageHome: View {
    let tokens:Tokens
    var body: some View { NavigationStack { List { Section(L("message.platform")) { NavigationLink { ChatRoom(tokens:tokens) } label: { HStack(spacing:14){Text("T").font(.headline).foregroundStyle(TekDesign.onPrimary).frame(width:46,height:46).background(TekDesign.primary.gradient,in:RoundedRectangle(cornerRadius:16));VStack(alignment:.leading){Text(L("message.direct_conversation")).fontWeight(.semibold);Text(L("message.start_hint")).font(.caption).foregroundStyle(TekDesign.muted)}} }.accessibilityIdentifier("chat.open") } }.navigationTitle(L("tab.message")).toolbar{ToolbarItem(placement:.topBarTrailing){NavigationLink(destination:ChatRoom(tokens:tokens)){Image(systemName:"square.and.pencil")}}} } }
}

private struct ChatRoom: View {
    let tokens:Tokens
    private enum Field: Hashable { case conversation, recipient, members, draft }
    @FocusState private var focusedField: Field?
    @State private var conversation="";@State private var recipient="";@State private var members="";@State private var draft="";@State private var messages:[ChatMessage]=[];@State private var status=L("message.start_hint");@State private var secret=false;@State private var group=false;@State private var secretService:SecretChatService?
    var body: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 10) {
                TextField(L("message.conversation_uuid"), text: $conversation)
                    .accessibilityIdentifier("chat.conversation").focused($focusedField, equals: .conversation)
                    .textInputAutocapitalization(.never).autocorrectionDisabled()
                TextField(L("message.recipient_uuid"), text: $recipient)
                    .accessibilityIdentifier("chat.recipient").focused($focusedField, equals: .recipient)
                    .textInputAutocapitalization(.never).autocorrectionDisabled()
                Toggle("Secret Chat",isOn:$secret).accessibilityIdentifier("chat.secret")
                if secret { Toggle("Group",isOn:$group).accessibilityIdentifier("chat.secret.group");if group{TextField("Member UUIDs, separated by commas",text:$members).accessibilityIdentifier("chat.secret.members").textInputAutocapitalization(.never).autocorrectionDisabled().focused($focusedField,equals:.members)};Button("Enable end-to-end encryption"){Task{await enableSecret()}}.accessibilityIdentifier("chat.secret.enable") }
                Button(L("message.sync_history")) { focusedField=nil; Task { await load() } }
                    .accessibilityIdentifier("chat.sync")
                Text(status).accessibilityIdentifier("chat.status").font(.caption).foregroundStyle(TekDesign.muted)
            }.textFieldStyle(.roundedBorder).padding().background(TekDesign.surfaceVariant)
            ScrollView {
                LazyVStack(spacing: 6) {
                    ForEach(messages) { message in
                        HStack {
                            if message.sender_id == tokens.user_id { Spacer() }
                            Text(message.body).padding(.horizontal,14).padding(.vertical,10)
                                .foregroundStyle(message.sender_id == tokens.user_id ? TekDesign.onPrimary : TekDesign.text)
                                .background(message.sender_id == tokens.user_id ? TekDesign.primary : TekDesign.surfaceVariant,in:RoundedRectangle(cornerRadius:TekDesign.Radius.bubble))
                            if message.sender_id != tokens.user_id { Spacer() }
                        }
                    }
                }.padding()
            }.scrollDismissesKeyboard(.interactively)
        }
        .safeAreaInset(edge: .bottom) {
            HStack {
                TextField(L("message.compose"), text: $draft)
                    .accessibilityIdentifier("chat.draft").focused($focusedField, equals: .draft)
                    .textFieldStyle(.roundedBorder)
                Button(L("message.send")) { Task { await send() } }
                    .accessibilityIdentifier("chat.send")
                    .buttonStyle(TekPrimaryButtonStyle())
                    .disabled(draft.isEmpty || UUID(uuidString:conversation)==nil || UUID(uuidString:recipient)==nil)
            }.padding().background(.bar)
        }
        .navigationTitle("TEKtalk Chat").navigationBarTitleDisplayMode(.inline)
        .task{do{if secretService==nil{secretService=try SecretChatService(tokens:tokens)};try await secretService?.registerDevice()}catch{status=error.localizedDescription}}
        .onChange(of: conversation){_ in messages=[]}
        .toolbar { chatToolbar }
    }
    @ToolbarContentBuilder private var chatToolbar: some ToolbarContent {
            ToolbarItem(placement: .topBarTrailing) {
                Button { status=L("call.not_available") } label: { Image(systemName:"phone.fill") }
            }
            ToolbarItemGroup(placement: .keyboard) {
                Spacer()
                Button(L("common.done")) { focusedField=nil }.accessibilityIdentifier("chat.keyboard.done")
            }
    }
    @MainActor private func enableSecret()async{guard let id=UUID(uuidString:conversation),let peer=UUID(uuidString:recipient),let service=secretService else{status=L("message.invalid_conversation");return};let ids=group ? members.split(separator:",").compactMap{UUID(uuidString:String($0).trimmingCharacters(in:.whitespaces))}:[peer];do{try await service.enable(conversation:id,kind:group ? "group":"one_to_one",members:ids);status="Secret Chat enabled • hybrid post-quantum suite"}catch{status=error.localizedDescription}}
    @MainActor private func load() async {guard let id=UUID(uuidString:conversation)else{status=L("message.invalid_conversation");return};do{if secret{guard let service=secretService else{throw URLError(.cannotLoadFromNetwork)};messages=try await service.history(conversation:id)}else{messages=try await APIClient().history(token:tokens.access_token,conversation:id)};status=String(format:L("message.synced_count"),Int64(messages.count))}catch{status=error.localizedDescription}}
    @MainActor private func send() async {focusedField=nil;if let validation=InputValidator.chat(conversation:conversation,recipient:recipient,text:draft){status=validation;return};guard let conversation=UUID(uuidString:conversation),let recipient=UUID(uuidString:recipient)else{return};let text=draft.trimmingCharacters(in:.whitespacesAndNewlines);draft="";do{if secret{guard let service=secretService else{throw URLError(.cannotLoadFromNetwork)};messages.append(try await service.send(conversation:conversation,text:text))}else{let response=try await APIClient().send(token:tokens.access_token,conversation:conversation,recipient:recipient,text:text);messages.append(response.message)}}catch{status=error.localizedDescription}}
}

private struct Placeholder: View {let title:String;let detail:String;var body:some View{NavigationStack{VStack(spacing:14){Image(systemName:"sparkles").font(.system(size:42)).foregroundStyle(TekDesign.primary);Text(title).font(.title2.bold());Text(detail).multilineTextAlignment(.center).foregroundStyle(TekDesign.muted)}.padding(28).navigationTitle(title)}}}
private struct ProfileView: View {let tokens:Tokens;let logout:()->Void;var body:some View{NavigationStack{Form{Section("Core Platform"){LabeledContent(L("profile.user"),value:tokens.user_id.uuidString);LabeledContent(L("profile.device"),value:tokens.device_id.uuidString);LabeledContent(L("profile.rust_core"),value:NativeCore.shared.isAvailable ? L("profile.connected") : L("profile.development_fallback"))};Section{Button(L("auth.sign_out"),role:.destructive,action:logout).accessibilityIdentifier("auth.logout")}}.navigationTitle(L("tab.me"))}}}

@main struct TEKtalkApp: App { var body: some Scene { WindowGroup { ContentView() } } }
