#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
required='Cargo.toml server/src/main.rs protocol/tektalk-v1.md clients/android/app/src/main/java/vn/tektalk/Protocol.kt clients/ios/TEKtalk/TEKProtocol.swift infra/postgres/001_init.sql infra/scylla/schema.cql infra/k8s/base.yaml'
for file in $required; do test -s "$root/$file" || { echo "missing: $file" >&2; exit 1; }; done
grep -q 'tektalk-v1' "$root/server/src/realtime.rs"
grep -q 'tektalk-v1' "$root/clients/android/app/src/main/java/vn/tektalk/Protocol.kt"
grep -q 'tektalk-v1' "$root/clients/ios/TEKtalk/TEKProtocol.swift"
grep -q 'AES-256-IGE' "$root/server/src/mtproto.rs"
echo "layout and cross-client protocol constants: ok"
