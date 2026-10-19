#!/usr/bin/env bash
set -euo pipefail

# Same reviewed scanner locally and in CI. Update version and digest together.
version=8.24.3
digest=9991e0b2903da4c8f6122b5c3186448b927a5da4deef1fe45271c3793f4ee29c
install_dir="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/tektalk-gitleaks"
mkdir -p "$install_dir"
curl --fail --silent --show-error --location --retry 3 \
  "https://github.com/gitleaks/gitleaks/releases/download/v${version}/gitleaks_${version}_linux_x64.tar.gz" \
  --output "$install_dir/gitleaks.tar.gz"
printf '%s  %s\n' "$digest" "$install_dir/gitleaks.tar.gz" | sha256sum --check --strict
tar --no-same-owner -xzf "$install_dir/gitleaks.tar.gz" -C "$install_dir" gitleaks
"$install_dir/gitleaks" version
if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
fi
