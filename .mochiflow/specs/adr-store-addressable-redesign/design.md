# Make the ADR store addressable and bounded — Design

> Decisions and contracts only. Concrete structs are read from source during
> build.

## Design Decisions

- **Per-file immutable records, not a monolith.** Each decision becomes
  `adr/decisions/{YYYY-MM-DD}-{slug}.md`; each pitfall
  `adr/pitfalls/{YYYY-MM-DD}-{slug}.md`. Records are immutable history; change is
  expressed by adding a record and flipping status, never by rewriting bodies.
  Rejected: keep the monolith with section anchors — still forces whole-file
  reads and offers no per-record front-matter.
- **Generated content index, derived not authored.** A per-store `INDEX.md`
  (date, title, area, status) is regenerated from record front-matter. It is
  gitignored and never staged, exactly like the existing board `INDEX.md`
  (precedent in `index.rs` + `.gitignore` + the pitfall "INDEX.md is a gitignored
  derived cache; never stage it"). Rejected: committed index — concurrent folds
  conflict and hand-edits go stale.
- **Active view by filter, not by synthesis.** "Current why" is computed as
  `status: active` over the immutable log. Rejected: a maintained
  "current-rationale" synthesis page — a third prose artifact that drifts,
  churns, and competes with code/immutable history (violates the No-go).
- **Hard cut, no backward compatibility.** No external users exist, so there is
  no legacy-monolith read path, no warn-and-fallback, and no permanent
  `mochiflow adr migrate` command. The repo's monoliths are split once by an
  in-spec migration. Rejected: dual-read compatibility — permanent legacy branch
  is debt for zero benefit.
- **Deterministic lint only.** `mochiflow adr lint` checks only mechanically
  decidable properties. Semantic contradiction detection needs content
  understanding (LLM) and is out of scope for a single-binary, embedding-free
  CLI; it is left to review judgment. Rejected: heuristic contradiction
  detection — false positives pollute the gate.
- **`area` from `surfaces`.** `area` defaults to the spec's `surfaces` and is
  written during the fold without a dedicated prompt; surface-granular tags stay
  stable across refactors (unlike module-path tags) and let selective load match
  `area ∩ spec.surfaces`.
- **`init` scaffolds directories, not a monolith stub.** `mochiflow init`
  currently writes ADR monolith stub files (`living_spec_stub` /
  `LivingSpecLayer::Adr` / `ADR_STUB_BODY`) at `decisions_path()` /
  `pitfalls_path()` and detects them via `is_living_spec_stub`. Under the
  directory contract, init SHALL instead create empty record directories
  (`adr/decisions/`, `adr/pitfalls/`) and write directory-rooted `[adr]` values
  in the generated `config.toml`; the ADR stub body and its stub-detection
  branch are removed. Rejected: keep scaffolding a stub monolith — directly
  reintroduces the monolith the redesign removes (violates AC-02).
- **Engineering basis.** Reuse existing derivation/staleness patterns from
  `index.rs` and the staging discipline in `ship.rs`; add a `clap` subcommand
  group per the project's existing command structure in `main.rs`. Primary
  source for `clap` derive subcommands: docs.rs/clap v4 (version per
  `cli/Cargo.toml`), confirmed at build time.

## Architecture

- `mochiflow-core`:
  - `config.rs` — `RawAdr` semantics change from file paths to directory roots;
    add accessors for each store's record directory and generated `INDEX.md`
    path. `decisions_path()` / `pitfalls_path()` are redefined or replaced by
    `*_dir()` + `*_index()` accessors.
  - new `adr.rs` — record model (front-matter + body), directory scan, active-set
    computation, INDEX rendering + staleness check, deterministic lint checks,
    and `list / show / search` query logic.
  - `index.rs` — reused patterns (mtime/staleness, table-cell escaping); ADR
    INDEX generation may live in `adr.rs` and reuse helpers.
  - `doctor.rs` — call the deterministic ADR lint subset; gate on
    dangling / missing cross-ref / schema; orphan, stale, and INDEX freshness
    warn (an absent / stale INDEX is regenerated, not a gate failure).
  - `ship.rs` — `run_accept` stages the ADR record directories (`git add
    <dir>`) and must never stage `INDEX.md`.
  - other accessor consumers reconciled by the `decisions_path()` /
    `pitfalls_path()` -> `*_dir()` rename: `init.rs` (scaffold + config template +
    `is_living_spec_stub` ADR branch), `detach.rs` (`keep_default_project_data`),
    `adapter.rs` (the `adr.decisions` / `adr.pitfalls` placeholder map + tests).
- `mochiflow-cli`:
  - `main.rs` — add the `Adr` subcommand group (`list`, `show`, `search`,
    `lint`) and a `DoctorTarget`/allowlist entry if doctor exposes adr; update
    the `config show` ADR lines for the directory accessors.
- `contracts/` — `config.schema.json` `adr` block, `contracts.lock` refreeze,
  `conformance.rs` schema + golden-index assertions for the new layout.
- Engine docs (repo-root `engine/`) — `commands/open.md`, `reference/git.md`,
  `reference/authoring.md`, `commands/discuss.md`, `commands/plan.md`, and the
  four adapter templates (agents / kiro / claude-code / copilot).

## Data Model / Interfaces

- Record front-matter (decisions): `id` (`{date}-{slug}`), `date`, `area`
  (list, surface-granular), `spec` (source slug), `status: active | superseded |
  deprecated`, optional `supersedes` / `superseded_by` (record ids). Body is the
  verbatim *why* (decision + rejected alternatives).
- Record front-matter (pitfalls): same identity fields plus
  `status: active | resolved`; body keeps `Applies to / Signal / Cause /
  Guardrail / Check / Status`.
- `[adr]` config: `decisions` and `pitfalls` are directory paths; the generated
  index is `<dir>/INDEX.md`. Schema requires non-empty strings (directories);
  no monolith file form is accepted.
- `mochiflow adr` CLI surface (read-only):
  - `adr list [--kind decisions|pitfalls] [--area A] [--status S] [--spec slug]`
    → header rows; default `--status active`.
  - `adr show <id> [--kind ...]` → full body + supersession lineage.
  - `adr search <term> [--kind ...] [filters]` → header rows of matches over the
    filtered, default-active set (front-matter + body keyword / substring within
    that bounded set; no embedding). `--status all` widens to full history and is
    the only path that scans superseded / deprecated bodies, keeping the default
    path `O(relevant active records)`.
  - `adr lint [--kind ...]` → deterministic structural report; non-zero exit on
    gating failures.
- Active-set rule: a decision is active unless `status` is `superseded` /
  `deprecated`; a record referenced by a valid `supersedes` is treated as
  superseded for active computation even if its own `status` lags (lint flags
  the lag).

## Error Handling

- Missing/empty store directory → zero records, success (no fallback).
- `[adr]` value resolving to an existing file where a record directory is
  expected → config validation error (AC-10), not a silent empty store.
- Malformed front-matter / unknown `area` / missing required key → lint schema
  violation (gating); `list/show/search` skip-with-warning rather than crash.
- Dangling `supersedes`/`superseded_by`, one-sided cross-ref, supersession cycle
  → lint gating failures with the offending ids.
- Record path containing separators or `..` → rejected/sanitized; never escape
  the store directory.
- INDEX absent or stale vs records → non-blocking warning; `doctor` regenerates
  it (as it does the board `INDEX.md`) rather than failing; `mochiflow adr lint`
  reports staleness but it is not a gating check. Generation never stages the
  file.

## Test Strategy

- Unit (`mochiflow-core`): config dir resolution + schema; record parse;
  active-set + supersession (incl. cycle, dangling, one-sided); lint check
  classification (gating vs warning); INDEX render + staleness; `list/show/search`
  filtering and header-vs-body output; path-traversal rejection.
- Integration (`mochiflow-cli/tests`): `mochiflow adr ...` exit codes and
  output; `accept` stages dirs but not `INDEX.md`; `doctor` gating subset.
- Conformance (`conformance.rs`): updated `config.schema.json` accepts the new
  `adr` shape (presence + non-empty strings per key); file-vs-directory rejection
  is runtime config validation (AC-10), not a schema constraint; golden ADR
  INDEX; `freeze --check` parity.
- Migration: verbatim-body assertion (migrated record bodies byte-identical to
  source entries), front-matter / encoding / entry-count parity (QA-05), and
  standalone-commit separation. No re-run guard — migration is one-time with no
  command surface.
- Full `default` verify profile is the build-completion gate.

## Workstreams

| Workstream | Surface | Responsibility | Depends on | Verification |
| --- | --- | --- | --- | --- |
| WS1 storage + config contract | cli | Dir-rooted `[adr]`, record model, generated gitignored INDEX, accept staging, schema + freeze, one-time migration | none | `cargo test`, conformance, `freeze --check`, verbatim diff |
| WS2 lifecycle + lint | cli | Front-matter, supersession active-set, `mochiflow adr lint`, doctor wiring | WS1 | `cargo test`, doctor gating tests |
| WS3 retrieval + selective load | cli | `mochiflow adr list/show/search`, discuss/plan selective-load docs | WS1, WS2 | `cargo test` CLI integration, engine lint, P7 review |

## Integration Contract

- Contract owner: `mochiflow-core` config + `contracts/config.schema.json`.
- Request/Response: the `[adr]` config keys (`decisions`, `pitfalls`) change
  meaning from file paths to directory roots; the generated index path is
  `<dir>/INDEX.md`.
- Error: config validation rejects empty values; a path that is a file where a
  directory is expected is a config error.
- Auth: none (local filesystem).
- Compatibility: hard cut — no legacy monolith acceptance. The repo migrates its
  own store in-spec; `contracts.lock` is refrozen so the integrity gate passes.
- Failure handling: `freeze --check` and conformance fail the build if schema,
  lock, or golden index drift; migration is a separate reversible commit.
- Verification: `default` profile (test + fmt + clippy + `freeze --check`) plus
  conformance suite.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Date: 2026-06-28 (elevated cadence: independent-reviewer run once after all tasks)
- Summary: All 10 ACs verified against implementation, tests, and git history;
  hard cut confirmed (no `decisions_path`/`pitfalls_path`, no ADR stub, no
  migrate command). AC-07 git-verified as an isolated migration commit with
  byte-identical record bodies and 15/8 entry-count parity.
- Findings (all non-blocking; no Critical/High):
  - MEDIUM (AC-05/QA-03): unknown `area` was not validated. **Resolved** in
    follow-up commit — `adr lint` / `doctor` now gate on an `area` that is not a
    configured surface (unit + conformance coverage added).
  - LOW: a `resolved` pitfall with no links emits a perpetual Orphan warning
    (warning noise only; non-gating). Left as-is.
  - LOW: `list`/`search` skip malformed records silently rather than
    skip-with-warning. Left as-is (lint is the gating surface).
  - LOW: some unrelated test fixtures still use the old monolith `*.md` config
    form (harmless — no file is created so validation never trips). Left as-is.

### Second review pass (2026-06-28, delegated)

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Findings (all non-blocking; no Critical/High): three MEDIUM; two resolved in
  code, one is the build/open boundary:
  - MEDIUM (AC-05): missing / empty `id` and `area` were not gated (`id` fell
    back to the file stem; `area` defaulted to empty). **Resolved** — both are
    now required front-matter keys; a missing / empty value is a gating schema
    violation (`adr::tests::missing_id_or_area_is_a_schema_problem`).
  - MEDIUM (AC-04 / AC-06): explicit `--status active` matched the raw status
    string, re-surfacing a status-lagged superseded record. **Resolved** —
    `--status active` now resolves to the effective active set
    (`adr::tests::status_active_filter_excludes_status_lagged_superseded`).
  - MEDIUM (AC-09 / QA-07): AC-09 is still `PENDING_HUMAN`. This is the
    build/open boundary: per `commands/build.md` step 6 a human-checked AC is
    recorded as `PENDING_HUMAN` at build, and the QA-07 doc-vs-behavior request
    is made once, in `open`. The done-eligible token is settled at accept/open,
    not build. AI-observed consistency was confirmed (discuss/plan, open, git,
    authoring, and the four adapters describe the implemented behavior; adapter
    `generate --check` reports 0 drift); final human sign-off remains for open.
