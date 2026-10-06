#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
required='Cargo.toml server/src/main.rs services/plugin-registry/src/main.rs services/plugin-registry/src/lib.rs services/plugin-registry/Dockerfile services/platform-services/Dockerfile services/platform-services/src/bin/signin-signup.rs services/platform-services/src/bin/chat.rs services/platform-services/src/bin/session-management.rs services/platform-services/src/bin/consent-management.rs protocol/mtproto-2.0.md docs/message-identifiers.md docs/tektalk-engineering.md docs/target-architecture.md docs/client-plugin-miniapp-architecture.md docs/client-four-platforms.md docs/getting-started.md docs/deployment.md contracts/buf.yaml contracts/proto/tektalk/v1/plugin_registry.proto contracts/proto/tektalk/v1/account.proto contracts/proto/tektalk/v1/chat.proto contracts/proto/tektalk/v1/session_management.proto contracts/proto/tektalk/v1/consent_management.proto core/Cargo.toml core/src/lib.rs core/src/id.rs core/src/mtproto.rs core/src/plugin.rs core/include/tektalk/ffi.h plugins/plugin-manifest.schema.json plugins/sdk/src/Host.ts plugins/message/manifest.json plugins/message/src/MessagePlugin.tsx plugins/ai/manifest.json plugins/ai/src/AIPlugin.tsx plugins/me/manifest.json plugins/me/src/MePlugin.tsx plugins/desktop/include/tektalk_plugin.h plugins/desktop/README.md plugins/desktop/artifacts.example.json plugins/desktop/macos/NativeTabPlugin.swift plugins/desktop/windows/NativeTabPlugin.cpp plugins/desktop/windows/TEKtalk.NativePlugin.vcxproj mini-apps/sdk/src/MiniApp.ts scripts/validate-plugins.sh scripts/local-up.sh scripts/local-down.sh scripts/smoke-test.sh scripts/build-core-android.sh scripts/build-core-ios.sh scripts/build-core-macos.sh scripts/build-core-windows.ps1 scripts/build-desktop-plugins-macos.sh scripts/build-desktop-plugins-windows.ps1 clients/android/app/src/main/java/vn/tektalk/NativeCore.kt clients/android/app/src/main/java/vn/tektalk/Protocol.kt clients/android/app/src/main/java/vn/tektalk/plugins/PluginContract.kt clients/ios/TEKtalk/NativeCore.swift clients/ios/TEKtalk/TEKProtocol.swift clients/ios/TEKtalk/PluginContract.swift clients/macos/TEKtalkMac/DesktopPluginLoader.swift clients/macos/TEKtalkMac/TEKtalkMacApp.swift clients/windows/TEKtalk.Windows/DesktopPluginLoader.h clients/windows/TEKtalk.Windows/DesktopPluginLoader.cpp clients/windows/TEKtalk.Windows/MainWindow.xaml.cpp infra/postgres/001_init.sql infra/scylla/schema.cql infra/k8s/base.yaml'
for file in $required; do test -s "$root/$file" || { echo "missing: $file" >&2; exit 1; }; done
grep -q 'tektalk-mtproto-bootstrap-v1' "$root/server/src/realtime.rs"
grep -q 'object MTProto2' "$root/clients/android/app/src/main/java/vn/tektalk/Protocol.kt"
grep -q 'enum MTProto2' "$root/clients/ios/TEKtalk/TEKProtocol.swift"
grep -q 'Aes256' "$root/core/src/mtproto.rs"
grep -q 'SnowflakeGenerator' "$root/server/src/state.rs"
test ! -e "$root/server/src/mtproto.rs" || { echo 'duplicate server MTProto codec found' >&2; exit 1; }
grep -q 'crate-type = \["staticlib", "cdylib", "rlib"\]' "$root/core/Cargo.toml"
grep -q 'extern "C" fn tektalk_session_create' "$root/core/src/lib.rs"
"$root/scripts/validate-plugins.sh"
if grep -Rqi 'chacha20' "$root/server" "$root/clients"; then echo 'ChaCha20 reference found' >&2; exit 1; fi
if grep -RqiE 'TEKtalk Realtime Protocol v1|MTProto-inspired|tektalk-v1|realtime-session' "$root/README.md" "$root/docs" "$root/protocol" "$root/server" "$root/clients"; then
  echo 'legacy realtime protocol reference found' >&2
  exit 1
fi
legacy_brand='tele''gram'
if grep -Rqi "$legacy_brand" "$root/README.md" "$root/docs" "$root/protocol" "$root/server" "$root/clients" "$root/upstream"; then
  echo 'legacy external product keyword found' >&2
  exit 1
fi
echo "layout and cross-client protocol constants: ok"
