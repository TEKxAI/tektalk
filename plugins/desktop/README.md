# Native desktop plugin ABI

Windows and macOS hosts use the same lifecycle and security model as Valdi
plugins, but load platform-native libraries when a Valdi desktop runtime is not
available. Each catalog entry resolves to an architecture-specific DLL or
dylib and exports `include/tektalk_plugin.h` ABI version 1.

Install sequence: download to a non-executable staging directory, verify the
signed catalog, SHA-256 and Ed25519 signature in Rust Core, verify the native
platform signature, atomically move into a versioned private directory,
health-check in an isolated helper process, activate, and retain the
last-known-good version for rollback. Required Message and Me plugins always
have bundled recovery libraries.

Plugins receive only a capability-filtered `tektalk_host_api`. They never
receive tokens, auth keys, raw sockets, database handles or another plugin's
storage. ABI objects must be destroyed before unloading the library.
