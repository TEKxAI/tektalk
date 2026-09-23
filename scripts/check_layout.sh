#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
required='Cargo.toml server/src/main.rs protocol/zchat-v1.md clients/android/app/src/main/java/vn/zchat/Protocol.kt clients/ios/ZChat/ZProtocol.swift infra/postgres/001_init.sql infra/scylla/schema.cql infra/k8s/base.yaml'
for file in $required; do test -s "$root/$file" || { echo "missing: $file" >&2; exit 1; }; done
grep -q 'zchat-v1' "$root/server/src/realtime.rs"
grep -q 'zchat-v1' "$root/clients/android/app/src/main/java/vn/zchat/Protocol.kt"
grep -q 'zchat-v1' "$root/clients/ios/ZChat/ZProtocol.swift"
grep -q 'HEADER_LEN:usize=32' "$root/server/src/protocol.rs"
echo "layout and cross-client protocol constants: ok"
