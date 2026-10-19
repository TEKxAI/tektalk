# CI, security and release gates

## Secret scanning

The `security / secrets` job uses Gitleaks 8.24.3 with a verified release
archive SHA-256. Every trigger (pull request, push, manual, scheduled and
reusable release workflow) scans `--full-history --all` after a full checkout.
This catches secrets removed from the current tree but retained in Git history.
Redacted SARIF results are retained as the `gitleaks-results` artifact.

Two findings from the scheduled run on 2026-10-19 were reviewed directly in
their original commits. Both are Swift type declarations, not secret values:

| Commit | File, line | Reviewed source |
| --- | --- | --- |
| `71e6e72b9d2fab37d840025219701cd60292f260` | `clients/ios/TEKtalk/TEKProtocol.swift:3` | `Curve25519.KeyAgreement.PrivateKey` type |
| `50f2d4f6c4ea73d08dc1de6c76a4071a599ee398` | `clients/ios/ZChat/ZProtocol.swift:3` | `Curve25519.KeyAgreement.PrivateKey` type |

`.gitleaksignore` contains only their exact commit/path/rule/line fingerprints.
It does not exclude any whole file, commit, rule, or future key value. No key
rotation or history rewrite is needed for these two type declarations.

Run locally on Linux x64:

```sh
bash scripts/ci/install-gitleaks.sh
export PATH="${TMPDIR:-/tmp}/tektalk-gitleaks:$PATH"
python3 scripts/ci/test-secret-scan.py
gitleaks git --redact --log-opts="--full-history --all" .
```

The regression check validates the original source and the exact ignore list.
It also creates an isolated temporary Git repository with synthetic secrets at
the same paths and lines, deletes them in a later commit, and verifies they
still fail the history scan. Never add an exception to make a real leaked
credential pass: revoke/rotate it first and review remediation separately.

## Dependency audit

The dependency job calls `cargo audit` directly so a vulnerability returns a
nonzero exit status on every event, including schedules. This avoids the old
action's different schedule/check-report behavior and needs no write token.
The existing `RUSTSEC-2023-0071` exception is retained; this CI repair does not
claim to fix the underlying RustCrypto RSA timing advisory. It must be reviewed
before production use or removed when an upstream fix is available.

## CI and release

`ci / architecture` lints every Actions workflow with actionlint. The CI and
security workflows also support `workflow_call`. A `v*` tag runs both through
`release-images`; container build/push jobs depend on both succeeding for that
exact release commit. A failed gate prevents image publication. CI validates
Docker builds and the local Compose stack; the visual workflow separately
tests registration, login and chat on five platforms.

## Deployment

Deployment remains a manual choice of environment and immutable version tag or
full commit SHA. Configure `KUBECONFIG_B64` and `BASE_URL` in each GitHub
environment. The workflow validates inputs before accessing the cluster,
installs kubectl and Kustomize explicitly, renders the overlay, waits for both
deployments, and runs the API smoke test. Rollback is attempted only after the
rollout step has started, including if the post-deploy smoke test fails.

CI cannot prove access to a production cluster. Validate an actual rollout in
sandbox with the configured credentials before promoting a release. Do not
treat a green build as evidence that an environment was deployed.
