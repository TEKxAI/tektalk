# TEKtalk software delivery lifecycle

## Flow

1. **Plan:** define the Core, Platform Service or Business Solution owner; record security, privacy, localization and SLO impact.
2. **Develop:** short-lived branch, backward-compatible contracts/migrations, unit tests and no secrets in source.
3. **Review:** CODEOWNERS approval plus CI, security and architecture required checks.
4. **Integrate:** merge to `main`; build/test all four clients, Rust workspace, containers and full Compose smoke flow.
5. **Release:** signed Git tag builds server and Business Solutions images with SBOM/provenance. Images are immutable.
6. **Deploy sandbox:** apply the sandbox overlay and execute authenticated commerce smoke tests.
7. **Promote staging:** reuse the exact image digest; run regression, migration compatibility, load and restore tests.
8. **Promote production:** GitHub Environment approval, rolling deployment, SLO observation and automatic rollback on verification failure.
9. **Operate:** incident response, vulnerability remediation, audit/outbox monitoring and tested database restoration.

## Required merge gates

- `ci / architecture`, `rust`, `android`, `ios`, `macos`, `windows`, `docker`, `local-stack`.
- `security / dependencies`, `filesystem`, `secrets`.
- At least one CODEOWNER approval; dismiss stale approvals after new commits.
- Linear history and no direct production deployment from an untagged artifact.

## Release rules

- Database changes are additive and compatible with the previous application version.
- A rollback never attempts to reverse destructive schema changes.
- Sandbox, staging and production promote the same tag/digest.
- Production secrets come from an external secret manager and are rotated independently of images.
- `OTP_DEV_ECHO`, unsigned plugins and development fallback paths are forbidden in production.

## Definition of done

A change is done only when implementation, tests, localization, telemetry, runbook, migration, deployment and rollback are complete. “Merged” is not equivalent to “released”; “released” is not equivalent to “verified in production.”

