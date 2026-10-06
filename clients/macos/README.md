# TEKtalk for macOS

Native SwiftUI/AppKit desktop client. It talks to the same HTTPS API as the
mobile clients and loads the shared Rust Core through its stable C ABI.

```bash
./scripts/build-core-macos.sh
cd clients/macos
xcodegen generate
open TEKtalkMac.xcodeproj
```

Set `TEKTALK_API_BASE_URL` in the scheme environment when the backend is not at
`http://localhost:8080`. Copy the generated dylib into the app's Frameworks
directory for release packaging; development gracefully reports fallback mode.
