---
slug: "plan-approval-gate-wording"
title: "Fix approve-to-build gate presentation wording"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "ship"
source_spec: "ship-qa-experience"
created: "2026-06-23"
updated: "2026-06-23"
---

# Fix approve-to-build gate presentation wording

## Signal

When plan asks for human approval (delivery gate 1), the agent presents it as
"実装を開始してよいですか？（OK で approve-to-build gate を通過します）".
This conflates two separate actions: approving the spec (`status: approved`) and
choosing to start build (step 10 choice card). Approval makes build *possible*;
it does not start it. The wording implies a single action that does both.

## Why It Matters

- Misleading UX: the user thinks "OK" triggers implementation immediately, but
  actually a choice card (review / build / later) follows.
- Breaks the mental model of the two-step flow (approve → choose next action).

## Open Questions

- Should this be a plan.md Presentation rule addition, or a constitution/pitfall?
- Exact wording: "承認しますか？承認後に review / build / later を選べます" or
  something shorter?
