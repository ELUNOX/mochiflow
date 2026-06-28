# Make the ADR store addressable and bounded

## Background and Design Rationale

The ADR store (`[adr].decisions` / `[adr].pitfalls`) is two single append-only
monoliths. `decisions.md` is ~424 lines / 15 dated entries and grows by one
entry per delivered spec via the `open` fold; `pitfalls.md` is ~152 lines / 8
entries. Because each file is read whole, consulting one past *why* costs
`O(total history)`. ADR is on-demand, so that cost lands during discuss / plan /
review — exactly when context budget matters. The defect is that retrieval is
all-or-nothing: the store is not addressable or filterable.

This redesign makes the store per-file, addressable, and bounded so reads become
`O(relevant active records)`. It mirrors `post-build-pr-close-flow`, which
replaced a committed growing board with a derived, bounded view.

Key decisions (full rationale and rejected alternatives in `design.md`):

- One immutable record per decision/pitfall under a directory; a generated
  `INDEX.md` is the content catalog. No synthesis/"current rationale" page —
  the active view is derived by filtering `status: active`, not maintained as a
  second prose artifact.
- `INDEX.md` is gitignored and regenerated (consistent with the existing board
  `INDEX.md`); retrieval derives its view from record front-matter at call time,
  so the index is a cache, never the only read path.
- Hard cut, no backward compatibility: the project has no external users, so
  there is no legacy-monolith read path and no permanent migration command. The
  existing monoliths are split once, verbatim, in a standalone reversible commit.
- `area` defaults to the spec's `surfaces`, assigned while writing the fold (no
  dedicated prompt); ADR lint is deterministic-structural only (semantic
  contradiction detection is out of scope, left to review judgment).

Origin: backlog seed `adr-store-addressable-redesign` (source: conversation,
from `post-build-pr-close-flow` close-out dogfooding).

## User Story

As a MochiFlow author running discuss / plan / review, I want the ADR store to
be per-file, filterable by area and status, and navigable through a generated
index and a read-only retrieval command, so that consulting prior rationale
loads only the relevant active records instead of the entire history.

## Scope

- In:
  - `[adr]` config contract change to directory-rooted stores (`config.rs`,
    `contracts/config.schema.json`, `contracts.lock` refreeze, conformance).
  - Per-file record layout, front-matter schema, and supersession lifecycle.
  - Generated, gitignored per-store `INDEX.md` and its staleness check.
  - `mochiflow adr <list | show | search>` and `mochiflow adr lint`; doctor
    wiring of the deterministic subset.
  - `mochiflow accept` staging of the record directories (never `INDEX.md`).
  - Engine doc updates: `commands/open.md` fold step, `reference/git.md` fold /
    staging / gitignore, `reference/authoring.md` SSOT row, discuss/plan
    selective-load wording, adapter templates and regenerated adapters.
  - One-time verbatim migration of the existing monoliths in this repo.
- Out:
  - Semantic contradiction detection (review judgment, not CLI lint).
  - A maintained "current rationale" synthesis page.
  - Backward-compatibility / legacy-monolith read code and any permanent
    `mochiflow adr migrate` command.
  - Embedding-based search.
  - Refreshing the `[context]` layer (separate onboard / refresh-context work).

## Edge Cases

- Empty or absent record directory: treated as zero records, never a fallback to
  a monolith.
- `supersedes:` pointing at a non-existent id, or a `superseded_by` with no
  reciprocal `supersedes:` (dangling / one-sided cross-ref).
- Supersession cycle (A supersedes B, B supersedes A).
- Record filename / slug containing path separators or traversal (`../`).
- Two concurrent feature branches both adding records: record files do not
  collide (distinct slugs); `INDEX.md` is never committed, so no merge conflict.
- A monolith entry that lacks a parseable date or slug during migration (stop
  and ask; never invent identity).
- An `[adr]` config value resolving to an existing file where a record directory
  is expected (config error, not a silent empty store).
- Unknown `area` value or missing required front-matter key.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL resolve `[adr].decisions` and `[adr].pitfalls` as
  directory roots and expose the record-directory path and the generated
  `INDEX.md` path for each store.
