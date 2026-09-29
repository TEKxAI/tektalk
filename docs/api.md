# HTTPS API

All request and response bodies are JSON. Production ingress must expose HTTPS only. Localhost may use HTTP for development.

| Method | Path | Authentication | Purpose |
|---|---|---|---|
| POST | `/v1/auth/register` | none | Create account, password, security question, first trusted device |
| POST | `/v1/auth/login` | none | Password login; returns tokens or an unfamiliar-device challenge |
| POST | `/v1/auth/device/verify` | challenge ID | Answer security question and trust the device |
| POST | `/v1/auth/password/reset/request` | none | Request recovery OTP; enumeration-safe response |
| POST | `/v1/auth/password/reset/confirm` | OTP | Set a new password and revoke refresh tokens |
| POST | `/v1/auth/password/change` | Bearer access token | Change password after verifying current password |
| POST | `/v1/chat/messages/send` | Bearer access token | Idempotently commit a direct text message |
| POST | `/v1/chat/messages/history` | Bearer access token | Read an authorized conversation page |
| POST | `/v1/realtime/bootstrap` | access token in body | Mint one-use realtime ticket and ephemeral server key |
| GET | `/v1/realtime/connect?ticket=...` | one-use ticket | Upgrade to binary WebSocket |

Registration password policy in the reference is 10+ characters with an uppercase letter and a number. A production policy should also check breached-password corpora and allow password managers rather than requiring arbitrary symbol rotation.

The HTTPS chat façade is the reliable foreground path for native clients. Realtime delivery continues over the encrypted MTProto envelope; clients reconcile through history after reconnecting. The server validates membership, recipient, message length and client-generated idempotency keys.
