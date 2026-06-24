---
slug: "discuss-completion-choice-card"
title: "Add plan/later choice card to discuss completion"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "ship"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Add plan/later choice card to discuss completion

## Signal

After discuss completes, the agent improvises "plan に進めますか？" each time
because discuss.md step 9 says "guide the user toward plan" but does not define
the presentation format. plan.md step 10 has a structured choice card
(review/build/later) but discuss has no equivalent.

## Why It Matters

- Inconsistent UX across verbs (plan has a choice card, discuss does not).
- Agent must invent phrasing every time, leading to varying quality.
- Users don't know `later` is an option (they might not want to plan immediately).

## Proposed Solution

Add a structured 2-choice presentation to discuss.md step 9:
- **plan** — proceed to spec creation in this session
- **later** — output a resume note and stop

Match the pattern of plan.md step 10 (stable keywords + conversation-language
labels + "選択してください"). No `review` option (no reviewable artifact exists
yet at discuss completion).

Patch-eligible: discuss.md wording-only addition, no logic change, reversible.

## Open Questions

- None.
