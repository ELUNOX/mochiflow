---
name: spec-discuss
phase: discuss
description: |
  mochiflow's discuss phase. Investigate code, organize design decisions through
  interactive interview, and reach an agreed why/what. Activate on the explicit
  command `mochiflow-discuss`, or natural phrasing like "ブレストして" / "壁打ちして"
  / "相談したい". Writes no spec documents or implementation code; the agreed
  decisions are captured into spec.md by plan.
triggers:
  - mochiflow-discuss
  - ブレストして
  - 壁打ちして
  - 相談したい
trigger_patterns:
  - "{slug} discuss"
artifacts:
  - Decision summary only (captured into spec.md 背景と設計判断 by plan)
prerequisites: []
execution: inline
allowed_writes: []
forbidden_writes:
  - "{specs_dir}/**"
  - "{write.allow}"
  - .git/**
references:
  - reference/workflow.md
  - reference/language.md
  - reference/engineering-standards.md
---

# spec-discuss

## Purpose

Reach agreement on the why / what / key design decisions through investigation and discussion. Write no spec documents and no implementation code.

## Procedure

1. Investigate before asking. Always fix current state from **code**; never ask the user what code can answer. Read the constitution (`[constitution].project` / `[constitution].local`) and foundational context (`[context].product` / `[context].structure` / `[context].tech`) for current-state orientation (a map derived from code, refreshed via onboard / `refresh-context`). Read ADR (`[adr].decisions` / `[adr].pitfalls`) only on demand, as historical records of *why* (decision rationale, rejected alternatives, known pitfalls); re-verify any current-state claim against code. Prose is never the source of truth for current state.
2. `{slug} discuss` reads `_backlog/{slug}.md` as raw input (`reference/workflow.md ## Backlog seeds`) when the slug exists only in `_backlog/`; if `{specs_dir}/{slug}/` already exists it re-opens that spec instead. Do not copy seed content straight into conclusions.
3. Organize the UI / data model / API / migration / error handling / testing decision tree internally and resolve it dependency-first.
4. One question at a time, each with a recommended answer, rationale, why the main alternatives are rejected, and impact.
5. Ask for specifics when answers are vague ("make it nice", "your call").
6. When every branch is resolved, present the agreement in the project language
   using plain labels for purpose / background / scope / decisions / assumptions
   / open questions / change impact. Internally this is the Decision summary;
   do not lead with internal headings. Guide the user toward creating the plan,
   mentioning `spec-plan` only as the command if useful.

## Stop conditions

- Do not move to the next branch while scope is undefined, contradictory, or unjustified.
- Do not proceed to `spec-plan` with Open Questions unresolved (carrying them into plan as `[NEEDS-CLARIFICATION]` is allowed; resolve before `approved`).
- Do not delete a backlog seed in discuss alone (deletion happens in plan).
- Keep transient notes in the conversation only; do not create or update spec files.
- Do not touch implementation code / branch / PR.
