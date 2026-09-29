#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
output="${COVERAGE_OUTPUT_DIR:-$root/coverage}/ios"
result="$output/TEKtalk.xcresult"
mkdir -p "$output"
rm -rf "$result"

cd "$root/clients/ios"
xcodegen generate
device_id="$(xcrun simctl list devices available -j | python3 -c 'import json,sys; data=json.load(sys.stdin); print(next(d["udid"] for devices in data["devices"].values() for d in devices if d["name"].startswith("iPhone")))')"
xcodebuild test \
  -project TEKtalk.xcodeproj \
  -scheme TEKtalk \
  -destination "platform=iOS Simulator,id=$device_id" \
  -enableCodeCoverage YES \
  -resultBundlePath "$result" \
  CODE_SIGNING_ALLOWED=NO

xcrun xccov view --report --json "$result" > "$output/xccov.json"
python3 "$root/scripts/coverage/ios.py" < "$output/xccov.json"
