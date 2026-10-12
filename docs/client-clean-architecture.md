# Client Clean Architecture

All TEKtalk clients use the same dependency direction:

```text
Presentation -> Domain <- Data
                     ^
                     |
               Rust Shared Core
```

## Layer ownership

| Layer | Owns | Must not own |
| --- | --- | --- |
| Presentation | UI state, navigation, rendering, OS lifecycle | HTTP DTOs, database queries, protocol framing |
| Domain | Entities, repository contracts, use-case rules | UI frameworks, HTTP clients, persistence drivers |
| Data | HTTP/database/plugin adapters, DTO mapping | UI state or product presentation decisions |
| Rust Shared Core | MTProto 2.0, transport policy, crypto, IDs, sync/state primitives | Platform UI or OS permission flows |

Android packages are `presentation`, `domain`, and `data`. iOS and macOS use matching source groups. Windows keeps generated WinUI views in the project root and places application code under `Domain`, `Data`, and `Infrastructure` source groups.

The dependency rule is enforced by `scripts/check-client-architecture.sh`. New shared business rules should be implemented in Rust first and exposed through the stable C ABI/JNI surface; platform code may contain only UI-specific validation and orchestration.

