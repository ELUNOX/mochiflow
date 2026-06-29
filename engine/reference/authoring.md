# Authoring Reference

How to write `spec.yaml`, `spec.md`, `design.md`, `tasks.md`. Used by `plan` and
`build`.

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

`status` flow is `draft → approved → accepted` (`workflow.md`); `done` is a
legacy/derived state for archived specs, not written by the current flow. Whether
`design.md` / `tasks.md` exist is expressed by file presence, not metadata.
`completed` is a legacy timestamp tied to `done`; the current flow does not write
it.

## SSOT discipline

Fix each fact in one place; reference by ID elsewhere.

| fact | source of truth | elsewhere |
| --- | --- | --- |
| AC (pass/fail criteria) | `spec.md` | task line reference `[AC-01]` (ID only) |
| surface / risk / type / status | `spec.yaml` | not repeated |
| QA scenarios | `spec.md` | PR `## Testing` derives from it; no intermediate file |
| reviewer / git / journal cadence | `risk.md` | not repeated |
| user-authored standing rules | `[constitution]` (project / local), written by the user | always-loaded; never generated from code |
| current-state orientation (purpose / layout / tech) | code/config, mapped into `[context]` (product / structure / tech) via onboard / `refresh-context` | always-loaded; never folded |
| design rationale (*why*) / pitfalls history | `[adr]` (directory-rooted `decisions` / `pitfalls` stores: one immutable per-file record each, plus a generated gitignored `INDEX.md`), grown by open's fold via supersession | on-demand / phase load |

The three durable guidance layers differ by lifecycle: `[constitution]` is
user-authored standing guidance, `[context]` is a code-derived current-state map
*refreshed* forward (onboard / `refresh-context`), and `[adr]` is dated history
*folded* at open (`reference/git.md ## Living-spec fold`). Code is always the
source of truth for current state; prose is not.

## Durable vs ephemeral artifacts

A spec folder holds only **durable** artifacts; delivery scaffolding is
**ephemeral** and lives under `{install_dir}/state/{slug}/` (gitignored, swept
post-merge per `reference/git.md ## Post-merge local cleanup`).

| class | artifacts | home | archived |
| --- | --- | --- | --- |
| durable | `spec.yaml` · `pitch.md` · `spec.md` (incl. AC Verification Matrix) · `design.md` · `tasks.md` | `{specs_dir}/{slug}/` (flat for life) | no |
| ephemeral | PR body file (`pr-body.md`) · `pr-request.json` (pr_driver only) | `{install_dir}/state/{slug}/` | no |

Ephemeral artifacts are regenerable from the durable spec; their durable record
is the merged PR (delivery) or the AC Verification Matrix (QA results). Rationale:
they are inter-process handoffs / working sheets, not knowledge, so keeping them
in the tracked tree pollutes history and the archive.

QA role split: `spec.md` QA Scenarios are the source of truth for *what* to test
and *how* (steps + expected result). The **AC Verification Matrix** is the results
ledger (result + evidence pointers). During open, the agent presents QA items
directly in conversation via the round-trip protocol (`commands/open.md`
Acceptance step 3); PR reviewers read the `## Testing` section in the PR
description (derived from QA Scenarios). There is no intermediate QA file.

## pitch.md

Discuss writes `pitch.md` as the first durable artifact, before plan expands the
work into `spec.md`. It carries the agreed problem and solution shape:

- Problem
- Appetite
- Solution
- Rabbit Holes
- No-gos
- Alternatives Considered
- Open Questions

No frontmatter; metadata lives in `spec.yaml`. Plan reads `pitch.md` and absorbs
the relevant rationale into `spec.md ## Background and Design Rationale`.

## spec.md

Single document carrying the **why** and the acceptance contract:

- Background and design rationale (the decisions and *why*; absorbs the old Decision Brief)
- User story (who / what / why)
- Scope boundary (in / out)
- Edge cases
- Acceptance criteria in **EARS** (`THE SYSTEM SHALL` / `WHEN` / `WHILE` / `IF...THEN` / `WHERE`), each third-party Yes/No decidable, IDs `AC-01`...
- QA scenarios (operation steps, with a `Scope` column: `ios`/`api`/`web`/`cross-surface`/`human`)
- Open items as `[NEEDS-CLARIFICATION: ...]` (lint warns; resolve before `approved`)

For a trivial change `spec.md` may be a few lines (problem / cause / change / verification).

## design.md

Write only when required (`risk.md ## design.md required condition`). Carries:

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

## Worker-recoverability (plan authoring rule)

Build may dispatch each task to a disposable worker that sees only `design.md` +
its task row + the committed code (`commands/build.md` 3·orchestrator). A worker
can read a prior worker's committed *code* but not its *reasoning*. So plan must
author tasks to be **worker-recoverable**:

- Every fact needed to implement a task correctly must be recoverable from
  (`design.md` + the task row + reading committed code). Cross-task reasoning
  that an inline build would carry implicitly is written into `design.md` at plan
  time — this is the practical form of "share the contract".
- When a file appears in more than one task's `Files`, each such task's `Done`
  states how it leaves the shared structure consistent, so a later worker can
  pick up the file from its committed state alone.

This is **plan authoring discipline enforced by reviewer Stage 1 judgment, not a
new deterministic lint** — recoverability cannot be decided mechanically, so no
lint check is added for it. A worker that finds a required fact missing returns
`blocked` (a plan gap) rather than improvising.

## Consistency check (plan, once)

After authoring, verify once against the spec's own intent before `approved`:
Open Questions closed (or kept as `[NEEDS-CLARIFICATION]`), design decisions not
contradicting the rationale in `spec.md`, task line `[AC-01]` / compound
`[AC-01, AC-02]` references covering `spec.md` AC, no dependency cycle in
Workstreams. No per-document self-review loop (deep defects are caught by
`independent-reviewer` during build).

Before asking for approval, remove all template residue. `lint` enforces these
checks for expanded spec documents; placeholder-like text inside fenced code
blocks or inline code spans is ignored so legitimate examples can remain:

- no `{...}` placeholder remains;
- no example-only row remains;
- no `TBD` remains except in AC Verification Matrix fields intentionally owned by
  `build`;
- no template-only HTML comment remains;
- no "None" is used for a required section without a concrete reason.
