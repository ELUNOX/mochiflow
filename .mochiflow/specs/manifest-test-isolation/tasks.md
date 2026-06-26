# Isolate MANIFEST integrity check from functional conformance tests — Tasks

Implementation Summary: Decouple the two MANIFEST-freshness CLI tests from the committed repo via a tempdir fixture, and add `freeze --check` to the `quick` profile.
risk: standard
Critical Stop Conditions:
- A refactored test still depends on the committed `engine/MANIFEST.json` freshness.
- The `default` verify profile or the CI `freeze --check` step would change (integrity-gate regression).

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01, AC-02] Refactor the two MANIFEST-freshness-coupled tests to an in-test tempdir fixture
  - Depends on: none
  - Files: `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: `freeze_root_check_uses_explicit_source_repo_from_other_cwd` and `freeze_without_root_keeps_cwd_upward_resolution` build a tempdir fixture source repo (cli/Cargo.toml + engine/ + contracts/ + tests/conformance/golden/), freeze it in-test, then assert `--root <fixture> --check` (from an unrelated cwd) and cwd-upward `--check` (from a fixture subdirectory) both report "all derived files are up to date"; neither test asserts freshness via `repo_root()`. The two tests pass even when the committed `engine/MANIFEST.json` is stale.
  - Stop: cwd-upward or `--root` resolution semantics can no longer be exercised against the fixture without touching `freeze.rs` resolution logic (out of scope) — stop and route to plan.
- [ ] T-002 [AC-03] Add `mochiflow freeze --check` to the `quick` verify profile
  - Depends on: none
  - Files: `.mochiflow/config.toml`
  - Done: `[surfaces.cli.verify].quick` runs the existing `cargo test --manifest-path cli/Cargo.toml` followed by `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; `mochiflow config show` reflects the updated `quick` profile.
  - Stop: the change would require touching the `default` profile string — stop (out of scope).
- [ ] T-003 [AC-04] Confirm integrity gate and out-of-scope surfaces are unchanged
  - Depends on: T-001, T-002
  - Files: `.mochiflow/config.toml`, `.github/workflows/ci.yml`
  - Done: `[surfaces.cli.verify].default` still ends with `cargo run --manifest-path cli/Cargo.toml -- freeze --check` and the CI workflow still runs `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; corrupting the committed `engine/MANIFEST.json` makes `cargo run -- freeze --check` fail (gate intact); version-gate hash code/tests untouched.
  - Stop: the gate no longer fails on a genuinely stale committed MANIFEST — stop and investigate the regression.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01 (`cargo test` with stale committed MANIFEST) | `cli/crates/mochiflow-cli/tests/cli.rs` | UNVERIFIED | | |
| AC-02 | cli | automated | QA-02, QA-05 (`cargo test --manifest-path cli/Cargo.toml`) | `cli/crates/mochiflow-cli/tests/cli.rs` | UNVERIFIED | | |
| AC-03 | cli | automated | QA-04 (`mochiflow config show` + `quick` profile run) | `.mochiflow/config.toml` | UNVERIFIED | | |
| AC-04 | cli | automated | QA-03 (corrupt MANIFEST → `freeze --check` fails); CI step + `default` profile inspection | `.mochiflow/config.toml`, `.github/workflows/ci.yml` | UNVERIFIED | | |
