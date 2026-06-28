---
id: 2026-06-25-manifest-test-isolation
date: 2026-06-25
area: [cli]
spec: manifest-test-isolation
status: active
---
## 2026-06-25 — manifest-test-isolation: functional freeze tests decoupled from committed MANIFEST

**Decision:** The two `freeze` resolution integration tests
(`freeze_root_check_uses_explicit_source_repo_from_other_cwd`,
`freeze_without_root_keeps_cwd_upward_resolution`) build an in-test tempdir
source-repo fixture and freeze it in-test, asserting `--root` and cwd-upward
resolution against that fixture rather than the committed repository
`engine/MANIFEST.json`. `cargo run -- freeze --check` (the `default` verify
profile + the existing CI step) remains the single authoritative integrity gate,
and the `quick` verify profile gains the same working-tree `freeze --check`.

**Why:** Editing any `engine/**` file invalidated `engine/MANIFEST.json`, and the
two tests asserted manifest freshness against the real repo, so `cargo test`
failed with `STALE` until a manual `mochiflow freeze` — a failure for the wrong
reason (uncommitted manifest, not broken logic). Investigation showed the
version-gate hash covers only `contracts/*.json` + `tests/conformance/golden/**`,
so the coupling was specifically the MANIFEST-freshness `freeze --check` path in
those two tests, not the seed's assumed "7 tests".

**Rejected:** `#[ignore]` on the failing tests (leaves functional tests coupled;
resurfaces whenever CI runs `--ignored`); a cargo feature gate for integrity
tests (more machinery than the existing CLI/CI gate needs); swapping `default` to
the installed `mochiflow freeze --check` (runs a possibly-stale installed binary
in the very repo that builds it — an integrity-gate regression).

**Consequence:** `cli/crates/mochiflow-cli/tests/cli.rs` builds the fixture inline
via a local `setup_freeze_fixture`; the `mochiflow-core` `setup_fixture` test
helper stays a private `#[cfg(test)]` helper (not shared, no visibility change).
`.mochiflow/config.toml` `quick` profile is now test + `freeze --check`. The
`default` profile, the version-gate hash composition, and the CI step are
unchanged. Risk classified `standard` (reversible, single surface, integration
none), so no `design.md`.
