# Threat model summary

Protected assets include credentials, session keys, phone identity, message contents and social graph. Principal threats are credential stuffing, OTP abuse/SIM swap, device theft, malicious clients, replay, traffic analysis, insider access, dependency compromise and denial of service.

Controls present in the skeleton: Argon2id password hashes, normalized answers hashed independently, constant external reset response, short access tokens, rotating opaque refresh tokens, one-use expiring device challenges, ephemeral X25519 exchange, AEAD frames, strict sequence checks and message deduplication.

Required before production: dedicated KMS/HSM, secrets rotation, TLS termination policy, mTLS service identity, certificate pinning/key transparency decision, device attestation, refresh endpoint and reuse detection, OTP provider plus throttling and attempt counters, CAPTCHA/risk engine, encrypted storage fields, content/E2EE product decision, audit trails, data retention/deletion, dependency scanning, penetration test and independent protocol review.

