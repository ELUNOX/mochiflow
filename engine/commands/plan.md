---
name: spec-plan
phase: plan
description: |
  mochiflow's plan phase. Turn an agreed pitch into spec.md, and grow design.md /
  tasks.md only as the change needs. Drive to a single human implementation
  approval gate. Activate on the explicit command `mochiflow-plan`, or
  natural phrasing like "仕様作って" / "プランして" / "計画作って". Does not implement.
triggers:
  - mochiflow-plan
  - 仕様作って
  - プランして
  - 計画作って
trigger_patterns:
  - "{slug} plan"
artifacts:
  - "{specs_dir}/{slug}/spec.yaml"
  - "{specs_dir}/{slug}/pitch.md"
  - "{specs_dir}/{slug}/spec.md"
  - "{specs_dir}/{slug}/design.md (conditional)"
  - "{specs_dir}/{slug}/tasks.md (conditional)"
prerequisites:
  - "{specs_dir}/{slug}/spec.yaml exists with status draft"
  - "{specs_dir}/{slug}/pitch.md exists"
execution: inline
references:
  - reference/workflow.md
  - reference/risk.md
  - reference/authoring.md
  - reference/engineering-standards.md
  - templates/spec/spec.yaml
  - templates/spec/spec.md
  - templates/spec/spec.micro.md
  - templates/spec/spec.standard.md
  - templates/spec/design.md
  - templates/spec/tasks.md
  - templates/handoff/build-session-prompt.md
---

# mochiflow-plan

## Purpose

From the agreement, create `spec.yaml` and the spec documents the change needs,
and drive to human approval for implementation. Do not start implementation.

## Procedure

1. Resolve the pitch. Read `{specs_dir}/{slug}/spec.yaml` and
   `{specs_dir}/{slug}/pitch.md`; proceed only when metadata status is `draft`.
   If only a raw `{specs_dir}/_backlog/{slug}.md` item exists, stop and route
   back to `{slug} discuss` rather than inventing decisions. Re-check any
   current-state claims against code before using them. When consulting ADR for
   *why*, load each store's generated `INDEX.md` first, then open only the
   records whose `area` intersects the spec's `surfaces` and whose `status` is
   active (`mochiflow adr list` / `search`); open superseded / deprecated
   records only when explicitly tracing supersession lineage. ADR is never the
   source of truth for current state.
2. Check for slug duplicates under `_done/`, then create `spec.md` per
   `reference/authoring.md`, absorbing the pitch into
   `## Background and Design Rationale`. Use `templates/spec/spec.micro.md` for
   trivial / narrow changes and `templates/spec/spec.standard.md` for changes
   needing the fuller contract. `templates/spec/spec.md` is the compatibility
   standard template. Re-judge risk per `reference/risk.md` and update
   `spec.yaml` when the plan discovers a different risk / surface / integration.
   Populate `spec.md ## QA Scenarios` with risk-appropriate persona attack
   coverage (P1-P7) following the mapping owned by
   `reference/risk.md ## QA attack coverage` — do not restate per-risk thresholds
   here. Record each persona as a `QA-XX` row (a non-applicable persona is a row
   with a reasoned `N/A: <reason>`), and reference an attack that backs an AC from
   the AC Matrix `Planned test/QA` / `Evidence` column by its `QA-XX` id. Do not
   promote attacks to ACs or mint a separate attack-id scheme.
