#!/usr/bin/env bash
set -euo pipefail

# Validate before any cluster mutation. Values are passed through environment
# variables, never interpolated into shell source from workflow inputs.
if [[ ! "${IMAGE_TAG:-}" =~ ^(v[0-9][A-Za-z0-9_.-]*|[0-9a-f]{40})$ ]] || (( ${#IMAGE_TAG} > 128 )); then
  echo 'image_tag must be a versioned release tag (v1.2.3) or a full 40-character commit SHA.' >&2
  exit 1
fi
if [[ ! "${BASE_URL:-}" =~ ^https?://[^[:space:]]+$ ]]; then
  echo 'Configure BASE_URL for the selected GitHub environment before deploying.' >&2
  exit 1
fi
if [[ -z "${KUBECONFIG_B64:-}" ]]; then
  echo 'Configure KUBECONFIG_B64 for the selected GitHub environment before deploying.' >&2
  exit 1
fi
printf '%s' "$KUBECONFIG_B64" | base64 --decode >/dev/null
