---
name: spec-plan
phase: plan
description: |
  mochiflow's plan phase. Turn an agreed discussion into spec.yaml + spec.md, and
  grow design.md / tasks.md only as the change needs. Drive to a single human
  implementation approval gate. Activate on the explicit command `mochiflow-plan`, or
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
  - "{specs_dir}/{slug}/spec.md"
  - "{specs_dir}/{slug}/design.md (conditional)"
  - "{specs_dir}/{slug}/tasks.md (conditional)"
prerequisites:
  - Agreed discussion (Decision summary), or a trivial / well-scoped direct plan request that can synthesize one from user intent + code investigation
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
  - templates/spec/design.md
  - templates/spec/tasks.md
  - templates/handoff/build-session-prompt.md
---

# spec-plan

## Purpose

From the agreement, create `spec.yaml` and the spec documents the change needs,
and drive to human approval for implementation. Do not start implementation.

## Procedure

1. Confirm there is an agreed Decision summary. For a trivial / well-scoped direct plan request, first investigate the relevant code, then synthesize a minimal Decision summary from the user request + discovered facts instead of requiring a separate `discuss` run. Check for slug duplicates under `{specs_dir}/` and `_done/`, then create `spec.yaml` (`status: draft`) and `spec.md` per `reference/authoring.md`. Judge risk per `reference/risk.md`.
2. Create `design.md` only when `reference/risk.md ## design.md required condition` applies. Create `tasks.md` when multi-step. Let depth follow `reference/workflow.md ## Depth scaling` (a trivial change is spec.md only).
3. If it came from a backlog seed, delete `_backlog/{slug}.md` after creating the spec documents and record the origin in spec.md `## Background and Decisions`.
4. Run `reference/authoring.md ## Consistency check` **exactly once**.
5. Run `mochiflow lint --spec {slug}` and fix any FAIL before asking for approval.
   When talking to the user, call this a consistency check unless the exact
   command matters.
6. Present readiness in project-language plain wording: what will change, what
   was checked, and what approval is needed to start implementation. On an
   approval word (`reference/workflow.md ## Delivery approval gates`), update
   `status: approved`.
7. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix any FAIL before ending plan.
8. After the approved consistency check passes, present the next action as a
   handoff card: recommend a new session and include a copy-paste prompt rendered
   from `templates/handoff/build-session-prompt.md`. The handoff prompt must
   include `{slug}` and `{specs_dir}/{slug}/` because the new session has no
   conversation state. Also offer an explicit same-session continuation phrase
   in the project language, without the slug (for example, "continue with
   implementation" / "このまま実装して").

## Stop conditions

- Do not proceed to spec creation without a Decision summary, except for a trivial / well-scoped direct plan request where the Decision summary is synthesized from the user request + code investigation. Do not proceed with Open Questions unresolved.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}` passes on the draft spec.
- Do not set `status: approved` without an approval word.
- Do not touch implementation code / branch / build / PR / archive.
- Continue to `spec-build` in the same session only when the user explicitly
  asks in the active spec context; do not require or suggest a slug for that
  same-session phrase. Otherwise guide `spec-build` in a separate session with
  the handoff prompt and stop.
