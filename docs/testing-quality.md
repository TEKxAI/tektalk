# Testing and coverage policy

Every production module must include unit tests for its domain rules, success paths, boundary values and expected failures. Integration tests cover contracts between modules; they do not replace focused unit tests.

## CI quality gates

| Scope | Line coverage gate | Deliberate exclusions |
|---|---:|---|
| Rust workspace | 35% | generated contracts, `build.rs`, binary entry points |
| Android testable logic | 60% | generated Android classes, Compose shell and `MainActivity` |
| iOS testable logic | 60% | declarative `ContentView` and declaration-only plugin contract |

The initial Rust baseline is intentionally lower because it covers the complete workspace, including transport and service integration code. It is a floor, not a target. Coverage must not decrease in a pull request. Raise each threshold in five-percentage-point steps as the test suite grows, aiming first for 70% workspace coverage and 80% for domain modules.

Coverage exclusions must be structural and reviewable. Do not exclude a file merely because it is difficult to test. Authentication, authorization, session elevation, consent evaluation, message validation, serialization and cryptographic failure handling are never eligible for exclusion.

## Running locally

Install `cargo-llvm-cov`, then run:

```bash
cargo install cargo-llvm-cov --locked
./scripts/coverage/rust.sh
./scripts/coverage/android.sh
./scripts/coverage/ios.sh # macOS with Xcode and XcodeGen
```

Reports are written below `coverage/`. Thresholds can be raised locally with `RUST_COVERAGE_MINIMUM`, `ANDROID_COVERAGE_MINIMUM` (ratio, for example `0.70`) and `IOS_COVERAGE_MINIMUM` (ratio).

## Agentic test workflow

AI agents may accelerate test inventory, boundary-case generation and mutation review, but generated tests are accepted only when they assert externally meaningful behavior. Each change should follow this loop:

1. Identify changed decisions, invariants and error paths.
2. Ask an agent to propose a test matrix, including abuse and concurrency cases.
3. A developer reviews the matrix and removes implementation-coupled assertions.
4. Implement deterministic tests using fake clocks, seeded randomness and in-memory adapters where appropriate.
5. Run the relevant module tests and all three coverage gates.
6. Review uncovered branches; add meaningful cases rather than snapshot-only or assertion-free tests.

Coverage is a safety signal, not proof of correctness. Security-sensitive code additionally requires threat-model cases and manual review.
