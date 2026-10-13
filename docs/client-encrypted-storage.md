# Client encrypted storage

TEKtalk native clients share the Rust `tektalk-client-core` persistence layer.
The Web client uses an IndexedDB/Web Crypto adapter because a browser cannot
link SQLCipher.

## Account layout

```text
accounts/{account_id}/
├── catalog.db
├── search.db
├── jobs.db
└── conversations/{id-prefix}/{conversation-id}.db
```

Conversation databases are created lazily. Rust keeps a bounded LRU of open
handles (8 on mobile, 12 on desktop); the number stored on disk is independent
of that runtime limit.

## Encryption

Every native database uses SQLCipher with a random 256-bit key. The core checks
`PRAGMA cipher_version` and fails closed if plaintext SQLite is linked.

- Android wraps the DB key with an AES-GCM Android Keystore key.
- iOS/macOS store the DB key in Keychain.
- Windows protects the DB key with user-scoped DPAPI.
- Web encrypts records with AES-256-GCM and a non-extractable Web Crypto key.

Web encryption protects data at rest, not against script executing in the same
origin, so CSP and XSS prevention remain mandatory.

## Schema decisions

- Server entity IDs are positive Snowflake `i64` values stored as `INTEGER`.
- `local_id` is the stable SQLite row identity.
- `client_nonce` is a random positive `i64` idempotency key.
- Identity, ordering, filtering, sync and frequently updated values are columns.
- Extensible data uses `BLOB + codec + version`; codec `1` is JSON UTF-8.
- Profiles use `avatar_url`, `cover_url`, `cover_type` and one monotonic
  `profile_version` covering all visible profile changes.

`catalog.db` supplies the complete Messages-tab projection without opening all
conversation DBs. `search.db` is rebuildable. Conversation DBs remain the
message source of truth, and projection outboxes make cross-DB updates retryable.

## Migrations

Physical versions remain explicit. Conversation DBs migrate lazily when opened,
avoiding an all-database startup scan. Payload versions evolve separately and
are upgraded in memory by the Rust codec layer.
