#!/usr/bin/env sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
port=${WEB_PORT:-5173}
command -v python3 >/dev/null 2>&1 || { echo "Python 3 is required for the Web development server." >&2; exit 1; }
echo "TEKtalk Web: http://localhost:$port/"
echo "API default: http://localhost:8080 (override with ?api=https://host)"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root/clients/web"
