# TEKtalk shared Rust core

The Rust core owns deterministic cross-platform protocol/session state, retry decisions, sync reconciliation, local repository interfaces and media chunking. Platform networking, secure storage, background execution and OS capabilities remain behind Swift/Kotlin adapters. WebRTC and platform codecs remain isolated C/C++ dependencies when their upstream APIs require it.

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
