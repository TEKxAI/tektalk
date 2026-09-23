#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

command -v docker >/dev/null 2>&1 || {
  echo "Docker is required." >&2
  exit 1
}

if [ "${1:-}" = "--volumes" ]; then
  docker compose down --volumes --remove-orphans
  echo "Stopped TEKtalk and removed local data volumes."
else
  docker compose down --remove-orphans
  echo "Stopped TEKtalk. Local data volumes were preserved."
fi
