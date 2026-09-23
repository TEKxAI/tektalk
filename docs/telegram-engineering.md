# Telegram engineering patterns adopted by TEKtalk

TEKtalk is a technology-learning template, not a Telegram-compatible client or server. The selection rule is pragmatic: adopt a Telegram technique when it teaches a durable messaging-system principle or materially improves reliability, security, performance, or client experience. Keep TEKtalk-specific APIs where exact compatibility would add cost without learning value.

## Adoption matrix

| Layer | Pattern selected from Telegram | TEKtalk implementation direction | Priority |
|---|---|---|---|
| Cryptography | MTProto 2.0 message key, directional SHA-256 KDF, AES-256-IGE, authenticated padding | Implemented in Rust and Android; complete and cross-test iOS | P0 |
| Session | Stable logical session independent of a physical connection | Persist session identity and resume after reconnect | P0 |
| Delivery | Monotonic message IDs, sequence numbers, ACK, resend and deduplication | Add explicit service messages and bounded resend queue | P0 |
| Batching | Message containers and acknowledgements for multiple messages | Add size/time-bounded containers after single-message correctness | P1 |
| Transport | Encryption envelope separated from WebSocket/TCP/QUIC carrier | Keep WSS default; provide adapters behind one interface | P1 |
| Data model | Local-first message database and server reconciliation | Introduce repositories, optimistic send states and gap recovery | P0 |
| Sync | Difference-based updates instead of full conversation reload | Add per-user/per-conversation cursors and gap fetch | P1 |
| Media | Upload parts, resumability, content hashes and CDN-friendly storage | Add object-storage adapter and encrypted upload sessions | P1 |
| Client architecture | Native UI, shared protocol invariants, isolated networking/storage layers | Define matching Kotlin/Swift interfaces and conformance vectors | P0 |
| Performance | Compact binary schema and generated codecs | Use a small TEKtalk schema/codegen layer; Telegram TL compatibility is not required | P2 |
| Operations | Stateless gateways, connection-aware routing and regional cells | Evolve modular monolith through measured extraction points | P2 |

## Intentionally not adopted

- Telegram production API IDs, keys, DC addresses, branding, assets, or service endpoints.
- Full Telegram TL API surface and layer-by-layer backward compatibility.
- The requirement that official Telegram clients connect to TEKtalk unchanged.
- Feature breadth such as channels, bots, stories, payments, or calls before core chat invariants are proven.
- Source-code copying without preserving the upstream license and notices.

## Client component blueprint

Both native clients should expose the same conceptual components while remaining idiomatic to their platforms:

| Component | Responsibility |
|---|---|
| `AuthService` | Login, refresh, device challenge and recovery |
| `AuthKeyStore` | Protect long-lived session/auth keys with Keychain or Android Keystore |
| `MTProtoCodec` | Encrypt/decrypt envelopes and validate message invariants |
| `Transport` | WSS connection lifecycle, heartbeat and network migration |
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

## Upstream source policy

Telegram Android and iOS clients are valuable references for component boundaries, connection state machines, storage/sync behavior, and performance techniques. When code is reused, review the exact upstream license, retain notices, isolate the derivative client work, and document modifications. Prefer reimplementing small protocol or architecture concepts from public specifications when that keeps the learning template clear and independently maintainable.
