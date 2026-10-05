# TEKtalk shared Rust core

The Rust core is the single implementation of deterministic cross-platform protocol/session state, MTProto 2.0 envelope cryptography and transport framing. It also owns the production Snowflake generator used for persisted message IDs. Platform sockets, secure storage, background execution and OS capabilities remain behind Swift/Kotlin adapters. WebRTC and platform codecs remain isolated C/C++ dependencies when their upstream APIs require it.

Build and test:

```bash
cargo test -p tektalk-client-core
cargo build --release -p tektalk-client-core
```

The library emits `staticlib`, `cdylib` and `rlib` artifacts. Cross-compile by installing the required Rust target and passing it to Cargo, for example:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo build --release -p tektalk-client-core --target aarch64-apple-ios

rustup target add aarch64-linux-android
cargo build --release -p tektalk-client-core --target aarch64-linux-android
```

Android cross-compilation also requires the matching Android NDK linker configuration. Production packaging should build an XCFramework for iOS and per-ABI `.so` files for Android.

The public mobile boundary remains the stable C ABI in `include/tektalk/ffi.h`. Swift imports it through a module map or bridging header; Android calls it through a thin JNI adapter. Rust ownership never crosses either boundary, and each opaque session handle must be accessed serially and destroyed exactly once.

Two ID domains are intentionally separate:

- Persisted `server_message_id`: Snowflake `41-bit timestamp / 10-bit node / 12-bit sequence`.
- MTProto wire `msg_id`: Unix-time fixed-point value with direction parity and monotonic ordering.

Each deployed message-writer instance must receive a unique `SNOWFLAKE_NODE_ID` in `0..1023`. Reusing a node ID concurrently can create duplicate IDs.
