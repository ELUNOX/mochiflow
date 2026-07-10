---
name: spec-discuss
phase: discuss
description: |
  mochiflow's discuss phase. Investigate code, organize design decisions through
  interactive interview, and reach an agreed why/what. Writes the agreed pitch as
  the first durable spec artifact,
  creates the spec branch, and commits `spec.yaml (draft)` + `pitch.md`.
  Discuss is optional for explicit concrete micro requests handled by plan.
artifacts:
  - "{specs_dir}/{slug}/spec.yaml (status: draft)"
  - "{specs_dir}/{slug}/pitch.md"
prerequisites: []
execution: inline
load:
  required:
    - reference/specs.md
    - reference/git.md
    - reference/knowledge.md
    - reference/presentation.md
  conditional:
    - when: writing the agreed pitch and lint-valid draft metadata
      files:
        - templates/spec/pitch.md
        - templates/spec/spec.yaml
    - when: user-facing wording or an upstream-standard decision needs the rule
      files:
        - reference/language.md
        - reference/engineering-standards.md
---

# spec-discuss

## Purpose

Reach agreement on the why / what / key design decisions through investigation
and discussion. Create the feature branch and persist the durable discuss output
as `spec.yaml (draft)` + `pitch.md`. Do not write implementation code.
For explicit concrete micro work, `plan` may skip discuss and create the
pitchless draft directly.

## Procedure

1. Investigate before asking. Always fix current state from **code**; never ask the user what code can answer. Read the constitution (`[constitution].project` / `[constitution].local`) and foundational context (`[context].product` / `[context].structure` / `[context].tech`) for current-state orientation (a map derived from code, refreshed via onboard / `refresh-context`). Read ADR (`[adr].decisions` / `[adr].pitfalls`) only on demand, as historical records of *why* (decision rationale, rejected alternatives, known pitfalls): load each store's generated `INDEX.md` first, then open only the records whose `area` intersects the spec's `surfaces` and whose `status` is active (use `mochiflow adr list` / `search` to filter); open superseded / deprecated records only when explicitly tracing supersession lineage. Re-verify any current-state claim against code. Prose is never the source of truth for current state.
2. `{slug} discuss` reads `_backlog/{slug}.md` as raw input (`reference/specs.md ## Backlog seeds`) when the slug exists only in `_backlog/`; if `{specs_dir}/{slug}/` already exists it re-opens that spec instead. Do not copy seed content straight into conclusions.
3. Organize the UI / data model / API / migration / error handling / testing decision tree internally and resolve it dependency-first.
4. One question at a time, each with a recommended answer, rationale, why the main alternatives are rejected, and impact.
5. Ask for specifics when answers are vague ("make it nice", "your call").
6. When every branch is resolved, create `{specs_dir}/{slug}/spec.yaml` with a
   lint-valid `draft` metadata set (`version`, `slug`, `title`, `type`,
   `surfaces`, `integration`, `risk`, `status`, `created`, `updated`). Resolve
   `type` conservatively from the agreed change because it determines the branch
   prefix (`reference/git.md ## Branch`). Write `{specs_dir}/{slug}/pitch.md`
   from `templates/spec/pitch.md` with the agreed Problem, Appetite, Solution,
   Rabbit Holes, No-gos, Alternatives Considered, and Open Questions.
7. Prepare the branch per `reference/git.md ## Branch`: fetch `origin`, then
   create/switch to `{prefix}/{slug}` **from `origin/{[git].base_branch}`** before
   committing — never from a stale local base. Warn when the local
   `{[git].base_branch}` is behind `origin/{[git].base_branch}` (this reduces the
   "forgot to report merge → new spec on a stale base" accident, independent of
   the provider). On a detached HEAD or non-spec branch, degrade gracefully with
   a clear message rather than crashing. If a raw seed exists at
   `{specs_dir}/_backlog/{slug}.md`, delete it in the same change so seed
   promotion is atomic. Run `mochiflow lint --spec {slug}`; fix any FAIL before
   committing.
8. Commit the discuss artifacts with a `docs(spec): ...` Conventional Commit and
   `Spec: {slug}` trailer. Stage only `spec.yaml`, `pitch.md`, and the seed
   deletion when present.
9. Present the agreement in the conversation language
   using plain labels for purpose / background / scope / decisions / assumptions
   / open questions / change impact. Internally this is the Decision summary;
   do not lead with internal headings. Then ask the user to choose the next step
   with a numbered choice card.

   - **Create the plan** (`plan` / `mochiflow-plan`) — proceed to
     `mochiflow-plan` in the same session.
   - **Create a resume prompt** (`resume` / `later`) — stop here; output a
     resume note (spec slug and path) that can be pasted into a new session to
     continue with `{slug} plan`.

## Stop conditions

- Do not move to the next branch while scope is undefined, contradictory, or unjustified.
- Do not proceed to `mochiflow-plan` with Open Questions unresolved (carrying them into plan as `[NEEDS-CLARIFICATION]` is allowed; resolve before `approved`).
- Do not commit until pitch-only `mochiflow lint --spec {slug}` passes.
- Keep scratch notes in the conversation only; persist the final agreement in
  `pitch.md`. Do not create `spec.md`, `design.md`, `tasks.md`, implementation
  code, PR metadata, fold, or archive artifacts.
