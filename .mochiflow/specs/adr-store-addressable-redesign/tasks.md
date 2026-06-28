# Make the ADR store addressable and bounded — Tasks

Implementation Summary: split the ADR monoliths into addressable per-file records with a generated index, supersession lifecycle, deterministic lint, and a read-only retrieval command.
risk: elevated
Critical Stop Conditions:
- Never rewrite an ADR record body during migration — verbatim extraction only.
- Never stage any `INDEX.md` (gitignored derived cache).
- Do not add a legacy-monolith read path or a permanent `mochiflow adr migrate` command.

## Defaults

- Verification: surface `cli` `default` profile (`cargo test` + `cargo fmt --check` + `cargo clippy -D warnings` + `freeze --check`).
- Engine edits target the repo-root source `engine/` (the `.mochiflow/engine/` copy is gitignored / vendored). After editing `engine/`, run `mochiflow freeze`, then `mochiflow upgrade --source engine` to reinstall the vendored copy, then `mochiflow adapter generate --check` to confirm generated adapters match.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing.

## Tasks

- [x] T-001 [AC-01, AC-02, AC-10] Make `[adr]` config directory-rooted
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/config.rs`
  - Done: `RawAdr` resolves `decisions` / `pitfalls` as directory roots; accessors (`*_dir()` + `*_index()`) return each store's record directory and `INDEX.md` path; an absent/empty directory yields zero records with no monolith fallback; a value resolving to an existing file where a directory is expected returns a config error; unit tests cover resolution, the empty-store case, and the file-where-directory error.
  - Stop: if any consumer still expects a single monolith file, reconcile it here before proceeding.
- [x] T-002 [AC-08] Update config contract and refreeze
  - Depends on: T-001
  - Files:
    - `contracts/config.schema.json`
    - `contracts/contracts.lock`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: schema `adr` block documents the directory contract (presence + non-empty string per key); the file-vs-directory rejection is runtime config validation in `config.rs` (T-001), not a schema constraint; `contracts.lock` refrozen via `freeze`; conformance schema and golden-config tests updated and passing.
  - Stop: never hand-edit `contracts.lock`; regenerate with `freeze`. Do not claim the JSON schema can distinguish a file path from a directory path.
- [x] T-003 [AC-03] Generated gitignored INDEX and accept staging
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/adr.rs`
    - `cli/crates/mochiflow-core/src/ship.rs`
  - Done: each store's `INDEX.md` is rendered from record front-matter with a staleness check; `run_accept` stages the record directories and never `INDEX.md`, updating the `lifecycle_paths` / `allowed_ship_paths` call sites for the `decisions_path()` / `pitfalls_path()` -> `*_dir()` rename from T-001; tests assert staging includes the directories and excludes the index. The existing bare `INDEX.md` `.gitignore` pattern already matches `adr/**/INDEX.md`, so no new pattern is added.
  - Stop: if INDEX generation would need to be committed, revisit the derived-cache decision before coding.
- [x] T-004 [AC-04] Record model, front-matter, and supersession active-set
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/adr.rs`
  - Done: record parse (front-matter + body) for decisions and pitfalls; active-set computation excludes `superseded` / `deprecated`; `supersedes` marks the target superseded; cycles, dangling, and one-sided cross-refs are detected; path traversal in record names is rejected.
  - Stop: do not mutate record bodies to express supersession — status/links only.
- [x] T-005 [AC-05] `mochiflow adr lint` deterministic checks
  - Depends on: T-004
  - Files:
    - `cli/crates/mochiflow-core/src/adr.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
  - Done: `adr lint` reports dangling `superseded_by`, orphan, stale, missing cross-ref, schema violation, and INDEX staleness, classified into gating vs warning; non-zero exit on gating failures; no semantic-contradiction heuristic is added.
  - Stop: if a check cannot be decided mechanically, leave it out (review judgment), do not approximate.
