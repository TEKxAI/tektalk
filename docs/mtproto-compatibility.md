# Telegram MTProto compatibility program

Target: Telegram MTProto 2.0 established-session wire compatibility and, incrementally, Telegram TL layer 225 behavior. This repository does **not** claim full Telegram compatibility yet.

## Current conformance

| Area | State |
|---|---|
| `auth_key_id` (SHA-1 lower 64 bits) | Implemented and unit tested |
| MTProto 2.0 `msg_key` and SHA-256 KDF | Implemented and unit tested |
| AES-256-IGE encryption/decryption | Implemented and round-trip tested |
| Encrypted internal/external headers | Implemented |
| 12–1024 byte authenticated padding | Implemented |
| Message-ID parity validation | Implemented |
| Abridged transport framing | Encoder implemented |
| RSA/DH authorization-key exchange | Pending |
| Salt rotation, ACK, resend, containers | Pending |
| Telegram TL layer 225 | Pending; vertical slice starts with auth/users/messages |
| Android/iOS Telegram forks | Pending upstream import and GPL compliance review |

The deployable path remains on the existing TEKtalk transport until the MTProto client adapters pass bidirectional conformance tests. This prevents a partially migrated protocol from breaking production.

## Client upstream and license boundary

- Android upstream: `https://github.com/DrKLO/Telegram`, GPL-2.0.
- iOS upstream: `https://github.com/TelegramMessenger/Telegram-iOS`, GPL family; preserve the exact upstream license and notices.
- Forked mobile clients must remain separately identifiable GPL works. Server code remains independently licensed and communicates over a network protocol boundary.
- Never copy Telegram branding, API credentials, signing keys, production DC addresses, or private server keys.

## Delivery gates

1. Official MTProto test vectors plus cross-language AES-IGE/KDF vectors pass.
2. RSA/DH handshake generates a 2048-bit auth key without transmitting it.
3. Salt/session/msg-id/seq-no/ACK/resend/container behavior passes fault-injection tests.
4. Android fork connects to a TEKtalk test DC and completes TEK auth plus 1:1 chat.
5. iOS fork passes the same black-box suite.
6. Only then enable MTProto by default in production.

