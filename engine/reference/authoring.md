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
  - app              # surface names come from config.toml [surfaces.*]
integration: none    # none | contract | workflow
risk: standard       # standard | elevated | critical (ordered enum)
status: draft        # draft | approved | done
created: YYYY-MM-DD
updated: YYYY-MM-DD
```

`status` flow is `draft → approved → done` (`workflow.md`). Whether `design.md` /
`tasks.md` exist is expressed by file presence, not metadata.

## SSOT discipline

Fix each fact in one place; reference by ID elsewhere.

| fact | source of truth | elsewhere |
| --- | --- | --- |
| AC (pass/fail criteria) | `spec.md` | tasks header `Covers AC: AC-01` (ID only; legacy `対応 AC:` is accepted) |
| surface / risk / type / status | `spec.yaml` | not repeated |
| QA scenarios | `spec.md` | `qa-instructions.md` references, does not copy |
| reviewer / git / journal cadence | `risk.md` | not repeated |
| user-authored standing rules | `[constitution]` (project / local), written by the user | always-loaded; never generated from code |
| current-state orientation (purpose / layout / tech) | code/config, mapped into `[context]` (product / structure / tech) via onboard / `refresh-context` | always-loaded; never folded |
| design rationale (*why*) / pitfalls history | `[adr]` (decisions / pitfalls), appended by ship's fold | on-demand / phase load |

The three durable guidance layers differ by lifecycle: `[constitution]` is
user-authored standing guidance, `[context]` is a code-derived current-state map
*refreshed* forward (onboard / `refresh-context`), and `[adr]` is dated history
*folded* at ship (`reference/git.md ## Living-spec fold`). Code is always the
source of truth for current state; prose is not.

## Durable vs ephemeral artifacts

A spec folder holds **durable** artifacts, including the reviewer-facing QA
instructions. PR handoff scaffolding is **ephemeral** and lives under
`{install_dir}/state/{slug}/` (gitignored, swept post-merge per
`reference/git.md ## Post-merge local cleanup`).

| class | artifacts | home | archived |
| --- | --- | --- | --- |
| durable | `spec.yaml` · `spec.md` (incl. AC Verification Matrix) · `design.md` · `tasks.md` · `qa-instructions.md` | `{specs_dir}/{slug}/` → `_done/` | yes |
| ephemeral | PR body file (`pr-body.md`) · `pr-request.json` (pr_driver only) | `{install_dir}/state/{slug}/` | no |

Ephemeral PR artifacts are regenerable from the durable spec and their durable
record is the merged PR. Rationale: they are inter-process handoffs, not
reviewer-facing knowledge, so keeping them in the tracked tree pollutes history.

QA role split: `spec.md` QA scenarios are the source of truth for *what* to test;
`qa-instructions.md` is the durable reviewer-facing guide for *how* to run and
where to capture evidence; the **AC Verification Matrix** is the results ledger
(result + evidence pointers). The human follows `qa-instructions.md` during ship
and never reads the AC Matrix as an instruction sheet.

## spec.md

Single document carrying the **why** and the acceptance contract:

- Background and design rationale (the decisions and *why*; absorbs the old Decision Brief)
- User story (who / what / why)
- Scope boundary (in / out)
- Edge cases
- Acceptance criteria in **EARS** (`THE SYSTEM SHALL` / `WHEN` / `WHILE` / `IF...THEN` / `WHERE`), each third-party Yes/No decidable, IDs `AC-01`...
- QA scenarios (operation steps, with a `Scope` column using configured surfaces, `cross-surface`, or `human`)
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
- Optional `## Integration Log` appended during build when the risk table calls for it (seam decisions, ownership, dead-code handling, handoff). Replaces the old separate journal file. Legacy `## 統合ログ` headings are historical and should not be created by new specs.

Do not write: AC tables (use tasks `Covers AC` + AC Matrix), concrete class/struct
definitions (read source during build).

## tasks.md

Write when the change is multi-step. Carries:

- Header: one-line scope · risk · critical stop conditions (1–3 spec-specific)
- `## Defaults` preamble (shared verification command + stop conditions)
- Dependency-ordered tasks; each: `Covers AC`, change area, task-specific stop conditions
- `[P]` parallel marks: `[P]` tasks run parallel to the previous `[P]` block; no `[P]` depends on the previous task; never `[P]` two tasks editing the same file

## Consistency check (plan, once)

After authoring, verify once against the spec's own intent before `approved`:
Open Questions closed (or kept as `[NEEDS-CLARIFICATION]`), design decisions not
contradicting the rationale in `spec.md`, tasks `Covers AC` covering `spec.md` AC,
no dependency cycle in Workstreams. No per-document self-review loop (deep defects
are caught by `independent-reviewer` during build).
