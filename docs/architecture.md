# Target architecture and extraction path

The initial service is a modular monolith to keep transactions and local development straightforward. Boundaries are designed for later extraction:

| Domain | Initial implementation | Scale target |
|---|---|---|
| Edge / HTTPS | Axum process | Stateless Rust gateway per cell |
| Identity | Rust module + PostgreSQL | Sharded PostgreSQL behind Identity RPC |
| Realtime | Rust WebSocket tasks | Rust gateway plus BEAM session/presence fabric |
| Message write | PostgreSQL transactional slice | Rust data service + bucketed ScyllaDB |
| Presence | In-process registry | Redis-compatible ephemeral store / BEAM |
| Events | Integration seam | Kafka/Redpanda outbox publisher |

Do not dual-write PostgreSQL and Scylla. The next milestone adds a transactional outbox, replays into Scylla, validates parity, changes reads, then changes the authoritative write path. Partition keys are `conversation_id + time_bucket`; hot conversations use adaptive buckets and a consistent-hashed data-service layer for coalescing and backpressure.

Cells own gateway, realtime workers, message/data services, caches and shards. `home_cell = rendezvous_hash(user_id, active_cells)`. Cross-cell conversations route commands to the conversation's authority cell. The control plane distributes placement/configuration but is never required for an established chat session.

## Reliability invariants

- ACK only after durable commit.
- A client retry never produces a second logical message.
- Ordering authority is scoped to a conversation, never global.
- Presence, search, analytics, AI and push can fail without blocking committed chat.
- Bulk OA traffic has separate queues, quotas and CPU from personal chat.