- [x] T-006 [AC-05] Wire deterministic subset into doctor
  - Depends on: T-005
  - Files:
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: doctor gates on dangling / missing cross-ref / schema; orphan, stale, and INDEX freshness surface as non-blocking warnings; an absent or stale INDEX is regenerated rather than failing (consistent with the board `INDEX.md`); tests cover gating vs warning vs regenerate behavior.
  - Stop: do not make orphan / stale / INDEX freshness block the quality gate.
- [x] T-007 [AC-07] One-time verbatim migration of the repo monoliths
  - Depends on: T-003, T-004
  - Files:
    - `.mochiflow/adr/decisions/`
    - `.mochiflow/adr/pitfalls/`
    - `.mochiflow/config.toml`
    - deleted: `.mochiflow/adr/decisions.md`
    - deleted: `.mochiflow/adr/pitfalls.md`
  - Done: each monolith entry is extracted verbatim into a per-file record with front-matter (`id` / `date` / `area` / `spec` / `status`); record bodies are byte-identical to source entries; the `[adr].decisions` / `[adr].pitfalls` values in `.mochiflow/config.toml` are repointed to the record directories; monoliths deleted; delivered as a standalone reversible commit separate from the engine / contract changes.
  - Stop: never reword a migrated body; if an entry lacks a parseable date/slug, stop and ask rather than inventing one.
- [x] T-008 [AC-06] `mochiflow adr list | show | search`
  - Depends on: T-004
  - Files:
    - `cli/crates/mochiflow-core/src/adr.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: read-only `list/show/search` support `--kind` / `--area` / `--status` / `--spec`, default `--status active`, return headers for `list`/`search` and full body + lineage for `show`; CLI integration tests cover filters and output shape.
  - Stop: no embedding/index-search engine; front-matter / keyword derivation only.
- [x] T-009 [AC-09] Selective-load wording in discuss/plan
  - Depends on: T-004, T-008
  - Files:
    - `engine/commands/discuss.md`
    - `engine/commands/plan.md`
  - Done: both commands document loading `INDEX.md` first, then opening only records whose `area` intersects the spec `surfaces` and whose `status` is active; superseded/deprecated opened only when tracing lineage; engine-edit propagation per Defaults (freeze / upgrade --source engine / adapter generate --check).
  - Stop: keep current-state truth in code; do not turn ADR into a current-state source.
- [x] T-010 [AC-08] Fold and storage docs, adapter templates
  - Depends on: T-002, T-003
  - Files:
    - `engine/commands/open.md`
    - `engine/reference/git.md`
    - `engine/reference/authoring.md`
    - `engine/adapters/agents/AGENTS.md.tpl`
    - `engine/adapters/kiro/steering/mochiflow.md.tpl`
    - `engine/adapters/claude-code/CLAUDE.md.tpl`
    - `engine/adapters/copilot/copilot-instructions.md.tpl`
  - Done: open.md fold appends a per-file record (with a supersession step) and regenerates the gitignored INDEX; git.md fold/staging/gitignore reflect the directory store; authoring SSOT row updated; the on-demand and fold wording in all four adapter templates (agents, kiro, claude-code, copilot) reflects per-file directory records and supersession; adapters regenerated via Defaults propagation and `adapter generate --check` plus `freeze --check` pass.
  - Stop: regenerate adapters via the tooling; do not hand-edit generated adapter targets or the vendored `.mochiflow/engine/` copy.
- [x] T-011 [AC-02, AC-08] Reconcile remaining `[adr]` accessor consumers
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-core/src/detach.rs`
    - `cli/crates/mochiflow-core/src/adapter.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
  - Done: `mochiflow init` creates empty record directories (`adr/decisions/`, `adr/pitfalls/`) and writes directory-rooted `[adr]` values in the generated `config.toml`, with the ADR monolith stub body and the `is_living_spec_stub` ADR branch removed; `detach.rs` keep-default data handling, `config show` ADR lines, and the `adapter.rs` `adr.decisions` / `adr.pitfalls` placeholder map and its tests are updated for the directory contract; affected unit tests pass.
  - Stop: do not let `init` scaffold a monolith stub file; if removing the ADR stub branch affects unrelated `is_living_spec_stub` callers, reconcile them here.

The AC Verification Matrix lives in `spec.md` under `## Verification Plan / AC Matrix`.
