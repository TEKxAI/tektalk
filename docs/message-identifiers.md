# Message identifier policy

TEKtalk uses one shared Rust implementation for every authoritative persisted message ID. The layout is a signed-positive 64-bit Snowflake value:

| Field | Bits | Meaning |
|---|---:|---|
| Timestamp | 41 | Milliseconds since `2024-01-01T00:00:00Z` |
| Node | 10 | Deployment-assigned writer ID `0..1023` |
| Sequence | 12 | Per-node counter within the millisecond |

The generator is thread-safe, rejects clock rollback, blocks generation until the next millisecond after sequence exhaustion, and exposes decode helpers for operations. Every concurrently active writer must have a unique `SNOWFLAKE_NODE_ID`; production deployment should allocate it from StatefulSet ordinal or a lease service.

`client_message_id` remains a UUID idempotency key. It lets a client safely retry before receiving the authoritative Snowflake ID.

MTProto `msg_id` is a different namespace. It is derived from Unix time, must increase within a session, and encodes direction parity. Replacing it with Snowflake would violate the protocol. Both generators live in `tektalk-client-core`, so server, iOS and Android share the same invariants without conflating their purposes.
