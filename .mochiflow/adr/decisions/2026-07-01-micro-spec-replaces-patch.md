---
id: 2026-07-01-micro-spec-replaces-patch
date: 2026-07-01
area: [cli]
spec: retire-patch-for-micro-spec-depth
status: active
---
## 2026-07-01 — micro spec replaces patch as the smallest delivery path

**Decision:** Retire `patch` as an active non-spec lane and make `micro` the
smallest spec depth. Micro is an artifact shape (`spec.yaml` + `spec.md`, no
`pitch.md`, `design.md`, or `tasks.md`), not a stored `depth` field. It is
eligible only for standard-risk, single-surface, `integration: none` work that
does not need design artifacts, human QA, or ADR folding. All depths, including
micro, use the normal feature branch, acceptance state, approve-PR gate, and PR
handoff.

**Why:** The old patch/no-PR paths optimized for small changes by bypassing the
spec lifecycle, which also bypassed AC Matrix evidence, accepted state, branch
and PR traceability, and consistent review/acceptance mechanics. Micro keeps the
small-change ergonomics while preserving the same audit path as every other
spec.

**Rejected:** Adding `depth` to `spec.yaml` was rejected to avoid schema and
field-vs-file drift. Keeping `mochiflow-patch` as a compatibility lane was
rejected because it would preserve the split lifecycle. Keeping a no-PR fast
path for micro was rejected because delivery traceability should not vary by
depth.

**Consequence:** `mochiflow-patch` is only a deprecated trigger that points
toward `plan`. Direct micro planning owns metadata confirmation, branch
creation/switch, and the first durable draft commit. Lint now accepts a
pitchless draft only when the existing metadata and file set match micro
eligibility; a forgotten-pitch standard draft with the same shape cannot be
mechanically distinguished without adding metadata.
