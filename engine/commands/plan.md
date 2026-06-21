---
name: spec-plan
phase: plan
description: |
  mochiflow's plan phase. Turn an agreed discussion into structured spec
  artifacts: spec.yaml, spec.md with the AC Matrix, and design.md / tasks.md
  when depth requires them. Drive to a single human implementation approval
  gate. Activate on the explicit command `mochiflow-plan`, or natural phrasing
  like "仕様作って" / "プランして" / "計画作って". Does not implement.
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

From the agreement, create the structured artifacts the change needs and drive
to human approval for implementation. Do not start implementation.

`spec.md` is the product contract. `design.md` is the technical contract when
required. `tasks.md` is the executable checklist when required. The AC Matrix in
`spec.md` is created during plan as the traceability ledger.

## Procedure

1. Confirm there is an agreed Decision summary. For a trivial / well-scoped
   direct plan request, first investigate the relevant code, then synthesize a
   minimal Decision summary from the user request + discovered facts instead of
   requiring a separate `discuss` run. If ambiguity is material, stop and ask
   for clarification through the expected flow.
2. Check for slug duplicates under `{specs_dir}/` and `_done/`. Create
   `spec.yaml` with `status: draft`, metadata consistent with the discovered
   surface / type / integration / risk, and no markdown frontmatter in the
   artifact documents.
3. Determine spec depth using `reference/workflow.md ## Depth scaling` and
   `reference/risk.md ## design.md required condition`.
4. Create or update `spec.md` from `templates/spec/spec.md` as the product
   contract:
   - Problem / Goal, Users / Actors, User Stories, Scope.
   - Requirements / Acceptance Criteria with stable `AC-01` IDs, Type,
     Priority, Requirement, and Verification.
   - Business Rules, Examples / QA Scenarios with stable `QA-01` IDs,
     Non-functional Requirements when needed, and Open Questions.
5. Create the `## Verification Plan / AC Matrix` in `spec.md` at plan time.
   Every AC must appear in the matrix with `Result` set to `UNVERIFIED` until
   build or ship records evidence. Use only canonical result values:
   `UNVERIFIED`, `PASS`, `PENDING_HUMAN`, `HUMAN_CONFIRMED`,
   `N/A: <reason>`, `FAIL`.
6. Create `design.md` when required. It captures decisions, alternatives,
   current-state scan, interface contracts, failure modes, rollout / rollback,
   observability, test strategy, review results, and integration log. Do not
   use it as an implementation diary.
7. Create `tasks.md` for standard-or-larger multi-step work. Each top-level
   task must be a markdown checkbox with a `T-001` style ID, AC/NFR/chore
   reference, `Depends on`, `Files`, `Done`, and `Stop` blocks. Use `[P]` only
   for tasks that can run in parallel without sharing files or dependencies.
8. If it came from a backlog seed, delete `_backlog/{slug}.md` after creating
   the spec documents and record the origin in `spec.md`.
9. Run cross-artifact analysis before approval:
   - Every AC has a Verification value.
   - Every AC appears in the AC Matrix.
   - Every AC is covered by at least one task, QA scenario, or explicit QA-only row.
   - Every QA scenario references one or more AC IDs.
   - Every task references AC, NFR, or chore reason.
   - Every task has Depends on, Files, Done, and Stop.
   - `design.md` decisions do not contradict `spec.md`.
   - risk, integration, and surfaces imply the required artifacts and sections.
   - no unresolved `[NEEDS-CLARIFICATION: ...]` remains before approval.
   - `spec.yaml` metadata is consistent with `spec.md`, `design.md`, and `tasks.md`.
   - language and surface placeholders do not hard-code a specific project unless configured.
10. Run `mochiflow lint --spec {slug}` and fix any FAIL before asking for
    approval. When talking to the user, call this a consistency check unless the
    exact command matters.
11. Present readiness in conversation-language plain wording: what will change, what
    was checked, and what approval is needed to start implementation. On an
    approval word (`reference/workflow.md ## Delivery approval gates`), update
    `status: approved`.
12. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix
    any FAIL before ending plan.
13. After the approved consistency check passes, present the next action as a
    handoff card: recommend a new session and include a copy-paste prompt
    rendered from `templates/handoff/build-session-prompt.md`. The handoff
    prompt must include `{slug}` and `{specs_dir}/{slug}/` because the new
    session has no conversation state. Also offer an explicit same-session
    continuation phrase in the conversation language, without the slug.

## Stop conditions

- Do not proceed to spec creation without a Decision summary, except for a
  trivial / well-scoped direct plan request where the Decision summary is
  synthesized from the user request + code investigation.
- Do not ask for implementation approval while Open Questions contain
  unresolved `NEEDS-CLARIFICATION` markers.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}`
  passes on the draft spec.
- Do not set `status: approved` without an approval word.
- Do not touch implementation code / branch / build / PR / archive.
- Do not add a new human approval gate. Cross-artifact analysis is a quality
  gate before the existing approve-to-build gate.
- Continue to `mochiflow-build` in the same session only when the user explicitly
  asks in the active spec context; do not require or suggest a slug for that
  same-session phrase. Otherwise guide `mochiflow-build` in a separate session with
  the handoff prompt and stop.
