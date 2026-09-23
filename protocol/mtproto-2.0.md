# TEKtalk MTProto 2.0 Wire Protocol

TEKtalk realtime messages use the MTProto 2.0 established-session encrypted envelope. The wire codec uses Telegram's MTProto 2.0 `auth_key_id`, SHA-256 `msg_key` derivation, AES-256-IGE encryption, internal message header, and authenticated random padding.

This document describes the code currently shipped in this repository. TEKtalk adopts selected MTProto 2.0 mechanisms but does not target full Telegram client interoperability. Reliability patterns such as service messages, salt rotation, containers and ACK/resend are tracked in `docs/telegram-engineering.md`; the full Telegram TL API is intentionally out of scope.

## Session bootstrap

1. The client authenticates over HTTPS and requests a one-use realtime ticket.
2. The server returns the ticket and its ephemeral X25519 public key.
3. The client connects to `wss://host/v1/realtime/connect?ticket=...` and sends its 32-byte ephemeral X25519 public key.
4. Both peers calculate the X25519 shared secret and expand a 256-byte MTProto authorization key with HKDF-SHA256 using salt `tektalk-mtproto-bootstrap-v1` and info `mtproto-auth-key`.
5. Every following binary WebSocket message contains one MTProto 2.0 encrypted message.

The X25519 bootstrap is an explicitly isolated TEKtalk adapter rather than Telegram's RSA/DH authorization-key exchange. This keeps the template compact while allowing the bootstrap to be reviewed or replaced independently of the MTProto 2.0 message codec.

## Encrypted message

The external envelope is:

| Field | Size | Encoding |
|---|---:|---|
| `auth_key_id` | 8 bytes | Lower 64 bits of SHA-1 of the 256-byte auth key |
| `msg_key` | 16 bytes | Bytes 8–23 of the MTProto 2.0 SHA-256 message-key digest |
| encrypted data | multiple of 16 bytes | AES-256-IGE |

The decrypted data is:

| Field | Size | Encoding |
|---|---:|---|
| server salt | 8 bytes | little-endian |
| session ID | 8 bytes | little-endian |
| message ID | 8 bytes | little-endian |
| sequence number | 4 bytes | little-endian |
| body length | 4 bytes | little-endian |
| message body | variable | current application payload |
| random padding | 12–1024 bytes | authenticated by `msg_key` |

Client-to-server and server-to-client directions use the MTProto 2.0 KDF with offsets `x = 0` and `x = 8`, respectively. Receivers validate `auth_key_id`, recompute `msg_key`, validate padding and message-ID parity, and reject non-increasing message IDs.

## Application payload

The current vertical slice carries TEKtalk JSON commands inside the MTProto message body. A compact generated binary schema is a future optimization; full Telegram TL compatibility is not a project requirement. The encryption envelope must not be described as a proprietary TEKtalk v1 frame.

## Transport

The runnable client and server carry encrypted messages over secure WebSocket. MTProto transport framing and the encryption envelope are separate layers; the repository also contains an abridged-transport encoder for the planned native MTProto TCP adapter.
