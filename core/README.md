# TEKtalk shared C++ core

The C++20 core owns cross-platform protocol/session state, retry decisions, sync reconciliation, local repository interfaces and media chunking. Platform networking, secure storage, background execution and OS capabilities remain behind Swift/Kotlin adapters.

Build and test:

```bash
cmake -S core -B build/core -DCMAKE_BUILD_TYPE=Release
cmake --build build/core
ctest --test-dir build/core --output-on-failure
```

The public mobile boundary is the stable C ABI in `include/tektalk/ffi.h`. Swift uses an Objective-C++ wrapper; Android uses JNI. Raw C++ ownership must never cross either boundary.
