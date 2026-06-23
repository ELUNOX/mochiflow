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
allowed_writes:
  - "{specs_dir}/**"
  - "{index}"
forbidden_writes:
  - "{write.allow}"
  - .git/**
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
   current-state claims against code before using them.
2. Check for slug duplicates under `_done/`, then create `spec.md` per
   `reference/authoring.md`, absorbing the pitch into
   `## Background and Design Rationale`. Use `templates/spec/spec.micro.md` for
   trivial / narrow changes and `templates/spec/spec.standard.md` for changes
   needing the fuller contract. `templates/spec/spec.md` is the compatibility
   standard template. Re-judge risk per `reference/risk.md` and update
   `spec.yaml` when the plan discovers a different risk / surface / integration.
3. Create `design.md` only when `reference/risk.md ## design.md required condition` applies. When creating it, delete optional sections at creation time unless their condition applies (`## Workstreams` only for multiple workstreams / cross-surface, `## Integration Contract` only for `integration ≠ none`, `## Review Results` only for `risk ≥ elevated`, `## Integration Log` only when the risk table calls for it during build). Create `tasks.md` when multi-step. Let depth follow `reference/workflow.md ## Depth scaling` (a trivial change is spec.md only).
4. Run `reference/authoring.md ## Consistency check` **exactly once**.
5. Remove all template residue before asking for approval:
   - no `{...}` placeholder remains;
   - no example-only row remains;
   - no `TBD` remains except in AC Verification Matrix fields intentionally owned by `build`;
   - no template-only HTML comment remains;
   - no "None" is used for a required section without a concrete reason.
6. Run `mochiflow lint --spec {slug}` and fix any FAIL before asking for approval.
   When talking to the user, call this a consistency check unless the exact
   command matters.
7. Present readiness in conversation-language plain wording: what will change, what
   was checked, and what approval is needed to start implementation. On an
   approval word (`reference/workflow.md ## Human gates`), edit `spec.yaml`
   `status: approved` and `updated` directly (there is no CLI transition command).
8. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix any FAIL before ending plan.
9. Commit the plan artifacts on the existing `{prefix}/{slug}` branch with a
   `docs(spec): ...` Conventional Commit and `Spec: {slug}` trailer. Stage only
   `spec.yaml`, `pitch.md` if it was corrected, `spec.md`, and conditional
   `design.md` / `tasks.md`.
10. After the approved consistency check passes and the plan commit is created,
   present the next action as a
   handoff card: recommend a new session and include a copy-paste prompt rendered
   from `templates/handoff/build-session-prompt.md`. The handoff prompt must
   include `{slug}` and `{specs_dir}/{slug}/` because the new session has no
   conversation state. Also offer an explicit same-session continuation phrase
   in the conversation language, without the slug (for example, "continue with
   implementation" / "このまま実装して").

## Stop conditions

- Do not proceed to spec authoring without `{specs_dir}/{slug}/pitch.md`. Do not
  use a raw `maturity: seed` backlog item as agreement.
- Do not ask for implementation approval while Open Questions remain unresolved
  unless they are explicitly carried as `[NEEDS-CLARIFICATION]`; resolve them
  before `approved`.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}` passes on the draft spec.
- Do not set `status: approved` without an approval word.
- Do not touch implementation code / build / PR / archive.
- Continue to `mochiflow-build` in the same session only when the user explicitly
  asks in the active spec context; do not require or suggest a slug for that
  same-session phrase. Otherwise guide `{slug} build` in a separate session with
  the handoff prompt and stop.