- AC-02: WHEN an ADR record directory is absent or empty, THE SYSTEM SHALL treat
  the store as zero records and SHALL NOT read any monolithic `decisions.md` /
  `pitfalls.md` fallback.
- AC-03: THE SYSTEM SHALL generate each store's `INDEX.md` from record
  front-matter, and WHEN `mochiflow accept` stages the ADR fold, THE SYSTEM
  SHALL stage the record directories and SHALL NOT stage any `INDEX.md`.
- AC-04: WHERE a decision record declares `supersedes: <id>`, THE SYSTEM SHALL
  exclude the referenced record from the active set, and IF the reciprocal
  `superseded_by` is missing THEN `mochiflow adr lint` SHALL report it.
- AC-05: THE SYSTEM SHALL provide `mochiflow adr lint` performing only
  deterministic structural checks (dangling `superseded_by`, orphan, stale,
  missing cross-ref, front-matter schema, INDEX freshness), and `doctor` SHALL
  gate on dangling / missing cross-ref / schema WHILE orphan, stale, and INDEX
  freshness remain non-blocking warnings; WHEN a store's `INDEX.md` is absent or
  stale, `doctor` SHALL regenerate it rather than fail (consistent with the
  board `INDEX.md` treatment).
- AC-06: THE SYSTEM SHALL provide read-only `mochiflow adr list | show | search`
  supporting `--kind decisions|pitfalls`, `--area`, `--status`, and `--spec`,
  defaulting to `status: active`, returning headers for `list` / `search` and
  the full body plus supersession lineage for `show`. WHERE `search` runs, THE
  SYSTEM SHALL match over only the filtered, default-active record set so
  retrieval stays bounded; scanning superseded / deprecated history requires an
  explicit `--status all`.
- AC-07: THE SYSTEM SHALL, after the one-time migration, expose each former
  monolith entry as a per-file record whose body is byte-identical to the source
  entry, with the monoliths removed; the migration SHALL be delivered as a
  standalone commit separate from the engine / contract changes. This is a
  build-task assertion verified by body diff and commit inspection, not a
  runnable command.
- AC-08: WHEN the `[adr]` config contract changes, THE SYSTEM SHALL update
  `contracts/config.schema.json`, refresh `contracts.lock` via `freeze`, update
  the affected engine docs and adapter templates, and `freeze --check` plus the
  conformance suite SHALL pass.
- AC-09: THE SYSTEM SHALL document in `commands/discuss.md` and `commands/plan.md`
  that ADR consultation loads the `INDEX.md` first, then opens only records
  whose `area` intersects the spec's `surfaces` and whose `status` is active.
- AC-10: IF an `[adr]` config value resolves to an existing file where a record
  directory is expected, THEN `config.rs` validation SHALL report a config error
  rather than silently treating the store as empty.

## QA Scenarios

