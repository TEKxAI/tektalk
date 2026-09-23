import SwiftUI
struct ContentView: View {
    @State private var phone="+84"; @State private var password=""; @State private var status="Đăng nhập bằng mật khẩu"
    var body: some View { NavigationStack { Form { TextField("Số điện thoại",text:$phone).textContentType(.telephoneNumber);SecureField("Mật khẩu",text:$password);Button("Đăng nhập"){Task{do{let r=try await APIClient().login(phone:phone,password:password,deviceId:nil);status=r.status=="authenticated" ? "Đăng nhập thành công" : "Thiết bị lạ: \(r.question ?? "Xác minh")"}catch{status=error.localizedDescription}}};Text(status) }.navigationTitle("ZChat") } }
}
@main struct ZChatApp: App { var body: some Scene { WindowGroup { ContentView() } } }
