# Native client production track

TEKtalk uses a clean-room native implementation. The interaction patterns are informed by mature messaging products, but no upstream UI source, name, logo or GPL asset is copied into this repository.

## Implemented vertical slice

| Capability | Android | iOS | Backend |
|---|---|---|---|
| Registration | Compose form | SwiftUI form | `/v1/auth/register` |
| Password login | Compose form | SwiftUI form | `/v1/auth/login` |
| Unknown-device verification | Security-answer flow | Security-answer flow | `/v1/auth/device/verify` |
| Direct text send | Chat composer | Chat composer | `/v1/chat/messages/send` |
| Message history | Bubble timeline | Bubble timeline | `/v1/chat/messages/history` |
| Host tabs | Message / AI / Me | Message / AI / Me | Signed plugin contract |

The first direct-chat screen accepts conversation and recipient UUIDs. A contact/conversation directory will replace those diagnostic fields when Friend and Group services expose client-facing APIs.

## Production gates still required

- Store refresh/device credentials in Android Keystore-backed encrypted storage and iOS Keychain; keep access tokens only in memory.
- Add refresh-token rotation, explicit logout/revocation, certificate pinning policy and attestation hooks.
- Add media upload sessions, thumbnails, local database, outbox retries and background reconciliation.
- Implement APNs/FCM registration and privacy-safe notification payloads.
- Complete the native MTProto connection manager for live delivery; HTTPS history remains the recovery source.
- Implement call signaling, WebRTC media, TURN credentials, regional SFU deployment and CallKit/ConnectionService integration before enabling calling.
- Add accessibility, localization, screenshot tests, UI automation, performance budgets and crash/telemetry consent.

The call action currently reports that signaling/SFU is unavailable instead of simulating a successful call. Production voice/video cannot be represented honestly by UI alone.
