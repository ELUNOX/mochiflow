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
  - Agreed discussion (Decision summary)
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
---

# spec-plan

## Purpose

From the agreement, create `spec.yaml` and the spec documents the change needs,
and drive to human approval for implementation. Do not start implementation.

## Procedure

1. Check for slug duplicates under `{specs_dir}/` and `_done/`, then create `spec.yaml` (`status: draft`) and `spec.md` per `reference/authoring.md`. Judge risk per `reference/risk.md`.
2. Create `design.md` only when `reference/risk.md ## design.md required condition` applies. Create `tasks.md` when multi-step. Let depth follow `reference/workflow.md ## Depth scaling` (a trivial change is spec.md only).
3. If it came from a backlog seed, delete `_backlog/{slug}.md` after creating the spec documents and record the origin in spec.md `## 背景と設計判断`.
4. Run `reference/authoring.md ## Consistency check` **exactly once**.
5. Run `mochiflow lint --spec {slug}` and fix any FAIL before asking for approval.
   When talking to the user, call this a consistency check unless the exact
   command matters.
6. Present readiness in project-language plain wording: what will change, what
   was checked, and what approval is needed to start implementation. On an
   approval word (`reference/workflow.md ## Human gates`), update `status:
   approved`.
7. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix any FAIL before ending plan.

## Stop conditions

- Do not proceed to spec creation without a Decision summary or with Open Questions unresolved.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}` passes on the draft spec.
- Do not set `status: approved` without an approval word.
- Do not touch implementation code / branch / build / PR / archive.
- Continue to `spec-build` in the same session only when the user explicitly asks; otherwise guide `spec-build` in a separate session and stop.
