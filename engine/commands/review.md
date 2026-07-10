---
name: spec-review
description: |
  mochiflow's ad-hoc review. On an explicit user request, run the read-only
  plan-auditor or change-reviewer against the active spec's latest artifacts via
  the review transport rule, regardless of risk level. Plain review is
  result-only and reports findings only. Fix mode runs bounded automatic rounds
  on the main agent. This is a non-phase command: it
  exposes ad-hoc review as a discoverable entry point and adds no delivery gate.
  Reports findings only in result-only mode; fix mode may edit in-scope files but
  never changes status, pushes, or creates PR metadata by itself.
delegate_to:
  - agents/plan-auditor.md
  - agents/change-reviewer.md
load:
  required:
    - reference/review.md
    - reference/presentation.md
  conditional:
    - when: reviewing a code-less spec before implementation
      files:
        - agents/plan-auditor.md
    - when: reviewing a change once code exists
      files:
        - agents/change-reviewer.md
---

# spec-review

## Purpose

Run an on-demand review of the active spec when the user asks for it,
independent of the automatic risk-cadence review. Plain `{slug} review` is
result-only and report-only. `{slug} review fix [N]` is the bounded automatic
fix path.

Supported forms:

- `{slug} review` — run exactly one read-only review and report findings.
- `{slug} review fix` — run one review/fix round.
- `{slug} review fix 1` — same as `{slug} review fix`.
- `{slug} review fix 2` — run at most two review/fix rounds.
- `{slug} review fix 3` — run at most three review/fix rounds.

Invalid forms are rejected before any reviewer runs. `{slug} review 2` is
ambiguous because result-only review has no numeric budget; use
`{slug} review fix 2` for automatic fixes. `{slug} review fix 0` and
`{slug} review fix 4` or higher are out of range; allowed fix rounds are 1, 2,
or 3.

## Procedure

Follow `reference/review.md ## Ad-hoc review`, `## Review-fix loop`, and
`## Review transport` (the single source of truth for this behavior):

1. Parse the requested mode. If the input is `{slug} review`, select
   result-only mode. If the input is `{slug} review fix` or
   `{slug} review fix N`, select fix mode with a fix-round budget of 1, 2, or
   3. Reject invalid forms before any review runs.
2. Run the selected canonical reviewer read-only using the review transport:
   prefer delegated subagent dispatch when available, and use inline reviewer
   role only when subagents are unavailable or dispatch fails for a
   runtime/tooling reason. The explicit review trigger is also the user's
   request to use delegated reviewer transport when the runtime requires that
   permission. Target the active spec's latest artifacts (`spec.md`, plus
   `design.md` / `tasks.md` when present). When no implementation exists yet
   (a code-less spec, e.g. ad-hoc review during plan), run
   `agents/plan-auditor.md`; when code exists, run
   `agents/change-reviewer.md`. Pass only the slug, the command path, a summary
   of the latest artifact, and a pointer to the spec — never the conversation
   history (`router.md` routing principle 5).
3. Report `Review profile: plan-auditor | change-reviewer`,
   `Reviewer mode: delegated | inline`, and the verdict / findings.
4. In result-only mode, stop after reporting. On High or Critical findings, ask whether to enter the appropriate build/fix flow. Do not fix inline as part of ad-hoc review in result-only mode.
5. In fix mode, the reviewer still remains read-only. The main agent applies at
   most one bounded fix pass for that round, verifies according to the current
   lifecycle context, updates the local review-fix ledger, and then either
   starts the next fresh independent review cycle or stops when the requested
   fix-round budget is spent. The number after `fix` is the maximum number of
   fix rounds, not the number of reviewer opinions. End after the final
   requested fix round; do not require a clean post-fix review.
6. On `pass` / `pass-with-comments` with no in-scope fix to apply, resume the
   interrupted flow.

## Presentation

- In user-facing speech, call the verdict the review result in the project
  language. Keep `Reviewer mode` and `delegated` / `inline` as a short
  `MochiFlow:` detail when useful, not as the main headline.
- Summarize findings by severity and required fixes. If High or Critical
  findings exist in result-only mode, state that fixes require a separate
  build/fix step. Avoid exposing routing terms unless the user asks how the
  review was run.
- For invalid numeric forms, use a concise correction: result-only review is
  `{slug} review`; automatic fixes are `{slug} review fix`, `{slug} review fix
  2`, or `{slug} review fix 3`.
- When review resumes a plan-confirmed flow or another `status: approved`
  implementation-ready context, present a numbered choice card with
  **Start implementation** (`build` / `mochiflow-build`) and
  **Create a resume prompt** (`resume` / `later`). Outside an approved context,
  report the review result and present only actions valid for the current
  lifecycle state.

## Stop conditions

- Do not change `spec.yaml` `status` or create PR metadata — review is
  non-transitional (`reference/review.md ## Ad-hoc review`).
- Result-only review must not edit files, stage, or commit. It is report-only.
- In fix mode, staging and commits follow the active lifecycle context
  (`plan` spec-artifact fixes, `build` post-completion bounded fixes, `open`
  rework, or `update` hold/finalize discipline); the review command itself does
  not invent a separate commit or push rule.
- Do not let fix mode exceed the parsed fix-round budget. Allowed fix rounds
  are 1, 2, or 3.
- Do not use this in place of the mandatory risk-cadence review during build
  (`reference/review.md ## Reviewer cadence`); the two are independent.
- Do not pass the conversation history to the reviewer; pass only the spec
  pointers per `router.md` routing principle 5.
- Do not choose inline while delegated subagent dispatch is available. If
  subagents are unavailable or dispatch fails for a runtime/tooling reason, do
  not stop; use inline reviewer role per `reference/review.md ## Review
  transport`.
