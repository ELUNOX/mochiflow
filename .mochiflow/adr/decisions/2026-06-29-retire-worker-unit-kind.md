---
id: 2026-06-29-retire-worker-unit-kind
date: 2026-06-29
area: [cli]
spec: retire-build-worker-orchestrator
status: active
supersedes: 2026-06-28-worker-unit-kind-discriminator
---
## 2026-06-29 - Retire worker unit kinds

**Decision:** MochiFlow no longer defines `unit_kind` for implementation work.
The old `build-task` and `rework` discriminator disappears with the
write-capable worker role. Build task commits remain identified by `Task:`
trailers, while `open` / `update` rework commits carry only the `Spec:` trailer
because they happen after build is complete.

**Why:** `unit_kind` existed only to make a shared worker role branch honestly
between task execution and rework execution. With no worker context pack or
compact report, the discriminator has no consumer and would be contract residue.

**Rejected:** keeping `unit_kind` as prose for inline rework. Inline procedures
already have host-specific steps and commit conventions, so retaining a retired
worker field would add terminology without behavior.
