$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root
try {
  rustup target add x86_64-pc-windows-msvc
  cargo build --release -p tektalk-client-core --target x86_64-pc-windows-msvc
  $Destination = Join-Path $Root "clients/windows/TEKtalk.Windows"
  Copy-Item (Join-Path $Root "target/x86_64-pc-windows-msvc/release/tektalk_client_core.dll") $Destination -Force
  Write-Host "Windows Rust Core copied to $Destination"
} finally { Pop-Location }
