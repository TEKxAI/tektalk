#!/usr/bin/env bash
set -euo pipefail

minimum="${RUST_COVERAGE_MINIMUM:-35}"
output="${COVERAGE_OUTPUT_DIR:-coverage}/rust"
mkdir -p "$output"

cargo llvm-cov --workspace --all-targets \
  --ignore-filename-regex '(/build\.rs$|/src/bin/|/target/)' \
  --lcov --output-path "$output/lcov.info" \
  --fail-under-lines "$minimum"

cargo llvm-cov report --workspace \
  --ignore-filename-regex '(/build\.rs$|/src/bin/|/target/)' \
  --summary-only | tee "$output/summary.txt"
