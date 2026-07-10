# Specs Reference

Artifact roles, spec depth, backlog promotion, authoring rules, the consistency
check, and session recovery. Used by `plan`, `build`, `discuss`, and `onboard`.
Lifecycle states and gates live in `reference/lifecycle.md`; the AC Matrix and
verification profiles live in `reference/verification.md`; the `design.md`
required condition and QA attack coverage live in `reference/risk.md`.

## Spec lane

The spec lane verbs create, approve, implement, accept, and deliver durable spec
artifacts: discuss / plan / build settle the work, then open / update / close
handle PR delivery. Specs stay flat at `{specs_dir}/{slug}/` for their whole
life. Micro is the smallest spec depth for concrete small work; it keeps
`spec.yaml`, `spec.md`, the AC Matrix, lifecycle state, and PR delivery while
skipping pitch/design/task documents when they are not needed.

## Depth scaling

A change is always one folder under `{specs_dir}/{slug}/`. Documents grow only
as far as the change needs:

| Depth | Use case | Documents | Requirements detail | Tasks |
| --- | --- | --- | --- | --- |
| Micro spec | Concrete small fix | `spec.yaml` + `spec.md` | problem / change / AC / verify | none |
| Standard spec | Normal feature/fix | `pitch.md` + `spec.md` + `tasks.md` | AC table + QA examples | checklist |
| Design spec | Design decision or multiple areas | `pitch.md` + `spec.md` + `design.md` + `tasks.md` | NFR / contract / examples | dependency checklist |
| Critical spec | migration / security / data loss / external contract | full | traceability / rollback / observability / reviewer | per-task verification checklist |

Let depth increase with risk, integration, surfaces, ambiguity, and external
contracts. Do not add prose for its own sake; detail should be checkable,
traceable, and executable.

Micro is inferred from file presence: `spec.yaml` + `spec.md`, with no
`pitch.md`, `design.md`, or `tasks.md`. It is eligible only for standard-risk,
single-surface, `integration: none` work with no design-required impact, human
QA, or ADR fold need. A micro candidate that discovers durable rationale,
pitfalls, integration, elevated/critical risk, public contract impact, or human
QA need escalates in place before approval or delivery.

`design.md` necessity is governed by `reference/risk.md ## design.md required
condition`. `tasks.md` is required for standard-or-larger multi-step work.

## Backlog seeds

`{specs_dir}/_backlog/{slug}.md` is a single-file inbox for raw ideas only. It
is not a spec and is not a plan-ready handoff.

- Raw seed: `maturity: seed`, created from `templates/backlog/seed.md`, and used
  as raw input for `discuss`. Body: `## Signal`, `## Why It Matters`,
  `## Evidence`, `## Open Questions`.
Shared frontmatter: `slug,title,maturity,source,created,updated` (+ optional
`module,surface,type_hint,source_spec,source_phase`).

Lifecycle: create raw seed → `discuss` reads it as input → when agreement is
reached, `discuss` creates `{specs_dir}/{slug}/spec.yaml` (`status: draft`) and
`{specs_dir}/{slug}/pitch.md`, creates/switches to `{prefix}/{slug}`, deletes the
raw seed when present, runs pitch-only lint, and commits the promotion. `plan`
then reads `pitch.md` as the durable input for standard-or-larger specs. For an
explicit concrete request with no existing draft, `plan` may create a direct
micro spec without `pitch.md`. Interrupted discuss keeps the raw seed file. Do
not put AC, QA, design, tasks, or final classification in backlog files.

Legacy `_backlog/{slug}/` spec-format directories are deprecated and no longer
rendered by tooling; they remain on disk read-only.

## spec.yaml schema

`spec.yaml` is the single source of truth for spec metadata. Markdown files
carry no frontmatter.

```yaml
version: 1
slug: measurement-sync
title: Measurement Sync
type: feature        # feature | fix | refactor | docs | chore
module: InspectionFeature   # optional
surfaces:
  - ios              # ios | api | web
integration: none    # none | contract | workflow
risk: standard       # standard | elevated | critical (ordered enum)
status: draft        # draft | approved | accepted (done = legacy/derived)
created: YYYY-MM-DD
updated: YYYY-MM-DD
completed: YYYY-MM-DDTHH:MM:SSZ   # legacy: ordered the Done view for archived specs; the engine no longer writes it
```

