#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
command -v cargo-ndk >/dev/null 2>&1 || cargo install cargo-ndk --locked
cd "$root"
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o clients/android/app/src/main/jniLibs build --release -p tektalk-client-core
echo "Android Rust Core: clients/android/app/src/main/jniLibs"
