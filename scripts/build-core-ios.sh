#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo build --manifest-path "$root/Cargo.toml" --release -p tektalk-client-core --target aarch64-apple-ios
cargo build --manifest-path "$root/Cargo.toml" --release -p tektalk-client-core --target aarch64-apple-ios-sim
echo "iOS static libraries generated under target/<target>/release"
