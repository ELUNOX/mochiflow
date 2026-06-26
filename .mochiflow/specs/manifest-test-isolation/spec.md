# Isolate MANIFEST integrity check from functional conformance tests

## Background and Design Rationale

- Problem / why now: Editing any `engine/**` file regenerates `engine/MANIFEST.json`.
  Two CLI integration tests run `freeze --check` against the **committed
  repository** and assert "all derived files are up to date", so they fail with
  stale-hash drift until the developer runs `mochiflow freeze`. This conflates
  "does the feature work?" (functional) with "is the frozen manifest committed?"
  (integrity), forcing a manual freeze (and a MANIFEST-bundled commit) on every
  task that touches engine files and producing failures for the wrong reason.
- Investigation findings that shape the design:
  - The version-gate hash (`freeze.rs compute_contracts_hash`) covers only
    `contracts/*.json` + `tests/conformance/golden/**`, so engine doc/template
    edits do not affect it. The coupling is narrower than the seed's "7 tests":
    it is the `freeze --check` (MANIFEST-freshness) path.
  - The exact coupled set, confirmed by reading the suite, is **two** integration
    tests in `cli/crates/mochiflow-cli/tests/cli.rs` that assert
    `freeze: all derived files are up to date` against the real `repo_root()`:
    `freeze_root_check_uses_explicit_source_repo_from_other_cwd` and
    `freeze_without_root_keeps_cwd_upward_resolution`. Sibling freeze tests
    (`freeze_check_detects_staleness`, `freeze_write_idempotent_in_fixture`,
    `freeze_version_triple_mismatch_fails_gate`) already use a tempdir fixture
    (`setup_fixture`) and are not coupled; `compute_contracts_hash_matches_committed_lock`
    and `schema_manifest_accepts_real_manifest` exercise the version-gate hash /
    schema shape, not MANIFEST freshness, and are out of scope.
  - CI already runs `cargo run --manifest-path cli/Cargo.toml -- freeze --check`
    (`.github/workflows/ci.yml`), and the `default` verify profile runs the same.
    The integrity gate already lives outside the Rust test suite.
- Chosen approach: rebuild the two coupled integration tests to construct an
  in-test tempdir fixture source repo, freeze it in-test, then assert the
  `--root` / cwd-upward resolution behavior against that fixture — never the
  committed repository MANIFEST. Keep `mochiflow freeze --check`
  (`default` profile + existing CI step) as the single authoritative integrity
  gate, and add `mochiflow freeze --check` to the `quick` verify profile for a
  fast intermediate loop.
- Rejected alternatives: `#[ignore]` on the failing tests (leaves functional
  tests coupled; resurfaces under `--ignored`); a cargo feature gate for
  integrity tests (more machinery than the existing CLI/CI gate needs); swapping
  `default` to `mochiflow freeze --check` (uses the installed, possibly stale
  binary in the very repo that builds it — an integrity-gate regression).
- Origin: backlog seed `manifest-test-isolation` (source: conversation, from
  `ac-matrix-token-normalization`); confirmed live during the `qa-attack-matrix`
  build.

## User Story

As a mochiflow maintainer editing `engine/**` files, I want functional `freeze`
tests to pass without first running `mochiflow freeze`, so that test failures
signal broken logic rather than an uncommitted manifest, while a single integrity
gate still catches a stale committed manifest.

## Scope

- In:
  - Refactor the two MANIFEST-freshness-coupled integration tests in
    `cli/crates/mochiflow-cli/tests/cli.rs` to a tempdir fixture engine.
  - Add `mochiflow freeze --check` to the `quick` verify profile in
    `.mochiflow/config.toml`.
- Out:
  - The version-gate hash composition (`compute_contracts_hash`) and its tests.
  - The `default` verify profile command and the CI `freeze --check` step.
  - `freeze.rs` error-type / format / visibility changes (separate
    `freeze-hardening` seed).
  - Any pre-commit hook or engine-docs "freeze first" note.

## Edge Cases

- The fixture must produce a manifest that `freeze --check` reports as fresh
  immediately after the in-test freeze (idempotent freeze), so the assertion is
  deterministic and independent of the committed tree.
- Cwd-upward resolution must be exercised from a **subdirectory** of the fixture
  root (mirroring the original `cli/crates/mochiflow-core` subdir case) so the
  resolution semantics are still covered.
- `--root` resolution must be exercised from an **unrelated** cwd (a separate
  tempdir) so the explicit-root path is still covered.

## Acceptance Criteria (EARS)

- AC-01: WHEN `cargo test --manifest-path cli/Cargo.toml` runs against a working
  tree whose committed `engine/MANIFEST.json` is stale relative to `engine/**`,
  THE SYSTEM SHALL pass every freeze test without any test failing because of
  committed-MANIFEST staleness.
- AC-02: THE SYSTEM SHALL verify both `freeze --root <source> --check` (from an
  unrelated cwd) and cwd-upward `freeze --check` (from a fixture subdirectory)
  resolution behavior using an in-test tempdir fixture engine, and SHALL NOT
  reference the committed repository MANIFEST in those two tests.
- AC-03: WHERE the `quick` verify profile is selected, THE SYSTEM SHALL execute
  `mochiflow freeze --check` in addition to the existing `cargo test` command.
- AC-04: THE SYSTEM SHALL keep the `default` verify profile command and the CI
  `freeze --check` step unchanged, so `mochiflow freeze --check` remains the
  single authoritative integrity gate.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1 | cli | Automated | Edit an `engine/**` doc (or otherwise make committed `engine/MANIFEST.json` stale), then run `cargo test --manifest-path cli/Cargo.toml`. | All freeze tests pass; no failure mentions `STALE` / committed MANIFEST. |
| QA-02 | P7 | cli | Automated | Inspect the two refactored tests; confirm they build a tempdir fixture + freeze it in-test and no longer assert freshness via `repo_root()`. Run them with a deliberately stale committed MANIFEST. | Tests contain no `repo_root()`-based freshness assertion and pass regardless of committed MANIFEST state, while still asserting `--root` and cwd-upward resolution. |
| QA-03 | P3 | cli | Automated | Attempt to bypass the integrity gate: corrupt the committed `engine/MANIFEST.json`, then run `cargo run --manifest-path cli/Cargo.toml -- freeze --check`. | `freeze --check` exits non-zero and reports the stale file; the gate is not weakened by the refactor. |
| QA-04 | P6 | cli | Automated | Run the full `default` verify profile; confirm sibling freeze tests (`freeze_check_detects_staleness`, `freeze_write_idempotent_in_fixture`, `freeze_version_triple_mismatch_fails_gate`, `freeze_root_invalid_path_fails_before_writing`) and the version-gate test still pass. | Entire suite is green; no nearby behavior regressed. |
| QA-05 | P4 | cli | Automated | After running the refactored tests, inspect the committed `engine/MANIFEST.json` for modification. | Committed `engine/MANIFEST.json` is unchanged; tests write only inside their tempdir fixtures. |
| QA-06 | P2 | cli | Automated | N/A check: no interactive / keyboard / large-input surface is introduced. | N/A: test-architecture + config change only, no interactive surface. |
| QA-07 | P5 | cli | Automated | N/A check: no persisted data format or migration is involved. | N/A: no stored data or schema migration; only test fixtures and a verify-profile string change. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix (in `tasks.md`) with a
  done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
