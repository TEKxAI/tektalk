#!/usr/bin/env sh
set -eu

base_url=${TEKTALK_BASE_URL:-http://localhost:8080}

health=$(curl --fail --silent --show-error "$base_url/healthz")
[ "$health" = "ok" ] || {
  echo "Unexpected health response: $health" >&2
  exit 1
}

# The suffix makes repeated smoke tests independent while remaining valid E.164.
suffix=$(date +%s | tail -c 9)
phone="+849$suffix"
response=$(curl --fail --silent --show-error \
  -H 'content-type: application/json' \
  -d "{\"phone\":\"$phone\",\"display_name\":\"Local Learner\",\"password\":\"Learning1234\",\"security_question\":\"Local test?\",\"security_answer\":\"yes\",\"device_name\":\"smoke-test\"}" \
  "$base_url/v1/auth/register")

printf '%s' "$response" | grep -q '"access_token"' || {
  echo "Registration smoke test did not return an access token." >&2
  exit 1
}

echo "Smoke test passed: health check and account registration."