`status` flow is `draft → approved → accepted` (`reference/lifecycle.md`);
`done` is a legacy/derived state for archived specs, not written by the current
flow. Whether `design.md` / `tasks.md` exist is expressed by file presence, not
metadata. `completed` is a legacy timestamp tied to `done`; the current flow
does not write it.

## SSOT discipline

Fix each fact in one place; reference by ID elsewhere.

| fact | source of truth | elsewhere |
| --- | --- | --- |
| AC (pass/fail criteria) | `spec.md` | task line reference `[AC-01]` (ID only) |
| surface / risk / type / status | `spec.yaml` | not repeated |
| QA scenarios | `spec.md` | PR `## Testing` derives from it; no intermediate file |
| reviewer / git / journal cadence | `reference/review.md` / `reference/risk.md` | not repeated |
| user-authored standing rules | `[constitution]` (project / local), written by the user | always-loaded; never generated from code |
| current-state orientation (purpose / layout / tech) | code/config, mapped into `[context]` (product / structure / tech) via onboard / `refresh-context` | loaded on demand for workflow or repository orientation; never folded |
| design rationale (*why*) / pitfalls history | `[adr]` (directory-rooted `decisions` / `pitfalls` stores: one immutable per-file record each, plus a generated gitignored `INDEX.md`), grown by open's fold via supersession | on-demand / phase load |

The three durable guidance layers differ by lifecycle: `[constitution]` is
user-authored standing guidance, `[context]` is a code-derived current-state map
*refreshed* forward (onboard / `refresh-context`), and `[adr]` is dated history
*folded* at open (`reference/knowledge.md ## Living-spec fold`). Code is always
the source of truth for current state; prose is not.

## Durable vs ephemeral artifacts

A spec folder holds only **durable** artifacts; delivery scaffolding is
**ephemeral** and lives under `{install_dir}/state/{slug}/` (gitignored, swept
post-merge per `reference/delivery.md ## Post-merge local cleanup`).

| class | artifacts | home | archived |
| --- | --- | --- | --- |
| durable | `spec.yaml` · optional `pitch.md` · `spec.md` (incl. AC Verification Matrix) · conditional `design.md` · conditional `tasks.md` | `{specs_dir}/{slug}/` (flat for life) | no |
| ephemeral | PR body file (`pr-body.md`) · `pr-request.json` (pr_driver only) | `{install_dir}/state/{slug}/` | no |

Ephemeral artifacts are regenerable from the durable spec; their durable record
is the merged PR (delivery) or the AC Verification Matrix (QA results). Rationale:
they are inter-process handoffs / working sheets, not knowledge, so keeping them
in the tracked tree pollutes history and the archive.

QA role split: `spec.md` QA Scenarios are the source of truth for *what* to test
and *how* (dimension + steps + expected result). The **AC Verification Matrix**
is the results ledger (result + evidence pointers). During open, the agent
presents QA items directly in conversation via the round-trip protocol
(`commands/open.md` Acceptance step 3); PR reviewers read the `## Testing`
section in the PR description (derived from QA Scenarios). There is no
intermediate QA file.

## pitch.md

Discuss writes `pitch.md` as the first durable artifact for standard-or-larger
work, before plan expands the work into `spec.md`. Direct micro planning may skip
`pitch.md` when the user request is already explicit and concrete. Pitch carries
the agreed problem and solution shape:

- Problem
- Appetite
- Solution
- Rabbit Holes
- No-gos
- Alternatives Considered
- Open Questions

No frontmatter; metadata lives in `spec.yaml`. For standard-or-larger specs,
plan reads `pitch.md` and absorbs the relevant rationale into
`spec.md ## Background and Design Rationale`.

## spec.md

Single document carrying the **why** and the acceptance contract:

- Background and design rationale (the decisions and *why*; absorbs the old Decision Brief)
- User story (who / what / why)
- Scope boundary (in / out)
- Edge cases
- Acceptance criteria in **EARS** (`THE SYSTEM SHALL` / `WHEN` / `WHILE` / `IF...THEN` / `WHERE`), each third-party Yes/No decidable, IDs `AC-01`...
- QA scenarios (operation steps, with `Dimension` and `Scope` columns; `Scope`
  values include `ios`/`api`/`web`/`cross-surface`/`human`)
