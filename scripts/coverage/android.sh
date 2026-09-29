#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
output="${COVERAGE_OUTPUT_DIR:-$root/coverage}/android"
mkdir -p "$output"

gradle -p "$root/clients/android" \
  -I "$root/scripts/coverage/android-coverage.gradle" \
  testDebugUnitTest jacocoDebugReport jacocoDebugCoverageVerification

cp "$root/clients/android/app/build/reports/jacoco/debug/jacoco.xml" "$output/jacoco.xml"
