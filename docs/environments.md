# Sandbox, staging and production

All environments run the same images and Kubernetes base. Only overlay values,
external endpoints, capacity and secrets differ. Promotion rebuilds nothing:
the exact image tag tested in sandbox is promoted to staging and production.

| Environment | Purpose | Namespace | Minimum replicas | Data policy |
|---|---|---|---:|---|
| sandbox | integration and exploratory testing | `tektalk-sandbox` | 1 | synthetic data; reset allowed |
| staging | release candidate and load validation | `tektalk-staging` | 2 | production-like, anonymized data only |
| production | customer traffic | `tektalk-production` | 3 | retention, backup and residency policies enforced |

## Required secrets

Create `tektalk-server-secrets` in every namespace through an external secret
manager. It must contain `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET` and
`OTP_HMAC_SECRET`. Never commit secret values. Each GitHub Environment must also
provide `KUBECONFIG_B64`; production should require reviewer approval.

## Promotion

1. CI must be green for the commit.
2. Publish both `server` and `business-solutions` images with the same immutable
   tag (prefer the commit SHA).
3. Run the `deploy` workflow for sandbox and execute API smoke tests.
4. Promote the same tag to staging; run migration, contract and load tests.
5. Approve production deployment and observe SLOs during the rollout window.

The public gateway exposes HTTPS endpoints while service-to-service traffic is
gRPC. `/readyz` verifies the gateway database dependency; Kubernetes uses
`/healthz` only for liveness. Business entitlements are reached through
`/v1/commerce/*`, so clients never call internal gRPC services directly.

