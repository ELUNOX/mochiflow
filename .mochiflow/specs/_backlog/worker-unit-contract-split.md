---
slug: "worker-unit-contract-split"
title: "Split worker contracts into build-task and rework units"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "build-orchestrator-subagent-execution"
source_phase: "review"
created: "2026-06-29"
updated: "2026-06-29"
---

# Split worker contracts into build-task and rework units

## Signal

The delegated build orchestrator added one worker role reused by `build`,
`open`, and `update`, but review exposed that the word "worker" covered
different execution units: an open `tasks.md` task during build, and a bounded
QA / PR-feedback fix during open or update.

## Why It Matters

Using one prose contract for both cases creates ambiguity around context packs,
write scope, checkbox ownership, `Task:` trailers, compact reports, blocked
handling, and commit conventions. Naming the units separately would make the
shared transport reusable without pretending every worker call has a `T-###`
task row.

## Evidence

- A build-task worker has a `tasks.md` row, `Files`, `Done`, `Stop`, checkbox
  tick, and one `Task:` trailer.
- An open/update rework worker usually has no open task, no checkbox to tick,
  and no `Task:` trailer; its unit is a bounded QA-`FAIL` or PR-feedback fix.
- Review suggested modeling the contract like an enum: `BuildTaskWorker` for
  task execution and `ReworkWorker` for open/update fixes, while keeping one
  shared delegation transport.

## Open Questions

- Should this be represented only in engine docs, or should the generated Kiro
  worker agent / prompt expose separate unit names?
- What exact compact-report fields should be shared, and which should be
  unit-specific (`task`, `unit`, `feedback_id`, `qa_item`, `commit`)?
- Should open/update use the same `agents/worker.md` role with an explicit
  `unit_kind`, or should they get a separate `agents/rework-worker.md` role?
