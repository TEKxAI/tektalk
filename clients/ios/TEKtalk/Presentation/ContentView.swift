import SwiftUI

private func L(_ key: String) -> String { NSLocalizedString(key, comment: "") }

struct ContentView: View {
    @State private var tokens: Tokens?
    var body: some View { Group { if let tokens { HostTabs(tokens: tokens) { self.tokens=nil } } else { AuthView { tokens=$0 } } }.tint(Color(red:0.09,green:0.41,blue:0.88)) }
}

private struct AuthView: View {
    let authenticated:(Tokens)->Void
    @State private var mode=AuthMode.login;@State private var phone="+84";@State private var password="";@State private var name="";@State private var question=L("auth.default_security_question");@State private var answer="";@State private var challenge:UUID?;@State private var serverQuestion="";@State private var error="";@State private var busy=false
    var body: some View { NavigationStack { ScrollView { VStack(alignment:.leading,spacing:14){
        Spacer(minLength:60);Text("TEKtalk").font(.system(size:42,weight:.bold));Text(mode == .verify ? serverQuestion : mode == .register ? L("auth.create_core_account") : L("auth.tagline")).foregroundStyle(.secondary)
        if mode != .verify { TextField(L("auth.phone_e164"),text:$phone).accessibilityIdentifier("auth.phone").textContentType(.telephoneNumber).textFieldStyle(.roundedBorder);if mode == .register { TextField(L("auth.display_name"),text:$name).accessibilityIdentifier("auth.name").textFieldStyle(.roundedBorder) };SecureField(L("auth.password"),text:$password).accessibilityIdentifier("auth.password").textFieldStyle(.roundedBorder);if mode == .register { TextField(L("auth.security_question"),text:$question).accessibilityIdentifier("auth.question").textFieldStyle(.roundedBorder);SecureField(L("auth.answer"),text:$answer).accessibilityIdentifier("auth.answer").textFieldStyle(.roundedBorder) } } else { SecureField(L("auth.security_answer"),text:$answer).accessibilityIdentifier("auth.answer").textFieldStyle(.roundedBorder) }
        if !error.isEmpty { Text(error).font(.footnote).foregroundStyle(.red) }
        Button(busy ? L("auth.working") : mode == .login ? L("auth.sign_in") : mode == .register ? L("auth.sign_up") : L("auth.verify_device")){ Task { await submit() } }.accessibilityIdentifier("auth.submit").buttonStyle(.borderedProminent).controlSize(.large).frame(maxWidth:.infinity).disabled(busy)
        if mode != .verify { Button(mode == .login ? L("auth.switch_to_register") : L("auth.switch_to_login")){mode = mode == .login ? .register : .login;error=""}.accessibilityIdentifier("auth.toggle").frame(maxWidth:.infinity) }
    }.padding(28) } } }
    @MainActor private func submit() async { if let validation=InputValidator.auth(mode:mode,phone:phone,password:password,displayName:name,securityQuestion:question,securityAnswer:answer){error=validation;return};busy=true;error="";defer{busy=false};do{let api=APIClient();switch mode{case .register:authenticated(try await api.register(phone:phone,name:name,password:password,question:question,answer:answer));case .verify:guard let challenge else{error=L("auth.invalid_verification_session");return};authenticated(try await api.verify(challenge:challenge,answer:answer));case .login:let response=try await api.login(phone:phone,password:password,deviceId:nil);if let tokens=response.tokens{authenticated(tokens)}else{challenge=response.challenge_id;serverQuestion=response.question ?? L("auth.verify_device");answer="";mode=AuthFlow.nextMode(for:response)}}}catch{self.error=error.localizedDescription}}
}

private struct HostTabs: View {
    let tokens:Tokens;let logout:()->Void
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var selection = 0
    @ViewBuilder private var selectedContent: some View { switch selection { case 1: Placeholder(title:"AI",detail:String(localized:"ai_plugin_description")); case 2: ProfileView(tokens:tokens,logout:logout); default: MessageHome(tokens:tokens) } }
    var body: some View { if horizontalSizeClass == .regular { NavigationSplitView { List { Button{selection=0}label:{Label("Message",systemImage:"message.fill")};Button{selection=1}label:{Label("AI",systemImage:"sparkles")};Button{selection=2}label:{Label("Me",systemImage:"person.fill")} }.buttonStyle(.plain).navigationTitle("TEKtalk") } detail: { selectedContent } } else { TabView(selection:$selection) { MessageHome(tokens:tokens).tabItem{Label("Message",systemImage:"message.fill")}.tag(0);Placeholder(title:"AI",detail:String(localized:"ai_plugin_description")).tabItem{Label("AI",systemImage:"sparkles")}.tag(1);ProfileView(tokens:tokens,logout:logout).tabItem{Label("Me",systemImage:"person.fill")}.tag(2) } } }
}

