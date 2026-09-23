# TEKtalk Realtime Protocol v1

TEKtalk v1 is transport-independent and MTProto-inspired, but not Telegram-compatible. Identity and session bootstrap use HTTPS. Realtime frames use secure WebSocket (`wss`) initially; QUIC can carry identical frames later.

## Handshake

1. Client authenticates through HTTPS and receives an access token.
2. Client posts the token to `/v1/realtime/bootstrap` and receives a one-use ticket plus the server ephemeral X25519 public key.
3. Client opens `wss://host/v1/realtime/connect?ticket=...` and sends its 32-byte ephemeral X25519 public key as the first binary WebSocket message.
4. Both sides calculate X25519 shared secret and derive a 32-byte session key using HKDF-SHA256, salt `tektalk-v1`, info `realtime-session`.
5. Every subsequent WebSocket message is exactly one encrypted frame. Tickets expire after 60 seconds and are consumed once.

TLS authenticates the bootstrap server; the in-band handshake adds session keys and protocol-level replay protection. Production clients should apply the organization's certificate pinning and key-transparency policy.

## Frame

All integers are unsigned, network byte order.

| Offset | Size | Field |
|---:|---:|---|
| 0 | 1 | version = 1 |
| 1 | 1 | kind |
| 2 | 2 | flags |
| 4 | 8 | logical session ID |
| 12 | 8 | message ID |
| 20 | 8 | monotonically increasing sequence |
| 28 | 4 | plaintext payload length |
| 32 | N+16 | ChaCha20-Poly1305 ciphertext and tag |

The 32-byte header is authenticated additional data. Nonce is `sequence[8] || low32(message_id)[4]`; a key must never reuse a `(sequence,message_id)` pair. Receivers close the connection when sequence is not strictly increasing.

Kinds: `1 SEND_MESSAGE`, `2 SEND_ACK`, `3 DELIVER_MESSAGE`, `4 DELIVERY_ACK`, `5 PING`, `6 PONG`, `7 ERROR`.

Payloads are UTF-8 JSON in this reference slice so native apps can inspect them easily. Production migration replaces payload encoding with Protobuf while retaining the envelope.

## Delivery semantics

- Client assigns a UUID `client_message_id` and retries until `SEND_ACK`.
- Server enforces uniqueness on `(sender_id, client_message_id)` and returns the original outcome on duplicate retry.
- Snowflake-like `server_message_id` determines order within a conversation.
- ACK means durably committed, not read by the recipient.
- Session survives network reconnect; transport does not define identity.
