# Getting started

This guide turns the repository into a repeatable learning environment. It assumes no TEK-specific infrastructure and keeps every required value configurable.

## 1. Prerequisites

- Git
- Docker Engine 24+ with Docker Compose v2
- Rust stable when running the server outside Docker
- Android Studio with JDK 17 for the Android client
- Xcode 16+ and XcodeGen for the iOS client

Verify the minimum toolchain:

```bash
git --version
docker version
docker compose version
```

## 2. Configure the project

```bash
git clone https://github.com/TEKxAI/tektalk.git
cd tektalk
cp .env.example .env
```

Replace both development secrets in `.env`. Each value must contain at least 32 bytes:

```bash
openssl rand -hex 32
openssl rand -hex 32
```

Use the first output for `JWT_SECRET` and the second for `OTP_HMAC_SECRET`. Keep `OTP_DEV_ECHO=true` only for local learning; it exposes OTP values in development responses.

## 3. Run the complete backend

The simplest path generates local secrets, builds the Rust server, starts all dependencies, waits for health checks, and registers a smoke-test account:

```bash
make local-up
```

Follow logs with `make logs`. Re-run verification with `make smoke`. Stop the stack with `make local-down`. To also discard local database state, explicitly run `make local-reset`.

## 4. Run the server from Rust

This is useful when studying or debugging server code:

```bash
docker compose up -d postgres redis scylla redpanda
set -a
. ./.env
set +a
cargo run -p chat-server
```

The SQL migration is mounted into PostgreSQL and executes the first time its volume is created. If a schema changes during experimentation, use a new migration or recreate only the disposable local volume.

## 5. Android client

Open `clients/android` in Android Studio and run the `app` configuration on an emulator. The debug build points to `http://10.0.2.2:8080`, which maps the Android emulator to the host machine.

Command-line build:

```bash
cd clients/android
gradle :app:assembleDebug
```

For a physical device, set `API_BASE_URL` to a reachable HTTPS endpoint and use a trusted development certificate. Do not enable cleartext traffic for production deployments.

## 6. iOS client

Generate the Xcode project and open it:

```bash
cd clients/ios
xcodegen generate
open TEKtalk.xcodeproj
```

Use `http://127.0.0.1:8080` for the simulator. A physical device requires a reachable HTTPS endpoint and an appropriate signing team.

## 7. Verification

Run the same essential checks used by CI:

```bash
./scripts/check_layout.sh
cargo clippy --workspace --all-targets
cargo test --workspace
docker build -t tektalk-server:local .
cd clients/android && gradle :app:assembleDebug
```

Expected result: the layout guard reports success, Rust tests pass, the Android APK builds, and the server image is created.

## 8. Learning path

1. Read `docs/architecture.md` for service boundaries.
2. Trace HTTPS authentication using `docs/auth-flows.md` and `docs/api.md`.
3. Study `protocol/mtproto-2.0.md` together with `server/src/mtproto.rs`.
4. Compare Android and iOS protocol adapters.
5. Use `docs/telegram-engineering.md` to study which Telegram patterns are adopted, adapted, or intentionally excluded.
6. Deploy the template with `docs/deployment.md`.

Never reuse development secrets, OTP echo mode, test credentials, or local HTTP endpoints in a public environment.
