---
name: independent-reviewer
role: legacy-reviewer-alias
description: |
  Legacy compatibility wrapper for the former public reviewer name. New
  references must use agents/plan-auditor.md for code-less plan audit and
  agents/change-reviewer.md for post-implementation review. This file exists so
  older adapter targets or prompts can route to the correct canonical contract
  during migration.
phases:
  - plan
  - build
canonical_commands:
  - commands/review.md
references:
  - agents/plan-auditor.md
  - agents/change-reviewer.md
  - reference/risk.md
---

# Legacy Reviewer Alias

`independent-reviewer` is no longer the public or canonical reviewer contract.
Use one of the canonical profiles:

- `agents/plan-auditor.md` for code-less spec/design/task/QA/ADR audit before
  implementation.
- `agents/change-reviewer.md` for post-implementation code review, including
  tests, code health, refactor safety, and behavior-preservation evidence.

Legacy mode names map as follows:

- `plan-quality mode` -> `plan-auditor`
- `post-implementation mode` -> `change-reviewer`

When invoked through an older adapter or prompt, choose the canonical profile by
whether an implementation diff exists:

- no diff / code-less spec: read and run `agents/plan-auditor.md`;
- implementation diff or changed files exist: read and run
  `agents/change-reviewer.md`.

The selected reviewer remains read-only, preserves repository grounding and
whole-tree impact / regression search, uses QA attack dimensions from
`reference/risk.md ## QA attack coverage`, and records the usual
`Reviewer mode: delegated | inline` plus `Verdict: pass | pass-with-comments |
fail`. Prefer also recording `Review profile: plan-auditor | change-reviewer`
when writing `design.md ## Review Results`.
