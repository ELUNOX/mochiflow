---
id: 2026-06-30-derived-delivery-next-actions
date: 2026-06-30
area: [cli]
spec: conversational-delivery-guidance
status: active
---
## 2026-06-30 - Delivery next actions are derived, JSON-pinned, and status-rendered

**Decision:** Delivery guidance is surfaced as an observed, never-stored next
action. `delivery.rs` derives a `NextActionKind` (`report_merge` while in review;
`local_cleanup_pending` for a done-derived flat spec whose local feature branch
or `state/{slug}/` scratch still exists). The stable machine contract lives in
the generated `state/index.json` as `next_action` (`null` or `{kind, message}`)
plus a `local_cleanup_pending` boolean on every active/done entry; the human
`mochiflow status` board renders the hint line; the `INDEX.md` dashboard is
intentionally left without it. The PR-created merge-then-report next action is
printed by `mochiflow pr` on every success backend and follows
`conversation_output_language()` (`auto` -> artifact language for CLI-only
output).

**Why:** Most users drive delivery through the agent, not by memorizing
commands, so the flow must say what to do next in conversation. Deriving the hint
from local git facts (trailer/branch/scratch) keeps `accepted` as the last
asserted status and avoids inventing a persisted cleanup lifecycle state that
would have to be migrated and kept consistent. Pinning the contract in the JSON
board (which the agent reads) keeps it test-stable while leaving human prose
free.

**Boundaries:** No new persisted status, no `_done/` move, no `status: done`
write. The post-merge next action is never inserted into the PR body (body comes
solely from `--body-file`; the next action is local workflow guidance). Cleanup
detection degrades to no hint when local branch state cannot be inspected, and
legacy archived `_done/` specs (status `done`) never show a cleanup hint.

**Rejected:** persisting a `cleanup_pending` state in `spec.yaml` (adds a
lifecycle state and migration burden for a purely local, observable fact);
mirroring the hint into `INDEX.md` markdown (would force a second rendered
contract and churn the golden dashboard fixture while AC-04/AC-05 only require
the JSON `next_action.kind`); requiring exact `{slug} merged` syntax for the
normal path (re-introduces command memorization — kept only as the unambiguous
fallback alongside contextual bare merge-report routing that asks one
disambiguation question on multiple candidates and falls through to normal
routing on none).
