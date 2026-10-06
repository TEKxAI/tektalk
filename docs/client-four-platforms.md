# Four-platform native client architecture

TEKtalk uses native UI on all four target operating systems. The shared
boundary is the Rust Core and the versioned HTTPS/realtime contracts, not a
cross-platform widget layer.

| Platform | UI and OS integration | Rust integration |
| --- | --- | --- |
| iOS | SwiftUI + UIKit | static library through the stable C ABI |
| Android | Kotlin + Jetpack Compose | JNI backed by Rust |
| macOS | SwiftUI + AppKit | bundled dynamic library through the C ABI |
| Windows | C++/WinRT + WinUI 3 | native DLL through the C ABI |

The executable MVP covers registration, password login, unknown-device
verification, direct text-message history/send, session identity and logout.
Message identifiers, MTProto established-session state, artifact verification
and capability decisions belong in Rust. Camera, notification, secure storage,
window lifecycle, menus and accessibility stay native.

## Local development

Start the backend with `make local-up`. Android emulators use
`http://10.0.2.2:8080`; iOS simulators, macOS and Windows use
`http://localhost:8080` by default.

- Android: `scripts/build-core-android.sh`, then open `clients/android`.
- iOS: `scripts/build-core-ios.sh`, run XcodeGen, then open the project.
- macOS: `scripts/build-core-macos.sh`, run XcodeGen, then open the project.
- Windows: run `scripts/build-core-windows.ps1`, then build the vcxproj in Visual Studio 2022.

CI builds each client on the matching hosted OS. Release signing and
notarization remain deployment credential concerns and are not bypassed.