3. Create `design.md` only when `reference/risk.md ## design.md required condition` applies. When creating it, delete optional sections at creation time unless their condition applies (`## Workstreams` only for multiple workstreams / cross-surface, `## Integration Contract` only for `integration ≠ none`, `## Review Results` only for `risk ≥ elevated`, `## Integration Log` only when the risk table calls for it during build). Create `tasks.md` when multi-step. Let depth follow `reference/workflow.md ## Depth scaling` (a trivial change is spec.md only). Author tasks to be **worker-recoverable** per `reference/authoring.md ## Worker-recoverability`: because build may dispatch each task to a disposable worker that sees only `design.md` + the task row + committed code, write any cross-task reasoning into `design.md` at plan time, and make each task shared by a file document that file's shared-state handling in its `Done`. This is plan authoring discipline enforced by reviewer Stage 1 judgment, not a new lint.
4. Run `reference/authoring.md ## Consistency check` **exactly once**.
5. Remove all template residue before asking for approval. `lint` enforces these
   checks for expanded spec documents; placeholder-like text inside fenced code
   blocks or inline code spans is ignored so legitimate examples can remain:
   - no `{...}` placeholder remains;
   - no example-only row remains;
   - no `TBD` remains except in AC Verification Matrix fields intentionally owned by `build`;
   - no template-only HTML comment remains;
   - no "None" is used for a required section without a concrete reason.
6. Run `mochiflow lint --spec {slug}` and fix any FAIL before asking for approval.
   When talking to the user, call this a consistency check unless the exact
   command matters.
7. Present readiness in conversation-language plain wording: what will change and
   what was checked. Then present a numbered choice card whose approval action is
   **Confirm the plan** (`approve plan` / `approved`). The action means: edit
   `spec.yaml` `status: approved` and `updated` directly, re-run consistency
   checks, and commit the plan artifacts. It does not start implementation; the
   next-step card in step 10 decides whether to review, build, or generate a
   resume prompt. Free-form correction feedback revises the plan and re-presents
   readiness instead of adding a separate "fix the plan" command.
8. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix any FAIL before ending plan.
9. Commit the plan artifacts on the existing `{prefix}/{slug}` branch with a
   `docs(spec): ...` Conventional Commit and `Spec: {slug}` trailer. Stage only
   `spec.yaml`, `pitch.md` if it was corrected, `spec.md`, and conditional
   `design.md` / `tasks.md`. When fixing reviewer findings after a phase commit,
   create a separate `docs(spec): ...` commit with the same `Spec:` trailer. Do
   not amend the phase commit.
10. After the approved consistency check passes and the plan commit is created,
    ask the user to choose the next step with a numbered choice card. Use
    conversation-language action labels first and keep `review` / `build` /
    `later` as compatibility keywords.

    Display order depends on risk:
    - When `risk >= elevated`: **Review** (recommended; `review` /
      `mochiflow-review`) / **Start implementation** (`build` /
      `mochiflow-build`) / **Create a resume prompt** (`resume` / `later`).
    - When `risk = standard`: **Start implementation** / **Review** /
      **Create a resume prompt**.

    Behavior per choice:
    - **Review** — run `mochiflow-review` (spec/design quality review, not code
      review) on the current spec. On `pass` / `pass-with-comments`, re-present
      **Start implementation** / **Create a resume prompt** only. On `fail`, report findings
      and stop; the user decides whether to fix and re-review or proceed.
    - **Start implementation** — proceed to `mochiflow-build` in the same session.
    - **Create a resume prompt** — stop here; output a resume note (rendered from
      `templates/handoff/build-session-prompt.md`, includes `{slug}` and
      `{specs_dir}/{slug}/`) that can be pasted into a new session to continue.

## Stop conditions

- Do not proceed to spec authoring without `{specs_dir}/{slug}/pitch.md`. Do not
  use a raw `maturity: seed` backlog item as agreement.
- Do not ask for implementation approval while Open Questions remain unresolved
  unless they are explicitly carried as `[NEEDS-CLARIFICATION]`; resolve them
  before `approved`.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}` passes on the draft spec.
- Do not set `status: approved` without the approve-to-build choice action or a
  compatibility approval word.
- Do not touch implementation code / build / PR / archive.
- Continue to `mochiflow-build` in the same session only when the user chooses
  **Start implementation** (or `build`) from the step-10 choice card; do not
  require or suggest a slug for that same-session phrase. **Create a resume
  prompt** (or `later`) outputs the handoff prompt and stops. **Review** (or `review`)
  runs ad-hoc review and, on pass, re-presents build / resume.
