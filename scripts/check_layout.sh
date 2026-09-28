#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
required='Cargo.toml server/src/main.rs services/plugin-registry/src/main.rs services/plugin-registry/Dockerfile services/platform-services/Dockerfile services/platform-services/src/bin/signin-signup.rs services/platform-services/src/bin/chat.rs services/platform-services/src/bin/session-management.rs services/platform-services/src/bin/consent-management.rs protocol/mtproto-2.0.md docs/tektalk-engineering.md docs/target-architecture.md docs/getting-started.md docs/deployment.md contracts/buf.yaml contracts/proto/tektalk/v1/plugin_registry.proto contracts/proto/tektalk/v1/account.proto contracts/proto/tektalk/v1/chat.proto contracts/proto/tektalk/v1/session_management.proto contracts/proto/tektalk/v1/consent_management.proto core/CMakeLists.txt core/include/tektalk/ffi.h plugins/plugin-manifest.schema.json plugins/message/manifest.json plugins/message/src/MessagePlugin.tsx plugins/ai/manifest.json plugins/ai/src/AIPlugin.tsx plugins/me/manifest.json plugins/me/src/MePlugin.tsx scripts/local-up.sh scripts/local-down.sh scripts/smoke-test.sh clients/android/app/src/main/java/vn/tektalk/Protocol.kt clients/android/app/src/main/java/vn/tektalk/plugins/PluginContract.kt clients/ios/TEKtalk/TEKProtocol.swift clients/ios/TEKtalk/PluginContract.swift infra/postgres/001_init.sql infra/scylla/schema.cql infra/k8s/base.yaml'
for file in $required; do test -s "$root/$file" || { echo "missing: $file" >&2; exit 1; }; done
grep -q 'tektalk-mtproto-bootstrap-v1' "$root/server/src/realtime.rs"
grep -q 'object MTProto2' "$root/clients/android/app/src/main/java/vn/tektalk/Protocol.kt"
grep -q 'enum MTProto2' "$root/clients/ios/TEKtalk/TEKProtocol.swift"
grep -q 'AES-256-IGE' "$root/server/src/mtproto.rs"
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
