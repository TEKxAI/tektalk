# TEKtalk internal contracts

These Protobuf definitions are the source of truth for synchronous communication between backend services and for generated client/core DTOs.

Rules:

- Public mobile traffic does not call internal gRPC services directly.
- Every mutating request carries `RequestContext.request_id` for idempotency.
- Clients set deadlines; services propagate the earliest deadline downstream.
- Breaking changes require a new package version such as `tektalk.v2`.
- CI runs Buf lint and breaking-change checks before generation.
- Authentication, tracing and service identity travel in gRPC metadata, not business payloads.