> Adversarial personas P1-P7 per `reference/risk.md ## QA attack coverage`
> (elevated: exercise all relevant personas, especially P3/P4/P5). A
> non-applicable persona is a row with a reasoned `N/A: <reason>`.

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1 | cli | Automated | Run `mochiflow adr list` on the migrated store with no flags. | Lists active record headers (date, title, area, status); no body dump; exit 0. |
| QA-02 | P2 | cli | Automated | Run `mochiflow adr list --kind decisions --area cli --status active`; run `mochiflow adr search <term>` (default active) and again with `--status all`. | Filters apply; headers only; default search scans only the filtered active set; `--status all` is the explicit wider full-history path. |
| QA-03 | P3 | cli | Automated | Feed a record with a `supersedes:` to a missing id, a supersession cycle, an unknown `area`, and a slug containing `../`. | `mochiflow adr lint` reports dangling cross-ref, cycle, schema violation; path traversal is rejected/sanitized; no crash. |
| QA-04 | P4 | cli | Automated | Diff each migrated record body against the corresponding monolith entry. | Bodies are byte-identical (verbatim); only front-matter and file boundaries are added. |
| QA-05 | P5 | cli | Automated | Inspect the migrated records for missing or garbled front-matter, encoding differences, and entry-count parity against the source monolith. | Every source entry maps to exactly one record; front-matter present and well-formed; encoding preserved; no entry lost or duplicated. |
| QA-06 | P6 | cli | Automated | Run the full `default` verify profile (test + fmt + clippy + `freeze --check`) and `mochiflow index` / `accept` / `lint` / `doctor`. | Existing board index, accept staging, spec lint, and doctor still pass; ADR INDEX never staged. |
| QA-07 | P7 | cli | Human-operated | Compare `commands/open.md`, `reference/git.md`, and discuss/plan wording to the implemented behavior. | Docs describe per-file fold, gitignored INDEX, selective load, and deterministic lint exactly as built. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `cargo test -p mochiflow-core config` | `cli/crates/mochiflow-core/src/config.rs` | PASS | `config::tests::adr_accessors_resolve_directory_roots_and_indexes` |  |
| AC-02 | cli | automated | `cargo test -p mochiflow-core` empty/absent-store no-fallback | `cli/crates/mochiflow-core/src/config.rs` | PASS | `config::tests::adr_absent_or_empty_store_yields_zero_records_no_fallback`; `adr::tests::empty_store_renders_no_records_and_no_fallback` |  |
| AC-03 | cli | automated | `cargo test -p mochiflow-core` staging + index (QA-06) | `cli/crates/mochiflow-core/src/{adr,ship}.rs` | PASS | `ship::tests::accept_staging_includes_adr_record_dirs_but_never_index`; `adr::tests::index_staleness_and_generate_roundtrip` |  |
| AC-04 | cli | automated | `cargo test -p mochiflow-core adr` supersession (QA-03) | `cli/crates/mochiflow-core/src/adr.rs` | PASS | `adr::tests::{active_set_excludes_superseded_deprecated_and_status_lag,dangling_supersedes_is_detected,supersession_cycle_is_detected,well_formed_supersession_has_no_relational_problems,status_active_filter_excludes_status_lagged_superseded}` |  |
| AC-05 | cli | automated | `cargo test` lint + doctor warn/gate (QA-03, QA-06) | `cli/crates/mochiflow-core/src/{adr,doctor}.rs` | PASS | `adr::tests::{lint_classifies_gating_vs_warning_and_sets_exit,lint_flags_unknown_area_as_gating_schema,missing_id_or_area_is_a_schema_problem}`; conformance `doctor_adr_{gates_on_dangling_reference,warns_on_orphan_but_passes,regenerates_stale_index_instead_of_failing,gates_on_unknown_area}` |  |
| AC-06 | cli | automated | `cargo test --test cli` adr commands (QA-01, QA-02) | `cli/crates/mochiflow-cli/src/main.rs` | PASS | cli `adr_{list_returns_only_active_headers_by_default,list_status_all_includes_superseded,search_defaults_to_active_set_and_widens_with_status_all,show_returns_body_and_lineage,show_unknown_id_fails,filters_by_area_and_spec}`; `adr::tests::status_active_filter_excludes_status_lagged_superseded` |  |
| AC-07 | cli | automated | verbatim body diff + commit-separation inspection (QA-04, QA-05) | `.mochiflow/adr/decisions/`, `.mochiflow/adr/pitfalls/` | PASS | standalone migration commit `ca2fe24` touches only `.mochiflow/adr/**` + `config.toml`; record bodies byte-identical to deleted monolith sections; 15 decisions + 8 pitfalls entry-count parity (reviewer git-verified) |  |
| AC-08 | cli | automated | `freeze --check` + conformance (QA-06) | `contracts/config.schema.json`, engine docs | PASS | `cargo run -- freeze --check` clean; conformance schema + golden-config tests; refrozen `contracts.lock` |  |
| AC-09 | cli | human | review docs vs behavior (QA-07) | `engine/commands/{discuss,plan}.md` | CONFIRMED | QA-07 human-confirmed: engine docs (discuss, plan, open, git, authoring, 4 adapters) describe per-file fold, gitignored INDEX, selective load, and deterministic lint exactly as built | 2026-06-28 |
| AC-10 | cli | automated | `cargo test -p mochiflow-core config` file-where-dir error | `cli/crates/mochiflow-core/src/config.rs` | PASS | `config::tests::adr_value_resolving_to_file_is_a_config_error` |  |
