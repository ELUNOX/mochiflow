---
name: spec-review
description: |
  mochiflow's ad-hoc review. On an explicit user request, run the read-only
  independent-reviewer against the active spec's latest artifacts via the review
  transport rule, regardless of risk level. This is a non-phase command: it
  exposes the existing ad-hoc review as a discoverable entry point; the behavior
  itself is governed by reference/risk.md ## Ad-hoc review. Activate on the
  explicit command `mochiflow-review`, or natural phrasing like "レビューして".
  Does not change status, create commits, or block approval.
triggers:
  - mochiflow-review
  - レビューして
trigger_patterns:
  - "{slug} review"
delegate_to:
  - agents/independent-reviewer.md
references:
  - reference/risk.md
  - reference/workflow.md
---

# spec-review

## Purpose

Run an on-demand independent review of the active spec when the user asks for
it, independent of the automatic risk-cadence review. This command is the
discoverable entry point only; it adds no review rules of its own.

## Procedure

Follow `reference/risk.md ## Ad-hoc review` and `## Review transport` (the
single source of truth for this behavior):

1. Run `agents/independent-reviewer.md` read-only using the review transport:
   delegated subagent when available, otherwise inline reviewer role. Target the
   active spec's latest artifacts (`spec.md`, plus `design.md` / `tasks.md` when
   present). Pass only the slug, the command path, a summary of the latest
   artifact, and a pointer to the spec — never the conversation history
   (`router.md` routing principle 5).
2. On HIGH (or Critical) findings, fix inline and re-run `mochiflow lint --spec
   {slug}` before resuming the interrupted flow.
3. Report `Reviewer mode: delegated | inline` with the verdict. On `pass` /
   `pass-with-comments`, resume the interrupted flow.

## Presentation

- In user-facing speech, call the verdict the review result in the project
  language. Keep `Reviewer mode` and `delegated` / `inline` as a short
  `MochiFlow:` detail when useful, not as the main headline.
- Summarize findings by severity and required fixes. Avoid exposing routing
  terms unless the user asks how the review was run.

## Stop conditions

- Do not change `spec.yaml` `status`, stage, commit, or create PR metadata —
  ad-hoc review is non-transitional (`reference/risk.md ## Ad-hoc review`).
- Do not use this in place of the mandatory risk-cadence review during build
  (`reference/risk.md ## Consequences`); the two are independent.
- Do not pass the conversation history to the reviewer; pass only the spec
  pointers per `router.md` routing principle 5.
- Do not stop merely because subagents are unavailable; use inline reviewer role
  per `reference/risk.md ## Review transport`.
