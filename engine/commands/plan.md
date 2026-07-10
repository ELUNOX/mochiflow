---
name: spec-plan
phase: plan
description: |
  mochiflow's plan phase. Turn an agreed pitch or explicit concrete micro
  request into spec.md, and grow design.md / tasks.md only as the change needs.
  Drive to a single human implementation approval gate. Activate on the explicit
  command `mochiflow-plan`, or natural phrasing like "仕様作って" / "プランして" /
  "計画作って". Does not implement.
artifacts:
  - "{specs_dir}/{slug}/spec.yaml"
  - "{specs_dir}/{slug}/pitch.md (standard-or-larger)"
  - "{specs_dir}/{slug}/spec.md"
  - "{specs_dir}/{slug}/design.md (conditional)"
  - "{specs_dir}/{slug}/tasks.md (conditional)"
prerequisites:
  - "existing spec path: {specs_dir}/{slug}/spec.yaml exists with status draft"
  - "standard-or-larger existing spec path: {specs_dir}/{slug}/pitch.md exists"
  - "direct micro path: explicit concrete request with no existing draft"
execution: inline
load:
  required:
    - reference/lifecycle.md
    - reference/specs.md
    - reference/risk.md
    - reference/verification.md
    - reference/git.md
    - reference/presentation.md
  conditional:
    - when: writing spec.yaml metadata
      files:
        - templates/spec/spec.yaml
    - when: direct micro or other trivial / narrow spec body
      files:
        - templates/spec/spec.micro.md
    - when: standard-or-larger spec body needing the fuller contract
      files:
        - templates/spec/spec.standard.md
    - when: the compatibility standard spec body is used
      files:
        - templates/spec/spec.md
    - when: design.md is required (reference/risk.md ## design.md required condition)
      files:
        - templates/spec/design.md
    - when: the change is multi-step and needs a task checklist
      files:
        - templates/spec/tasks.md
    - when: offering pre-approval review (risk >= elevated) or step-10 review actions
      files:
        - reference/review.md
    - when: creating a resume prompt for a new session
      files:
        - templates/handoff/build-session-prompt.md
    - when: an upstream-standard decision needs the rule
      files:
        - reference/engineering-standards.md
---

# mochiflow-plan

## Purpose

From the agreement, create `spec.yaml` and the spec documents the change needs,
and drive to human approval for implementation. Direct micro can start from an
explicit concrete request without `pitch.md`. Do not start implementation.

## Procedure

1. Resolve the input as either an existing draft spec or direct micro intake.
   Existing draft: read `{specs_dir}/{slug}/spec.yaml` and, for
   standard-or-larger work, `{specs_dir}/{slug}/pitch.md`; proceed only when
   metadata status is `draft`. If only a raw
   `{specs_dir}/_backlog/{slug}.md` item exists, stop and route back to
   `{slug} discuss` rather than inventing decisions. Direct micro: use only when
   the user gave an explicit concrete request and no draft spec already exists.
   Derive a proposed `slug`, `title`, `type`, `surfaces`, `integration`, and
   `risk` from the request, config, and code context; present that metadata for
   user confirmation or correction; reject duplicate active/_done/backlog slugs;
   write `spec.yaml` from `templates/spec/spec.yaml`; then create or switch
   `{prefix}/{slug}` from `origin/{[git].base_branch}`. Re-check any
   current-state claims against code before using them. When consulting ADR for
   *why*, load each store's generated `INDEX.md` first, then open only the
   records whose `area` intersects the spec's `surfaces` and whose `status` is
   active (`mochiflow adr list` / `search`); open superseded / deprecated
   records only when explicitly tracing supersession lineage. ADR is never the
   source of truth for current state.
2. Create or refine `spec.md` per `reference/authoring.md`, absorbing the pitch
   into `## Background and Design Rationale` when `pitch.md` exists. Use
   `templates/spec/spec.micro.md` for direct micro or other trivial / narrow
   changes and `templates/spec/spec.standard.md` for changes needing the fuller
   contract. `templates/spec/spec.md` is the compatibility standard template.
   Re-judge risk per `reference/risk.md` and update `spec.yaml` when the plan
   discovers a different risk / surface / integration. A pitchless micro shape
   is valid only for standard-risk, single-surface, `integration: none` work with
   no design-required impact, human QA, or ADR fold need; otherwise escalate in
   place before approval.
   Populate `spec.md ## QA Scenarios` with risk-appropriate QA attack dimension
   coverage following the mapping owned by
   `reference/risk.md ## QA attack coverage` — do not restate per-risk thresholds
   here. Record each scenario as a `QA-XX` row with a `Dimension` value (a
   non-applicable dimension is a row with a reasoned `N/A: <reason>`), and
   reference an attack that backs an AC from the AC Matrix `Planned test/QA` /
   `Evidence` column by its `QA-XX` id. Do not promote attacks to ACs or mint a
   separate attack-id scheme.
3. Create `design.md` only when `reference/risk.md ## design.md required condition` applies. When creating it, delete optional sections at creation time unless their condition applies (`## Workstreams` only for multiple workstreams / cross-surface, `## Integration Contract` only for `integration ≠ none`, `## Review Results` only for `risk ≥ elevated`, `## Integration Log` only when the risk table calls for it during build). Create `tasks.md` when multi-step. Let depth follow `reference/workflow.md ## Depth scaling` (a trivial change is spec.md only). Author tasks to be **session-recoverable** per `reference/authoring.md ## Session-recoverability`: because build may resume in a new session that has only the durable artifacts, committed code, and git trailers, write cross-task reasoning into `design.md` at plan time, and make each task shared by a file document that file's shared-state handling in its `Done`. This is plan authoring discipline enforced by reviewer S1 Internal Coherence judgment, not a new lint.
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
   command matters. For direct micro, after this first draft lint passes, commit
   the draft micro artifacts (`spec.yaml` and `spec.md`) with `docs(spec): ...`
   and a `Spec: {slug}` trailer before presenting approve-to-build.
7. Present readiness in conversation-language plain wording: what will change and
   what was checked. Then present a numbered choice card. The approval action is
   **Confirm the plan** (`approve plan` / `approved`); it means: edit
   `spec.yaml` `status: approved` and `updated` directly, re-run consistency
   checks, and commit the plan artifacts. It does not start implementation; the
   next-step card in step 10 decides whether to build or generate a resume
   prompt. Free-form correction feedback revises the plan and re-presents
   readiness instead of adding a separate "fix the plan" command.

   Review is a quality assist, not a delivery approval gate
   (`reference/workflow.md ## Delivery approval gates`): the two gates stay
   approve-to-build and approve-PR, and review never sets `status` by itself.
   The card's ordering depends on risk:
   - When `risk >= elevated`: present **Review** actions before **Confirm the plan**. The visible actions are **Review results** (`review` / `mochiflow-review`) and **Review and fix** (`review fix`) **before** **Confirm the plan**, so the recommended quality check can inform the approve-to-build decision instead of running only after the spec has locked to `approved`. **Review results** runs `mochiflow-review` on the draft spec with `agents/plan-auditor.md` (S0 Grounding, S1 Internal Coherence, S2 Impact & Regression, S4 Knowledge Confrontation, and Falsification with S3 `N/A`, no diff/changed-files input, per `reference/risk.md ## Review transport`). **Review and fix** runs `{slug} review fix`, uses `agents/plan-auditor.md`, limits edits to spec artifacts, updates the local review-fix ledger under `{install_dir}/state/{slug}/`, and re-presents the readiness card after verification. On `pass` / `pass-with-comments`, re-present **Confirm the plan**. On `fail` from result-only review, report the findings and stop: leave `spec.yaml` `status: draft`, make no plan commit, and let the user revise and re-review, choose review-and-fix, or confirm directly. Review stays optional — the user may choose **Confirm the plan** without taking review.
   - When `risk = standard`: present **Confirm the plan** as today; review is not
     offered pre-approval and remains available post-approval at step 10.
8. Re-run `mochiflow lint --spec {slug}` after setting `status: approved`; fix any FAIL before ending plan.
9. Commit the plan approval artifacts on the existing `{prefix}/{slug}` branch with a
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
    - When `risk >= elevated`: **Start implementation** (`build` /
      `mochiflow-build`) / **Create a resume prompt** (`resume` / `later`).
      Pre-approval review was already offered at step 7, so it is **not**
      re-offered here; ad-hoc `mochiflow-review` remains available on request but
      is not a listed action.
    - When `risk = standard`: **Start implementation** / **Review results** /
      **Review and fix** / **Create a resume prompt**. Compatibility **Review**
      (`risk = standard` only) maps to **Review results**.

    Behavior per choice:
    - **Review results** (`risk = standard` only; compatibility `review` /
      `mochiflow-review`) — run one result-only spec/design quality review, not
      code review, on the current spec. On `pass` / `pass-with-comments`,
      re-present **Start implementation** / **Create a resume prompt** only. On
      `fail`, report findings and stop; the user decides whether to fix and
      re-review or proceed.
    - **Review and fix** (`risk = standard` only; `review fix`) — run
      `{slug} review fix` with `agents/plan-auditor.md`, limit edits to spec
      artifacts, update the local review-fix ledger, verify, and re-present
      **Start implementation** / **Create a resume prompt** when the bounded
      fix round completes.
    - **Start implementation** — proceed to `mochiflow-build` in the same session.
    - **Create a resume prompt** — stop here; output a resume note (rendered from
      `templates/handoff/build-session-prompt.md`, includes `{slug}` and
      `{specs_dir}/{slug}/`) that can be pasted into a new session to continue.

## Stop conditions

- Do not proceed to standard-or-larger spec authoring without
  `{specs_dir}/{slug}/pitch.md`. Direct micro is the only pitchless plan path and
  requires explicit concrete input plus confirmed metadata before files are
  written. Do not use a raw `maturity: seed` backlog item as agreement.
- Do not create a direct micro draft when metadata cannot be derived safely or a
  duplicate active/_done/backlog slug exists.
- Do not ask for implementation approval while Open Questions remain unresolved
  unless they are explicitly carried as `[NEEDS-CLARIFICATION]`; resolve them
  before `approved`.
- Do not ask for implementation approval until `mochiflow lint --spec {slug}` passes on the draft spec.
- Do not set `status: approved` without the approve-to-build choice action or a
  compatibility approval word. When `risk >= elevated` and pre-approval review
  returned `fail`, do not set `status: approved` or create the plan commit until
  the user re-confirms (review is a quality assist, not a gate; it never sets
  `status` by itself).
- Do not touch implementation code / build / PR / archive.
- Continue to `mochiflow-build` in the same session only when the user chooses
  **Start implementation** (or `build`) from the step-10 choice card; do not
  require or suggest a slug for that same-session phrase. **Create a resume
  prompt** (or `later`) outputs the handoff prompt and stops. For
  `risk >= elevated`, pre-approval review is offered at step 7 and is not
  re-offered at step 10; for `risk = standard`, **Review** (or `review`) at step
  10 runs ad-hoc review and, on pass, re-presents build / resume.
