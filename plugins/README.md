# TEKtalk dynamic plugins

The host resolves a signed catalog from `PluginRegistryService`, downloads immutable module artifacts from a CDN, verifies SHA-256 and Ed25519 signatures, then installs them atomically.

Runtime rules:

1. Download into a staging directory.
2. Verify catalog signature, artifact digest and artifact signature.
3. Validate manifest schema, host version and requested capabilities.
4. Load the module in an isolated runtime with a memory/time budget.
5. Mark healthy only after the entry point renders and responds to a probe.
6. Promote it to the active slot atomically.
7. Roll back to the last-known-good bundled or cached version after repeated crashes.

Plugins never receive raw tokens, keys, sockets or native objects. They call versioned host capabilities and shared-core façades. The default Message, AI and Me modules are bundled as recovery versions even when remote delivery is enabled.
