# TEKtalk for Windows

Native C++/WinRT + WinUI 3 client. It loads `tektalk_client_core.dll` through
the stable C ABI, so Snowflake, MTProto state and plugin verification remain in
the same Rust Core used by mobile and macOS.

Build from a Visual Studio 2022 Developer PowerShell with the Windows App SDK,
C++/WinRT and Windows 11 SDK workloads installed:

```powershell
./scripts/build-core-windows.ps1
msbuild clients/windows/TEKtalk.Windows/TEKtalk.Windows.vcxproj /restore /p:Configuration=Release /p:Platform=x64
```

The MVP supports account registration/login, unknown-device verification,
Message history/send, AI placeholder, profile/session display and logout.
