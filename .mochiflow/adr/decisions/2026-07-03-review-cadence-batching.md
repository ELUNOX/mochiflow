---
id: 2026-07-03-review-cadence-batching
date: 2026-07-03
area: [cli]
spec: review-cadence-batching
status: active
---
## 2026-07-03 - Batch bounded fixes until push or accept

**Decision:** MochiFlow now treats bounded in-scope fixes after build completion
or during PR feedback as local held commits until an explicit push/accept
boundary. For `risk >= elevated`, the mandatory change review reruns at most
once at that boundary when code-changing commits exist beyond the recorded
`Reviewed through: <sha>` line.

**Why:** The old wording forced the same mandatory review cadence onto every
small fix request, even though the quality invariant only needs to hold before
the diff is pushed or accepted. Holding local commits preserves that invariant
while avoiding repeated full-diff reviews and repeated pushes during a single
feedback batch.

**Key sub-decisions:**
- The in-scope judgment stays prose-based: no task-structure change, no new AC,
  and no new design decision. Out-of-scope work still routes back to planning.
- Bare natural-language PR feedback is hold-only. Explicit update commands,
  slug-qualified update patterns, or unambiguous completion statements finalize
  the batch.
- `Reviewed through: <sha>` is a textual convention in `design.md`, placed on
  its own line below `Verdict:` so existing verdict parsing remains unchanged.
- The reviewer's input remains the full git diff; only trigger frequency
  changes.

**Rejected:** Reviewing each held fix immediately was rejected because it keeps
the overhead this change is meant to remove. Reviewing only the incremental
range since the last sha was rejected because it would weaken the existing
full-diff reviewer contract.
