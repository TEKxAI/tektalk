# TEKtalk engineering patterns

TEKtalk is a reusable technology-learning template. The selection rule is pragmatic: adopt a technique when it teaches a durable messaging-system principle or materially improves reliability, security, performance, or client experience. Keep the public API and product model independently owned by TEKtalk.

## Engineering matrix

| Layer | Selected pattern | TEKtalk implementation direction | Priority |
|---|---|---|---|
| Cryptography | MTProto 2.0 message key, directional SHA-256 KDF, AES-256-IGE and authenticated padding | One Rust Core implementation consumed through native bindings | P0 |
| Session | Stable logical session independent of a physical connection | Persist session identity and resume after reconnect | P0 |
| Delivery | Snowflake persisted IDs; MTProto time/parity IDs; sequence numbers, ACK, resend and deduplication | Shared generators implemented; add explicit service messages and a bounded resend queue | P0 |
| Batching | Message containers and acknowledgements for multiple messages | Add size/time-bounded containers after single-message correctness | P1 |
| Transport | Encryption envelope separated from TCP, WebSocket or QUIC carrier | Native TCP is primary; WSS is mandatory fallback behind Rust TransportManager | P0 |
| Data model | Local-first message database and server reconciliation | Introduce repositories, optimistic send states and gap recovery | P0 |
| Sync | Difference-based updates instead of full conversation reload | Add per-user and per-conversation cursors with gap fetch | P1 |
| Media | Upload parts, resumability, content hashes and CDN-friendly storage | Add object-storage adapter and encrypted upload sessions | P1 |
| Client architecture | Native UI, shared protocol invariants and isolated networking/storage layers | Define matching Kotlin and Swift interfaces with conformance vectors | P0 |
| Performance | Compact binary schema and generated codecs | Use a small TEKtalk schema and code-generation layer | P2 |
| Operations | Stateless gateways, connection-aware routing and regional cells | Evolve the modular monolith through measured extraction points | P2 |

## Product boundaries

- Use only TEKtalk API identifiers, keys, service endpoints, branding and assets.
- Keep the application schema intentionally small and owned by TEKtalk.
- Do not require third-party messaging clients to connect unchanged.
- Delay feature breadth such as channels, bots, stories, payments or calls until core chat invariants are proven.
- Preserve all applicable licenses and notices whenever external open-source code is introduced.

## Client component blueprint

Both native clients expose the same conceptual components while remaining idiomatic to their platforms:

| Component | Responsibility |
|---|---|
| `AuthService` | Login, refresh, device challenge and recovery |
| `AuthKeyStore` | Protect long-lived session/auth keys with Keychain or Android Keystore |
| `MTProtoCodec` | Encrypt/decrypt envelopes and validate message invariants |
| `TransportManager` | TCP-first selection, WSS fallback, heartbeat and network migration |
| `SessionCoordinator` | Salt/session IDs, sequence numbers, ACK/resend and reconnect |
| `MessageRepository` | Local database, optimistic state, deduplication and sync cursors |
| `MediaTransfer` | Chunking, retry, hashing and background upload/download |
| `ConversationViewModel` | UI state only; no wire or storage logic |

## Server component blueprint

| Component | Responsibility |
|---|---|
| Edge/API | HTTPS auth/bootstrap, rate limits and request validation |
| Realtime gateway | Connection ownership, MTProto decoding and backpressure |
| Session service | Session state, replay window, ACK/resend and presence |
| Message service | Idempotent commit, ordering and conversation authorization |
| Sync service | Cursor/difference API and gap repair |
| Media service | Upload sessions, metadata and object-storage authorization |
| Outbox/event relay | Reliable downstream events without dual writes |

The current modular monolith may host several components in one process. The interfaces and invariants matter more than prematurely splitting them into network services.

## Delivery gates

1. Rust, Android and iOS pass shared MTProto envelope vectors.
2. Reconnect tests prove no duplicate logical messages and no lost committed ACKs.
3. Offline clients reconcile optimistic messages and server history deterministically.
4. Fault injection covers reordered, replayed, corrupted and delayed frames.
5. Media transfer resumes after process death and verifies content hashes.
6. Load tests, traces and production-like dashboards justify any service extraction.

## External source policy

External open-source clients and protocol implementations can be useful references for component boundaries, connection state machines, storage/sync behavior and performance techniques. Before reusing code, review the exact upstream license, retain required notices, isolate derivative work and document modifications. Prefer implementing compact protocol and architecture concepts from public specifications when that keeps the learning template clear and independently maintainable.
