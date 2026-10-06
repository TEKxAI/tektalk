import SwiftUI

@main struct TEKtalkMacApp: App {
    @StateObject private var session = SessionStore()
    var body: some Scene { WindowGroup { ContentView().environmentObject(session) }.commands { CommandGroup(replacing: .newItem) { } } }
}
