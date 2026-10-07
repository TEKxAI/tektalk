# TEKtalk service boundaries

The current runnable Rust server remains the migration source. New services are extracted behind the contracts in `contracts/proto` without changing public client behavior.

| Service | Owns | Synchronous API | Events |
|---|---|---|---|
| `signin-signup` | Core Platform accounts, credentials, profiles and devices | `AccountService` | account created, device challenged |
| `conversation` | membership and permissions | conversation contract | member changed |
| `chat` | OTT text, voice, sticker, video and photo messages | `ChatService` | message committed |
| `sync` | cursors and difference calculation | `SyncService` | consumes domain events |
| `session-management` | L0/L1/L2 access, scopes, audience and token lifecycle | `SessionManagementService` | session elevated/revoked |
| `consent-management` | platform and third-party consent grants | `ConsentManagementService` | consent granted/revoked |
| `media` | upload sessions and metadata | media contract | media ready |
| `notification` | APNs/FCM delivery | admin-only contract | consumes message events |
| `plugin-registry` | signed catalog and rollout | `PluginRegistryService` | plugin rollout changed |
| `ai-orchestrator` | models, tools and quotas | AI contract | AI task completed |
| `product-catalog` | uVAS/eVAS plans and capability composition | `ProductCatalogService` | plan published/retired |
| `subscription` | commercial lifecycle and effective entitlements | `SubscriptionService` | subscription activated/cancelled |

Each service owns its schema and publishes events through a transactional outbox. Cross-service database queries and distributed dual writes are forbidden.

## Runnable platform services

The learning stack packages four independent gRPC processes from one Rust crate so domain logic can be shared without coupling deployment or scaling.

| Process | Local port | Platform boundary |
|---|---:|---|
| `signin-signup` | 50052 | Core Platform registration and password sign-in |
| `chat` | 50053 | OTT message validation, idempotency, ordering and history |
| `session-management` | 50054 | Access-level, scope, audience, expiry, elevation and revocation checks |
| `consent-management` | 50055 | Purpose-bound data and scope grants for platform or third-party owners |
| `business-solutions` | 50056 | Product catalog, subscription lifecycle and commercial entitlement |

Session levels are cumulative. L0 covers `identifier.*` and `profile.*`; L1 adds `friend.*`, `group.*`, `community.*`, `oauth.*` and `security.*`; L2 adds `app.*`, `miniapp.*` and `business.*`. Elevation requires a security proof, rotates the access token and shortens the elevated lifetime.

Consent is evaluated independently of session authorization. A request is allowed only when an active, unexpired grant matches the user, owner type, owner, purpose, data categories and scopes. Revocation takes effect immediately. Production deployments should persist both domains in separate databases and publish immutable audit events; the template deliberately uses in-memory repositories so the service contracts and local topology remain easy to study.
