#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cargo build --manifest-path "$root/Cargo.toml" --release -p tektalk-client-core
mkdir -p "$root/clients/macos/Frameworks"
cp "$root/target/release/libtektalk_client_core.dylib" "$root/clients/macos/Frameworks/"
echo "macOS Rust Core: clients/macos/Frameworks/libtektalk_client_core.dylib"
