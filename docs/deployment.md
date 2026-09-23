# Deployment guide

The repository supports two deployment paths: Docker Compose for a single learning host and Kubernetes for a reusable environment template. It does not provision managed PostgreSQL, Redis, ScyllaDB, Kafka, DNS, TLS certificates, or a cloud load balancer.

## Release flow

Every push and pull request runs the layout/legacy guard, Rust Clippy and tests, Android debug build, and Docker image build. A Git tag matching `v*` runs `.github/workflows/release.yml` and publishes the server image to:

```text
ghcr.io/<owner>/<repository>/server:<tag>
```

Create a release after the normal CI run is green:

```bash
git tag -a v0.1.0 -m "TEKtalk learning template v0.1.0"
git push origin v0.1.0
```

In GitHub Actions, wait for `release-image` to complete. Configure the resulting GHCR package as public, or create an Kubernetes `imagePullSecret` for private packages.

## Single-host deployment

On a Linux host with Docker Compose:

```bash
git clone https://github.com/TEKxAI/tektalk.git
cd tektalk
cp .env.example .env
# Replace secrets and set OTP_DEV_ECHO=false.
docker compose up --build -d
curl --fail http://127.0.0.1:8080/healthz
```

Put a TLS reverse proxy in front of port 8080. It must support WebSocket upgrades, preserve `X-Forwarded-Proto`, use HTTPS/WSS externally, and apply an idle timeout suitable for persistent chat connections. Do not expose PostgreSQL, Redis, ScyllaDB, or Kafka ports to the internet.

## Kubernetes prerequisites

- A Kubernetes 1.29+ cluster and `kubectl`
- A published server image
- Reachable PostgreSQL and Redis services
- A namespace and ingress/TLS solution
- Metrics Server if the included HPA is used

ScyllaDB and Kafka variables are retained for the architecture roadmap, but the current runnable Rust slice requires PostgreSQL and Redis at startup.

## Kubernetes secrets

Create a namespace and secret without committing values:

```bash
kubectl create namespace tektalk
kubectl -n tektalk create secret generic tektalk-server-secrets \
  --from-literal=DATABASE_URL='postgres://USER:PASSWORD@HOST:5432/DB' \
  --from-literal=REDIS_URL='redis://HOST:6379' \
  --from-literal=JWT_SECRET="$(openssl rand -hex 32)" \
  --from-literal=OTP_HMAC_SECRET="$(openssl rand -hex 32)" \
  --from-literal=OTP_DEV_ECHO='false'
```

For shared environments, replace this imperative example with External Secrets, Sealed Secrets, or the cloud provider's secret manager. Never store plaintext Kubernetes Secrets in Git.

## Apply the template

Update the image in `infra/k8s/base.yaml` to the immutable tag produced by the release workflow, then apply:

```bash
kubectl -n tektalk apply -f infra/k8s/base.yaml
kubectl -n tektalk rollout status deployment/tektalk-server --timeout=180s
kubectl -n tektalk get pods,service,hpa,pdb
kubectl -n tektalk port-forward service/tektalk-server 8080:80
curl --fail http://127.0.0.1:8080/healthz
```

For a repeatable promotion pipeline, avoid editing the manifest in place. Render the image tag with Kustomize, Helm, or your GitOps controller and promote the same image digest between environments.

## HTTPS and WSS ingress

Expose both REST and realtime endpoints through the same TLS hostname. The ingress or gateway must:

- redirect HTTP to HTTPS;
- support WebSocket connection upgrades;
- set an idle timeout longer than the client heartbeat interval;
- limit request bodies and connection creation rates;
- pass `/healthz` only to internal probes when possible.

Native client base URLs must use `https://` and `wss://`. Certificate pinning is an optional learning extension and requires an operational rotation strategy.

## Rollout and rollback

Inspect the new version before promoting traffic:

```bash
kubectl -n tektalk logs deployment/tektalk-server --tail=200
kubectl -n tektalk rollout history deployment/tektalk-server
```

Rollback if health checks, authentication, realtime connection rate, or message ACK behavior regresses:

```bash
kubectl -n tektalk rollout undo deployment/tektalk-server
kubectl -n tektalk rollout status deployment/tektalk-server
```

Database migrations must remain backward compatible with the previous application image. A pod rollback cannot undo a destructive schema migration.

## Post-deploy checklist

- `/healthz` succeeds through the intended route.
- Registration, login, token refresh, and OTP recovery are tested with non-production accounts.
- Two clients can connect, send, deduplicate, and acknowledge a message.
- Invalid/replayed MTProto frames are rejected.
- Logs contain no secrets, OTPs, tokens, or message bodies.
- Alerts cover error rate, latency, reconnects, database saturation, and pod restarts.
- Backups and restore procedures have been tested for the selected data services.
