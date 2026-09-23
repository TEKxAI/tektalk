#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

command -v docker >/dev/null 2>&1 || {
  echo "Docker is required. Install Docker Desktop or Docker Engine with Compose v2." >&2
  exit 1
}
docker info >/dev/null 2>&1 || {
  echo "Docker daemon is not running. Start Docker and retry." >&2
  exit 1
}
docker compose version >/dev/null 2>&1 || {
  echo "Docker Compose v2 is required." >&2
  exit 1
}
command -v curl >/dev/null 2>&1 || {
  echo "curl is required for the local smoke test." >&2
  exit 1
}

if [ ! -f .env ]; then
  command -v openssl >/dev/null 2>&1 || {
    echo "OpenSSL is required to generate local development secrets." >&2
    exit 1
  }
  jwt_secret=$(openssl rand -hex 32)
  otp_secret=$(openssl rand -hex 32)
  sed \
    -e "s|replace-with-at-least-32-random-bytes|$jwt_secret|" \
    -e "s|replace-with-an-independent-random-secret|$otp_secret|" \
    .env.example > .env
  chmod 600 .env
  echo "Created .env with random local-only secrets."
fi

echo "Building and starting TEKtalk local stack..."
docker compose up --build -d --wait --wait-timeout 300

echo "Running API smoke test..."
"$root/scripts/smoke-test.sh"

echo
echo "TEKtalk is ready:"
echo "  API/health: http://localhost:8080/healthz"
echo "  PostgreSQL: localhost:5432"
echo "  Redis:      localhost:6379"
echo "  ScyllaDB:   localhost:9042"
echo "  Kafka:      localhost:19092"
echo "  Redpanda:   http://localhost:9644"
echo "Use 'make logs' to follow server logs and 'make local-down' to stop."
