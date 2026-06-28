---
id: 2026-06-25-editing-engine-requires-freeze
date: 2026-06-25
area: [cli]
status: active
---
## Editing engine/ requires `mochiflow freeze` before the integrity gate passes (2026-06-25)

**Applies to:** dogfood builds that edit repo-root `engine/**` (docs, templates,
reference, agents).
**Signal:** `mochiflow freeze --check` (run by the `default` / `quick` verify
profiles and CI) reports `STALE: engine/MANIFEST.json` after editing any
`engine/` file, even though no Rust changed. NOTE: as of `manifest-test-isolation`
the functional `cargo test` suite no longer fails for this reason — the two
`freeze_*` resolution tests use an in-test tempdir fixture, so only the
`freeze --check` integrity gate is affected.
**Cause:** `freeze --check` compares `engine/MANIFEST.json` against `engine/`
contents; any engine edit invalidates the manifest hash until re-frozen.
**Guardrail:** After each `engine/` edit and before running `freeze --check` /
committing, run `mochiflow freeze` to regenerate `engine/MANIFEST.json`, and
stage the regenerated manifest with that task's commit. Per the constitution
dogfood rule, run `freeze` -> `upgrade --source engine` -> `adapter generate
--check` before final verification. Note: the vendored `.mochiflow/engine/` is
gitignored (synced by `upgrade`, not committed), and adapters that only reference
file paths (`AGENTS.md`, `.kiro/*`) stay byte-identical, so engine prose edits
usually leave no adapter diff to stage.
**Check:** `mochiflow freeze --check` reports "all derived files are up to date"
and the full `default` verification is green before close-out.
**Status:** Active (test-suite coupling resolved 2026-06-25 by
`manifest-test-isolation`; the integrity gate still requires a fresh manifest).
