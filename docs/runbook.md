# Operations runbook

## Deploy

Build an immutable image, scan it, deploy to one canary cell, verify auth success rate, WebSocket handshake rate, send/ACK latency and database errors, then expand by cell. Database migrations must be backward compatible and applied before code that depends on them.

## Golden signals

- HTTPS and realtime request rate, error rate, p50/p95/p99 latency
- Active connections, reconnect rate, invalid-frame/replay closures
- Message commit and delivery latency, dedup hit rate, ACK timeout rate
- PostgreSQL saturation/replica lag; Scylla p99/compaction/backlog; Kafka consumer lag
- OTP request/verification rates by risk segment, without phone numbers in metric labels

## Failure policy

If Kafka, search, analytics, AI, push or presence fails, durable message commit continues. If Scylla is authoritative and cannot meet the configured consistency level, do not acknowledge the message. Shed bulk/business traffic before personal chat. Never retry non-idempotent writes without `client_message_id`.

## Secrets

Secrets are injected by the platform secret manager, never committed or stored in ConfigMaps. Rotate JWT and OTP keys with overlapping key IDs. Store long-lived signing and encryption keys in an HSM-backed KMS.