- Open items as `[NEEDS-CLARIFICATION: ...]` (lint warns; resolve before `approved`)

For a micro spec, `spec.md` may be a few lines plus the AC Matrix: problem /
cause / change / verification.

## design.md

Write only when required (`reference/risk.md ## design.md required condition`).
Carries:

- Design decisions and rationale
- Architecture, data model / interface at signature level
- Error handling, test strategy
- `## Workstreams` table (surface · responsibility · depends · verification) when multi-workstream
- `## Integration Contract` when `integration ≠ none`
- `## Review Results` for mandatory reviewer runs when `risk ≥ elevated` (`Reviewer mode: delegated | inline`, `Verdict: pass | pass-with-comments | fail`)
- Optional `## Integration Log` appended during build when the risk table calls for it (seam decisions, ownership, dead-code handling, handoff). Replaces the old separate journal file.

Do not write: AC tables (use task line `[AC-01]` references + AC Matrix), concrete class/struct
definitions (read source during build).

## tasks.md

Write when the change is multi-step. Carries:

- Header: one-line scope · risk · critical stop conditions (1–3 spec-specific)
- `## Defaults` preamble (shared verification command + stop conditions)
- A `## Tasks` checklist of dependency-ordered checkbox tasks. Each task line is
  `- [ ] T-### [AC-01] title` — the bracket reference is required and is one or
  more AC IDs (`[AC-01]`, `[AC-01, AC-02]`), one NFR (`[NFR-01]`), or a chore
  reason (`[chore: ...]`). Use a compound AC reference when one implementation
  task naturally covers multiple related ACs; do not split tasks just to make
  one task per AC. Each task body needs `Depends on:` (prior `T-###` IDs or
  `none`), `Files:`, `Done:`, and `Stop:`.
- `Files:` entries list planned file touches. Use normal paths for files that
  will be created or modified. For planned deletions, prefix the path with
  `deleted:`, for example ``- deleted: `path/to/OldFile.ext` ``. The marker
  applies to every path parsed from that line, though one deleted path per line
  is the clearest style.
- `[P]` parallel marks: a task tagged `- [ ] T-### [P] ...` runs parallel to the
  previous `[P]` block; no `[P]` depends on the previous task; never `[P]` two
  tasks editing the same file

`lint` enforces this structure (top-level `T-###` checkbox tasks with the four
required blocks and a valid reference); authored tasks must match it or `plan`'s
lint gate fails.

## Session-recoverability (plan authoring rule)

Build executes inline, but long work may resume in a new session. A new session
must be able to recover the implementation contract from durable artifacts and
committed state, not from hidden conversation memory. Author tasks to be
**session-recoverable**:

- Every fact needed to implement or resume a task correctly must be recoverable
  from `spec.md`, `design.md`, the task row, committed code, and git trailers.
  Cross-task reasoning that the current session might otherwise carry implicitly
  is written into `design.md` at plan time — this is the practical form of
  "share the contract".
- When a file appears in more than one task's `Files`, each such task's `Done`
  states how it leaves the shared structure consistent, so a later session can
  pick up the file from its committed state alone.

This is **plan authoring discipline enforced by `plan-auditor` S1 Internal Coherence
judgment, not a new deterministic lint** — recoverability cannot be decided
mechanically, so no lint check is added for it. If implementation finds a
required fact missing from the durable source set, stop and route back to `plan`
rather than improvising.

## Consistency check (plan, once)

After authoring, verify once against the spec's own intent before `approved`:
Open Questions closed (or kept as `[NEEDS-CLARIFICATION]`), design decisions not
contradicting the rationale in `spec.md`, task line `[AC-01]` / compound
`[AC-01, AC-02]` references covering `spec.md` AC, no dependency cycle in
Workstreams. No per-document self-review loop (deep defects are caught by
`plan-auditor` before approval when selected and `change-reviewer` during build).

Before asking for approval, remove all template residue. `lint` enforces these
checks for expanded spec documents; placeholder-like text inside fenced code
blocks or inline code spans is ignored so legitimate examples can remain:

- no `{...}` placeholder remains;
- no example-only row remains;
- no `TBD` remains except in AC Verification Matrix fields intentionally owned by
  `build`;
- no template-only HTML comment remains;
- no "None" is used for a required section without a concrete reason.