private struct MessageHome: View {
    let tokens:Tokens
    var body: some View { NavigationStack { List { Section("OTT Platform") { NavigationLink { ChatRoom(tokens:tokens) } label: { HStack(spacing:14){Text("T").font(.headline).foregroundStyle(.white).frame(width:46,height:46).background(.blue.gradient,in:RoundedRectangle(cornerRadius:16));VStack(alignment:.leading){Text("Cuộc trò chuyện trực tiếp").fontWeight(.semibold);Text("Text messaging qua TEKtalk API").font(.caption).foregroundStyle(.secondary)}} }.accessibilityIdentifier("chat.open") } }.navigationTitle("Tin nhắn").toolbar{ToolbarItem(placement:.topBarTrailing){NavigationLink(destination:ChatRoom(tokens:tokens)){Image(systemName:"square.and.pencil")}}} } }
}

private struct ChatRoom: View {
    let tokens:Tokens
    @FocusState private var composeFocused: Bool
    @State private var conversation="";@State private var recipient="";@State private var draft="";@State private var messages:[ChatMessage]=[];@State private var status=L("message.start_hint")
    var body: some View { VStack(spacing:0){Form{TextField(L("message.conversation_uuid"),text:$conversation).accessibilityIdentifier("chat.conversation").textInputAutocapitalization(.never);TextField(L("message.recipient_uuid"),text:$recipient).accessibilityIdentifier("chat.recipient").textInputAutocapitalization(.never);Button(L("message.sync_history")){Task{await load()}}.accessibilityIdentifier("chat.sync");Text(status).font(.caption).foregroundStyle(.secondary)}.frame(height:190);ScrollView{LazyVStack(spacing:6){ForEach(messages){message in HStack{if message.sender_id == tokens.user_id {Spacer()};Text(message.body).padding(.horizontal,14).padding(.vertical,10).foregroundStyle(message.sender_id == tokens.user_id ? .white : .primary).background(message.sender_id == tokens.user_id ? Color.blue : Color(.secondarySystemBackground),in:RoundedRectangle(cornerRadius:18));if message.sender_id != tokens.user_id {Spacer()}}}}.padding()};HStack{TextField(L("message.compose"),text:$draft).accessibilityIdentifier("chat.draft").focused($composeFocused).textFieldStyle(.roundedBorder);Button(L("message.send")){Task{await send()}}.accessibilityIdentifier("chat.send").disabled(draft.isEmpty || UUID(uuidString:conversation)==nil || UUID(uuidString:recipient)==nil)}.padding()}.navigationTitle("TEKtalk Chat").navigationBarTitleDisplayMode(.inline).toolbar{ToolbarItem(placement:.topBarTrailing){Button{status=L("call.not_available")}label:{Image(systemName:"phone.fill")}}} }
    @MainActor private func load() async {guard let id=UUID(uuidString:conversation)else{status=L("message.invalid_conversation");return};do{messages=try await APIClient().history(token:tokens.access_token,conversation:id);status=String(format:L("message.synced_count"),Int64(messages.count))}catch{status=error.localizedDescription}}
    @MainActor private func send() async {composeFocused=false;if let validation=InputValidator.chat(conversation:conversation,recipient:recipient,text:draft){status=validation;return};guard let conversation=UUID(uuidString:conversation),let recipient=UUID(uuidString:recipient)else{return};let text=draft.trimmingCharacters(in:.whitespacesAndNewlines);draft="";do{let response=try await APIClient().send(token:tokens.access_token,conversation:conversation,recipient:recipient,text:text);messages.append(response.message)}catch{status=error.localizedDescription}}
}

private struct Placeholder: View {let title:String;let detail:String;var body:some View{NavigationStack{VStack(spacing:14){Image(systemName:"sparkles").font(.system(size:42)).foregroundStyle(.blue);Text(title).font(.title2.bold());Text(detail).multilineTextAlignment(.center).foregroundStyle(.secondary)}.padding(28).navigationTitle(title)}}}
private struct ProfileView: View {let tokens:Tokens;let logout:()->Void;var body:some View{NavigationStack{Form{Section("Core Platform"){LabeledContent(L("profile.user"),value:tokens.user_id.uuidString);LabeledContent(L("profile.device"),value:tokens.device_id.uuidString);LabeledContent(L("profile.rust_core"),value:NativeCore.shared.isAvailable ? L("profile.connected") : L("profile.development_fallback"))};Section{Button(L("auth.sign_out"),role:.destructive,action:logout).accessibilityIdentifier("auth.logout")}}.navigationTitle(L("tab.me"))}}}

@main struct TEKtalkApp: App { var body: some Scene { WindowGroup { ContentView() } } }
