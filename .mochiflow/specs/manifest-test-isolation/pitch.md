# Isolate MANIFEST integrity check from functional conformance tests

## Problem

Editing any `engine/**` file regenerates `engine/MANIFEST.json`, and the CLI test
suite contains tests that run `freeze --check` against the real repository. Those
tests fail with `STALE: engine/MANIFEST.json` until the developer runs
`mochiflow freeze`. This forces a manual freeze (and a MANIFEST-bundled commit)
on every task that touches engine files, inflating `tasks.md` with freeze steps
and producing test failures for the "wrong reason" (stale hash, not broken
logic). It conflates two questions: "does the feature work?" (functional) and
"is the frozen manifest committed?" (integrity). Confirmed live during the
`qa-attack-matrix` build. Origin: backlog seed `manifest-test-isolation`
(source: conversation, from `ac-matrix-token-normalization`).

## Appetite

Small, test-architecture-focused refactor plus a one-line config addition. No
change to the integrity guarantee itself, no version-gate change, no new CI step.

## Solution

Decouple `cargo test` from the committed repository MANIFEST so functional tests
never fail on stale-hash drift, while keeping a single authoritative integrity
gate:

- Investigation findings that shape the design:
  - The version-gate hash covers only `contracts/*.json` + `tests/conformance/golden/**`
    (`freeze.rs compute_contracts_hash`), so engine doc/template edits do not
    affect it — only `engine/MANIFEST.json` freshness. The coupling is therefore
    narrower than the seed's "7 tests"; it is the `freeze --check` (MANIFEST
    freshness) path.
  - CI already runs `cargo run --manifest-path cli/Cargo.toml -- freeze --check`
    as an explicit step, and the `default` verify profile runs the same. The
    integrity gate already lives outside the Rust test suite.
- Refactor the repo-MANIFEST-freshness-dependent tests
  (`freeze_without_root_keeps_cwd_upward_resolution`,
  `freeze_root_check_uses_explicit_source_repo_from_other_cwd`, and any sibling
  found during build) to build a tempdir fixture engine and freeze it in-test,
  so they assert `freeze`/`--root`/cwd-resolution behavior without depending on
  the committed repository MANIFEST. The exact test set is confirmed during plan
  / build by reading the suite.
- Keep `mochiflow freeze --check` (via `default` = `cargo run -- freeze --check`
  and the existing CI step) as the single integrity gate.
- Add `mochiflow freeze --check` to the `quick` verify profile in
  `.mochiflow/config.toml` for a fast intermediate loop.

## Rabbit Holes

- Weakening the integrity gate: do not remove or `#[ignore]` a check without
  confirming `cargo run -- freeze --check` still runs in both CI and `default`.
- Over-reach into the version-gate hash composition (out of scope; unaffected by
  engine doc edits).
- Editing the vendored `.mochiflow/engine/` copy instead of repo-root `engine/`;
  after engine edits run the dogfood steps (`freeze` -> `upgrade --source engine`
  -> `adapter generate --check`).
- Merging in the freeze-module code-quality work (`freeze-hardening` seed) — keep
  separate.

## No-gos

- No change to the `default` verify profile (stays `cargo run -- freeze --check`;
  the source repo must validate with the working-tree freeze logic, not the
  installed binary).
- No version-gate hash change.
- No new CI step (CI already runs `freeze --check`).
- No pre-commit hook or engine-docs "freeze first" note (the isolation removes the
  root cause).
- No `freeze.rs` error-type / format / visibility changes (separate seed).

## Alternatives Considered

- Minimal `#[ignore]` on the failing tests only: leaves functional tests coupled
  to repository MANIFEST freshness, so the coupling resurfaces whenever CI runs
  `--ignored`. Fixture isolation is the root-cause fix.
- A cargo feature gate for integrity tests: more machinery than `#[ignore]` +
  the existing CLI/CI gate needs.
- Swapping `default` to `mochiflow freeze --check`: uses the installed (possibly
  stale) binary in the very repo that builds it — an integrity-gate regression.

## Open Questions

- None - ready for plan.
