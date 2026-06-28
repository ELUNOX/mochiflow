---
id: 2026-06-26-choice-card-command-ux
date: 2026-06-26
area: [cli]
spec: choice-card-command-ux
status: active
---
## 2026-06-26 — choice-card-command-ux: localized choice cards with numbered aliases

**Decision:** Phase-completion prompts use localized, user-facing action labels
as the visible interface. Short numbered replies are ephemeral aliases for the
most recent unambiguous choice card in the current conversation, not durable
commands or stored state. Existing tokens such as `review`, `build`, `ship`,
`later`, `approved`, and `create pr` remain compatibility triggers, but they are
secondary to the displayed action. Selecting the localized plan-confirmation
action by label or number is the approve-to-build gate input; it confirms and
commits the plan without starting implementation. Resume-prompt actions are
shown only at discuss completion, plan confirmation, review completion, and
build completion, with build-to-ship handoff generated inline from slug/path.

**Why:** Internal workflow words were accurate for engine authors but ambiguous
for users. In particular, plan approval could read as immediate implementation,
and `later` hid that the system produces a prompt for a new session. Dispatching
the visible option keeps the UX understandable while preserving compatibility
for experienced users and older transcripts.

**Rejected:** Making numbers global commands (unsafe because the meaning changes
by phase); storing choice-card state in the CLI (unnecessary for a
documentation-level behavior contract); adding a dedicated PR-body-edit command
(turns ordinary text feedback into a workflow transition); hard-coding Japanese
labels as engine canonical actions (breaks non-Japanese conversations).

**Consequence:** Engine command and reference guidance now defines choice-card
presentation, localized labels, compatibility triggers, approval-gate dispatch,
and limited resume-prompt placement. Engine source edits require the usual
freeze, source upgrade, adapter drift check, and full CLI verification.
