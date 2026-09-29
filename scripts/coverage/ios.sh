#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
output="${COVERAGE_OUTPUT_DIR:-$root/coverage}/ios"
result="$output/TEKtalk.xcresult"
mkdir -p "$output"
rm -rf "$result"

cd "$root/clients/ios"
xcodegen generate
xcodebuild test \
  -project TEKtalk.xcodeproj \
  -scheme TEKtalk \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  -enableCodeCoverage YES \
  -resultBundlePath "$result" \
  CODE_SIGNING_ALLOWED=NO

xcrun xccov view --report --json "$result" > "$output/xccov.json"
python3 "$root/scripts/coverage/ios.py" < "$output/xccov.json"
