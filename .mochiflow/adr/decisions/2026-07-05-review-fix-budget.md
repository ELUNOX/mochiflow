---
id: 2026-07-05-review-fix-budget
date: 2026-07-05
area: [cli]
spec: review-fix-budget
status: active
---
## 2026-07-05 - Keep review result-only by default and add bounded fixes

**Decision:** MochiFlow keeps plain `{slug} review` as a single read-only,
result-only reviewer pass, and adds `{slug} review fix [1-3]` as the bounded
automatic-fix path. The numeric value is the maximum number of fix rounds, not a
request for multiple read-only opinions, and the loop stops after the requested
fix budget without forcing a clean post-fix review pass.

**Why:** The existing report-only review path remains useful when a user wants
findings for a human or another agent. At the same time, common review findings
are often small, in-scope corrections that the main agent can handle without
starting a separate workflow. Keeping both forms under `review` avoids a second
public verb while making the risk boundary explicit.

**Key sub-decisions:**
- Reviewers stay read-only in every mode. The main agent owns fixes,
  verification, commits, push boundaries, status changes, and PR metadata.
- Later fix cycles are fresh independent reviews of current artifacts or the
  current full diff. Prior findings, verdicts, reviewer summaries, local ledger
  contents, and conversation history are not reviewer input.
- Recovery state lives only in a gitignored local ledger under
  `{install_dir}/state/{slug}/review-fix.json`; it is for main-agent recovery,
  not durable evidence.
- Automatic fixes use the existing bounded-fix judgment: no task-structure
  change, no new AC, no new design decision, no spec split, and no unrelated
  work. Repeated unresolved findings or verification failure stop the loop.
- Code-present review fixes follow the current lifecycle context: post-build
  fixes are held until open/accept, and in-review fixes follow update's
  hold-by-default behavior.

**Rejected:** Adding a new public verb such as `revise`; treating `{slug} review
2` as a valid grammar; creating a write-capable reviewer or worker role; and
requiring every fix budget to end with an extra post-fix review pass.
