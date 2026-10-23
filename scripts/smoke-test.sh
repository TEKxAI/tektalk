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

peer_phone="+848$suffix"
peer_response=$(curl --fail --silent --show-error -H 'content-type: application/json' \
  -d "{\"phone\":\"$peer_phone\",\"display_name\":\"Secret Peer\",\"password\":\"Learning1234\",\"security_question\":\"Sandbox test?\",\"security_answer\":\"yes\",\"device_name\":\"smoke-peer\"}" \
  "$base_url/v1/auth/register")
peer_token=$(printf '%s' "$peer_response" | sed -n 's/.*"access_token":"\([^"]*\)".*/\1/p')
peer_id=$(printf '%s' "$peer_response" | sed -n 's/.*"user_id":"\([^"]*\)".*/\1/p')
conversation_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
client_message_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
bundle=$(python3 -c 'import json; print(json.dumps({"suite":"TEKTALK_X25519_MLKEM768_AES256GCMSIV_SHA256","x25519_public":[1]*32,"ml_kem_768_public":[2]*1184}))')
curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $token" -d "$bundle" "$base_url/v1/secret/devices/register" >/dev/null
curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $peer_token" -d "$bundle" "$base_url/v1/secret/devices/register" >/dev/null
policy=$(curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $token" \
  -d "{\"conversation_id\":\"$conversation_id\",\"kind\":\"one_to_one\",\"member_ids\":[\"$peer_id\"]}" "$base_url/v1/secret/conversations/enable")
printf '%s' "$policy" | grep -q 'TEKTALK_X25519_MLKEM768_AES256GCMSIV_SHA256' || { echo "Secret Chat policy did not require the hybrid suite." >&2; exit 1; }
ciphertext=$(python3 -c 'import json; print(json.dumps([7]*32))')
secret_message=$(curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $token" \
  -d "{\"conversation_id\":\"$conversation_id\",\"client_message_id\":\"$client_message_id\",\"epoch\":1,\"counter\":0,\"nonce\":[1,2,3,4,5,6,7,8,9,10,11,12],\"ciphertext\":$ciphertext}" "$base_url/v1/secret/messages/send")
printf '%s' "$secret_message" | grep -q '"ciphertext"' || { echo "Secret Chat ciphertext transport failed." >&2; exit 1; }
if printf '%s' "$secret_message" | grep -q '"body"'; then echo "Secret Chat response leaked a plaintext field." >&2; exit 1; fi

subscription=$(curl --fail --silent --show-error -H 'content-type: application/json' -H "authorization: Bearer $token" \
  -d '{"plan_code":"user.premium"}' "$base_url/v1/commerce/subscription")
printf '%s' "$subscription" | grep -q '"status":"active"' || { echo "Subscription activation failed." >&2; exit 1; }

entitlements=$(curl --fail --silent --show-error -H "authorization: Bearer $token" "$base_url/v1/commerce/entitlements")
printf '%s' "$entitlements" | grep -q 'ai.assistant.standard' || { echo "Entitlement resolution failed." >&2; exit 1; }

echo "Smoke test passed: readiness, auth, Secret Chat ciphertext, catalog, subscription and entitlements."
