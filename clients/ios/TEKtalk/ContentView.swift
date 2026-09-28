import SwiftUI
struct ContentView: View {
    @State private var authenticated=false
    var body: some View { if authenticated { HostTabs() } else { AuthView(authenticated:$authenticated) } }
}

private struct AuthView: View {
    @Binding var authenticated:Bool;@State private var phone="+84";@State private var password="";@State private var status="Đăng nhập bằng mật khẩu"
    var body: some View { NavigationStack { Form { TextField("Số điện thoại",text:$phone).textContentType(.telephoneNumber);SecureField("Mật khẩu",text:$password);Button("Đăng nhập"){Task{do{let r=try await APIClient().login(phone:phone,password:password,deviceId:nil);if r.status=="authenticated"{authenticated=true}else{status="Thiết bị lạ: \(r.question ?? "Xác minh")"}}catch{status=error.localizedDescription}}};Text(status) }.navigationTitle("TEKtalk") } }
}

private struct HostTabs: View {
    var body: some View { TabView { PluginMount(id:"tektalk.message",title:"Message").tabItem{Label("Message",systemImage:"message")};PluginMount(id:"tektalk.ai",title:"AI").tabItem{Label("AI",systemImage:"sparkles")};PluginMount(id:"tektalk.me",title:"Me").tabItem{Label("Me",systemImage:"person")}} }
}

private struct PluginMount: View {
    let id:String;let title:String
    var body: some View { NavigationStack { VStack(spacing:8){Text(title).font(.title);Text("Valdi plugin mount: \(id)").foregroundStyle(.secondary)}.navigationTitle("TEKtalk") } }
}
@main struct TEKtalkApp: App { var body: some Scene { WindowGroup { ContentView() } } }
