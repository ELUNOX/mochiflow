---
name: spec-discuss
phase: discuss
description: |
  mochiflow's discuss phase. Investigate code, organize design decisions through
  interactive interview, and reach an agreed why/what. Activate on the explicit
  command `mochiflow-discuss`, or natural phrasing like "ブレストして" / "壁打ちして"
  / "相談したい". Writes no spec documents or implementation code; the agreed
  decisions are persisted as a ready-for-plan handoff under `_backlog/` and
  captured into spec.md by plan.
triggers:
  - mochiflow-discuss
  - ブレストして
  - 壁打ちして
  - 相談したい
trigger_patterns:
  - "{slug} discuss"
artifacts:
  - "{specs_dir}/_backlog/{slug}.md (maturity: ready-for-plan handoff)"
prerequisites: []
execution: inline
allowed_writes:
  - "{specs_dir}/_backlog/{slug}.md"
forbidden_writes:
  - "{specs_dir}/{slug}/**"
  - "{specs_dir}/_done/**"
  - "{write.allow}"
  - .git/**
references:
  - reference/workflow.md
  - reference/language.md
  - reference/engineering-standards.md
  - templates/backlog/discuss-handoff.md
---

# spec-discuss

## Purpose

Reach agreement on the why / what / key design decisions through investigation
and discussion. Write no spec documents and no implementation code; persist only
the ready-for-plan handoff in `_backlog/{slug}.md`.

## Procedure

1. Investigate before asking. Always fix current state from **code**; never ask the user what code can answer. Read the constitution (`[constitution].project` / `[constitution].local`) and foundational context (`[context].product` / `[context].structure` / `[context].tech`) for current-state orientation (a map derived from code, refreshed via onboard / `refresh-context`). Read ADR (`[adr].decisions` / `[adr].pitfalls`) only on demand, as historical records of *why* (decision rationale, rejected alternatives, known pitfalls); re-verify any current-state claim against code. Prose is never the source of truth for current state.
2. `{slug} discuss` reads `_backlog/{slug}.md` as raw input (`reference/workflow.md ## Backlog seeds`) when the slug exists only in `_backlog/`; if `{specs_dir}/{slug}/` already exists it re-opens that spec instead. Do not copy seed content straight into conclusions.
3. Organize the UI / data model / API / migration / error handling / testing decision tree internally and resolve it dependency-first.
4. One question at a time, each with a recommended answer, rationale, why the main alternatives are rejected, and impact.
5. Ask for specifics when answers are vague ("make it nice", "your call").
6. When every branch is resolved, write or update `{specs_dir}/_backlog/{slug}.md`
   from `templates/backlog/discuss-handoff.md` with `maturity: ready-for-plan`,
   `source: conversation`, and `source_phase: discuss`. Required frontmatter:
   `slug`, `title`, `maturity`, `source`, `source_phase`, `created`, `updated`;
   keep optional `surface`, `type_hint`, and `module` when known. Required body
   headings: `## Decision Summary`, `## Decisions`, `## Assumptions`,
   `## Open Questions`, `## Change Impact`, and `## Evidence`.
7. Present the agreement in the conversation language
   using plain labels for purpose / background / scope / decisions / assumptions
   / open questions / change impact. Internally this is the Decision summary;
   do not lead with internal headings. Guide the user toward creating the plan,
   mentioning `mochiflow-plan` or `{slug} plan` only as the command if useful.

## Stop conditions

- Do not move to the next branch while scope is undefined, contradictory, or unjustified.
- Do not proceed to `mochiflow-plan` with Open Questions unresolved (carrying them into plan as `[NEEDS-CLARIFICATION]` is allowed; resolve before `approved`).
- Do not delete a backlog seed in discuss alone (deletion happens in plan).
- Keep scratch notes in the conversation only; persist the final agreement in
  `_backlog/{slug}.md`. Do not create or update spec files.
- Do not touch implementation code / branch / PR.
