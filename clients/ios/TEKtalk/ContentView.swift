import SwiftUI

struct ContentView: View {
    @State private var tokens: Tokens?
    var body: some View { Group { if let tokens { HostTabs(tokens: tokens) { self.tokens=nil } } else { AuthView { tokens=$0 } } }.tint(Color(red:0.09,green:0.41,blue:0.88)) }
}

private struct AuthView: View {
    enum Mode { case login, register, verify }
    let authenticated:(Tokens)->Void
    @State private var mode=Mode.login;@State private var phone="+84";@State private var password="";@State private var name="";@State private var question="Tên trường tiểu học của bạn?";@State private var answer="";@State private var challenge:UUID?;@State private var serverQuestion="";@State private var error="";@State private var busy=false
    var body: some View { NavigationStack { ScrollView { VStack(alignment:.leading,spacing:14){
        Spacer(minLength:60);Text("TEKtalk").font(.system(size:42,weight:.bold));Text(mode == .verify ? serverQuestion : mode == .register ? "Tạo tài khoản Core Platform" : "Kết nối an toàn, trò chuyện tức thời").foregroundStyle(.secondary)
        if mode != .verify { TextField("Số điện thoại E.164",text:$phone).textContentType(.telephoneNumber).textFieldStyle(.roundedBorder);if mode == .register { TextField("Tên hiển thị",text:$name).textFieldStyle(.roundedBorder) };SecureField("Mật khẩu",text:$password).textFieldStyle(.roundedBorder);if mode == .register { TextField("Câu hỏi bảo mật",text:$question).textFieldStyle(.roundedBorder);SecureField("Câu trả lời",text:$answer).textFieldStyle(.roundedBorder) } } else { SecureField("Câu trả lời bảo mật",text:$answer).textFieldStyle(.roundedBorder) }
        if !error.isEmpty { Text(error).font(.footnote).foregroundStyle(.red) }
        Button(busy ? "Đang xử lý…" : mode == .login ? "Đăng nhập" : mode == .register ? "Đăng ký" : "Xác minh thiết bị"){ Task { await submit() } }.buttonStyle(.borderedProminent).controlSize(.large).frame(maxWidth:.infinity).disabled(busy)
        if mode != .verify { Button(mode == .login ? "Chưa có tài khoản? Đăng ký" : "Đã có tài khoản? Đăng nhập"){mode = mode == .login ? .register : .login;error=""}.frame(maxWidth:.infinity) }
    }.padding(28) } } }
    @MainActor private func submit() async { busy=true;error="";defer{busy=false};do{let api=APIClient();switch mode{case .register:authenticated(try await api.register(phone:phone,name:name,password:password,question:question,answer:answer));case .verify:authenticated(try await api.verify(challenge:challenge!,answer:answer));case .login:let response=try await api.login(phone:phone,password:password,deviceId:nil);if let tokens=response.tokens{authenticated(tokens)}else{challenge=response.challenge_id;serverQuestion=response.question ?? "Xác minh thiết bị";answer="";mode = .verify}}}catch{self.error=error.localizedDescription}}
}

private struct HostTabs: View {
    let tokens:Tokens;let logout:()->Void
    var body: some View { TabView { MessageHome(tokens:tokens).tabItem{Label("Message",systemImage:"message.fill")};Placeholder(title:"AI",detail:"Trợ lý AI sẽ được phân phối như một plugin đã ký.").tabItem{Label("AI",systemImage:"sparkles")};ProfileView(tokens:tokens,logout:logout).tabItem{Label("Me",systemImage:"person.fill")}} }
}

private struct MessageHome: View {
    let tokens:Tokens
    var body: some View { NavigationStack { List { Section("OTT Platform") { NavigationLink { ChatRoom(tokens:tokens) } label: { HStack(spacing:14){Text("T").font(.headline).foregroundStyle(.white).frame(width:46,height:46).background(.blue.gradient,in:RoundedRectangle(cornerRadius:16));VStack(alignment:.leading){Text("Cuộc trò chuyện trực tiếp").fontWeight(.semibold);Text("Text messaging qua TEKtalk API").font(.caption).foregroundStyle(.secondary)}} } } }.navigationTitle("Tin nhắn").toolbar{ToolbarItem(placement:.topBarTrailing){NavigationLink(destination:ChatRoom(tokens:tokens)){Image(systemName:"square.and.pencil")}}} } }
}

private struct ChatRoom: View {
    let tokens:Tokens
    @State private var conversation="";@State private var recipient="";@State private var draft="";@State private var messages:[ChatMessage]=[];@State private var status="Nhập hai UUID để bắt đầu"
    var body: some View { VStack(spacing:0){Form{TextField("Conversation UUID",text:$conversation).textInputAutocapitalization(.never);TextField("Recipient UUID",text:$recipient).textInputAutocapitalization(.never);Button("Đồng bộ lịch sử"){Task{await load()}};Text(status).font(.caption).foregroundStyle(.secondary)}.frame(height:190);ScrollView{LazyVStack(spacing:6){ForEach(messages){message in HStack{if message.sender_id == tokens.user_id {Spacer()};Text(message.body).padding(.horizontal,14).padding(.vertical,10).foregroundStyle(message.sender_id == tokens.user_id ? .white : .primary).background(message.sender_id == tokens.user_id ? Color.blue : Color(.secondarySystemBackground),in:RoundedRectangle(cornerRadius:18));if message.sender_id != tokens.user_id {Spacer()}}}}.padding()};HStack{TextField("Tin nhắn",text:$draft).textFieldStyle(.roundedBorder);Button("Gửi"){Task{await send()}}.disabled(draft.isEmpty || UUID(uuidString:conversation)==nil || UUID(uuidString:recipient)==nil)}.padding()}.navigationTitle("TEKtalk Chat").navigationBarTitleDisplayMode(.inline).toolbar{ToolbarItem(placement:.topBarTrailing){Button{status="Call cần signaling/SFU production"}label:{Image(systemName:"phone.fill")}}} }
    @MainActor private func load() async {guard let id=UUID(uuidString:conversation)else{status="Conversation UUID không hợp lệ";return};do{messages=try await APIClient().history(token:tokens.access_token,conversation:id);status="Đã đồng bộ \(messages.count) tin"}catch{status=error.localizedDescription}}
    @MainActor private func send() async {guard let conversation=UUID(uuidString:conversation),let recipient=UUID(uuidString:recipient)else{return};let text=draft;draft="";do{let response=try await APIClient().send(token:tokens.access_token,conversation:conversation,recipient:recipient,text:text);messages.append(response.message)}catch{status=error.localizedDescription}}
}

private struct Placeholder: View {let title:String;let detail:String;var body:some View{NavigationStack{VStack(spacing:14){Image(systemName:"sparkles").font(.system(size:42)).foregroundStyle(.blue);Text(title).font(.title2.bold());Text(detail).multilineTextAlignment(.center).foregroundStyle(.secondary)}.padding(28).navigationTitle(title)}}}
private struct ProfileView: View {let tokens:Tokens;let logout:()->Void;var body:some View{NavigationStack{Form{Section("Core Platform"){LabeledContent("User",value:tokens.user_id.uuidString);LabeledContent("Device",value:tokens.device_id.uuidString)};Section{Button("Đăng xuất",role:.destructive,action:logout)}}.navigationTitle("Tôi")}}}

@main struct TEKtalkApp: App { var body: some Scene { WindowGroup { ContentView() } } }
