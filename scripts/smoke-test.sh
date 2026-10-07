#!/usr/bin/env sh
set -eu

base_url=${TEKTALK_BASE_URL:-http://localhost:8080}

[ "$(curl --fail --silent --show-error "$base_url/healthz")" = "ok" ]
[ "$(curl --fail --silent --show-error "$base_url/readyz")" = "ready" ]

plans=$(curl --fail --silent --show-error "$base_url/v1/commerce/plans?segment=user")
printf '%s' "$plans" | grep -q 'user.premium' || {
  echo "Commerce gateway did not return the user catalog." >&2; exit 1;
}

suffix=$(date +%s | tail -c 9)
phone="+849$suffix"
response=$(curl --fail --silent --show-error -H 'content-type: application/json' \
  -d "{\"phone\":\"$phone\",\"display_name\":\"Sandbox Learner\",\"password\":\"Learning1234\",\"security_question\":\"Sandbox test?\",\"security_answer\":\"yes\",\"device_name\":\"smoke-test\"}" \
  "$base_url/v1/auth/register")
token=$(printf '%s' "$response" | sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p')
[ -n "$token" ] || { echo "Registration did not return an access token." >&2; exit 1; }

subscription=$(curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $token" \
  -d '{"plan_code":"user.premium"}' "$base_url/v1/commerce/subscription")
printf '%s' "$subscription" | grep -q '"status":"active"' || { echo "Subscription activation failed." >&2; exit 1; }

entitlements=$(curl --fail --silent --show-error -H "authorization: Bearer $token" "$base_url/v1/commerce/entitlements")
printf '%s' "$entitlements" | grep -q 'ai.assistant.standard' || { echo "Entitlement resolution failed." >&2; exit 1; }

echo "Smoke test passed: readiness, registration, catalog, subscription and entitlements."
