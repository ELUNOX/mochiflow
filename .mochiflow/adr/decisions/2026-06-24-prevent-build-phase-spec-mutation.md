---
id: 2026-06-24-prevent-build-phase-spec-mutation
date: 2026-06-24
area: [cli]
spec: prevent-build-phase-spec-mutation
status: active
---
## 2026-06-24 — prevent-build-phase-spec-mutation: approved tasks are a build contract

**Decision:** During build, approved `tasks.md` structure is treated as an
implementation contract. Build may mark task checkboxes and record AC Matrix
result/evidence fields, but task additions, deletions, splits, renumbering,
reference changes, dependency changes, and meaningful `Files:` / `Done:` /
`Stop:` changes must return to plan for re-approval. One task may still cover
multiple related ACs with a compound reference such as `[AC-01, AC-02]`.

**Why:** A prior build fixed a close-out lint failure by adding a missing task
mid-build. That made a plan-phase correction indistinguishable from normal
implementation progress and weakened the approve-to-build gate.

**Rejected:** Snapshot/hash drift detection was considered but deferred as too
heavy for the current failure mode. A one-task-per-AC rule was also rejected
because related ACs often map naturally to one implementation step and should
not force artificial task splitting.

**Consequence:** `mochiflow-build` now states the stop condition explicitly,
authoring guidance and the task template document compound AC references, and
conformance tests lock compound task coverage and unknown-AC rejection.
