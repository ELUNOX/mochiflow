# Clarify doctor/freeze coherence and context freshness — Tasks

Implementation Summary: Add CLI guidance, context freshness warnings, `freeze --root`, and documentation/tests.
risk: elevated
Critical Stop Conditions:
- Stop if implementing the plan requires `doctor` to run `freeze --check`.
- Stop if context freshness needs deterministic regeneration or semantic prose diffing.
- Stop if `--root` cannot be added without changing existing `freeze [--check]` behavior.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03] Add doctor guidance and context command freshness warnings
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/doctor.rs`, `cli/crates/mochiflow-cli/src/main.rs`, `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: `doctor config` reports source-repo guidance where applicable, warns on stale context command references, stays quiet for valid CLI command, valid workflow command, and no command references, and tests cover allowlist drift against clap's actual top-level subcommands.
  - Stop: a reliable check requires running `freeze --check` from `doctor` or fully regenerating context prose.
- [x] T-002 [AC-04, AC-05, AC-06] Add explicit root handling for freeze
  - Depends on: none
  - Files: `cli/crates/mochiflow-cli/src/main.rs`, `cli/crates/mochiflow-core/src/freeze.rs`, `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: `mochiflow freeze --root <source-repo> [--check]` uses the explicit root, invalid roots fail before writes, no-root cwd fallback still works, and tests cover all three cases.
  - Stop: implementing `--root` requires interpreting global `--config` as a source-root input.
- [x] T-003 [AC-07] Update user-facing documentation
  - Depends on: T-001, T-002
  - Files: `README.md`, `docs/versioning.md`, `contracts/VERSIONING.md`, `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: docs add only the missing guidance for `doctor` versus `freeze --check` and the new `freeze --root` usage, avoid duplicating existing consumer drift/versioning explanations, and doc assertion tests cover the required phrases.
  - Stop: docs need to claim that `doctor` verifies source derived-file coherence.
- [ ] T-004 [AC-01, AC-02, AC-03, AC-04, AC-05, AC-06, AC-07] Run verification and update evidence
  - Depends on: T-001, T-002, T-003
  - Files: `.mochiflow/specs/doctor-freeze-coherence/spec.md`, `.mochiflow/specs/doctor-freeze-coherence/tasks.md`
  - Done: targeted tests and the configured `cli.default` verification pass, task checkboxes and AC Matrix evidence are updated with results.
  - Stop: default verification fails for reasons outside this plan.
