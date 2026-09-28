# TEKtalk service boundaries

The current runnable Rust server remains the migration source. New services are extracted behind the contracts in `contracts/proto` without changing public client behavior.

| Service | Owns | Synchronous API | Events |
|---|---|---|---|
| `identity` | accounts, credentials, devices | `IdentityService` | device revoked, password changed |
| `conversation` | membership and permissions | conversation contract | member changed |
| `message` | idempotent commit and ordering | `MessageService` | message committed |
| `sync` | cursors and difference calculation | `SyncService` | consumes domain events |
| `session` | connection routing and presence | session contract | presence changed |
| `media` | upload sessions and metadata | media contract | media ready |
| `notification` | APNs/FCM delivery | admin-only contract | consumes message events |
| `plugin-registry` | signed catalog and rollout | `PluginRegistryService` | plugin rollout changed |
| `ai-orchestrator` | models, tools and quotas | AI contract | AI task completed |

Each service owns its schema and publishes events through a transactional outbox. Cross-service database queries and distributed dual writes are forbidden.
