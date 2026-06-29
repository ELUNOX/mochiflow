---
id: 2026-06-29-retire-build-worker-orchestrator
date: 2026-06-29
area: [cli]
spec: retire-build-worker-orchestrator
status: active
supersedes: 2026-06-28-build-orchestrator-disposable-workers
---
## 2026-06-29 - Retire write-capable build workers

**Decision:** `build`, `open`, and `update` keep implementation work on the main
agent. `build` executes task units inline, in order, with the existing task
checkbox, verification, and commit cadence. `open` QA-FAIL rework and `update`
PR-feedback fixes are bounded inline code changes that do not restart the build
phase, do not tick task checkboxes, and do not add `Task:` trailers. Delegation
is reserved for the read-only independent reviewer.

**Why:** Dogfooding showed that write-capable workers reduced the visible main
thread transcript but increased total work. Each worker started cold, re-read
overlapping artifacts and source, repeated expensive verification, and returned
a compact report that could not carry all implementation reasoning. MochiFlow's
durable artifacts, committed code, and git trailers already provide the
recoverable boundary without a write-capable subagent.

**Rejected:** keeping workers only for `open` / `update` rework, raising the
worker threshold, or expanding compact reports. Each option kept the cold-start
cost and a second write-capable execution contract. Removing delegated review was
also rejected: independent review is still useful and remains a separate
read-only role.
