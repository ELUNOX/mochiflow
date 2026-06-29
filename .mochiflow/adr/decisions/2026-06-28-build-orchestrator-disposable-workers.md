---
id: 2026-06-28-build-orchestrator-disposable-workers
date: 2026-06-28
area: [cli]
spec: build-orchestrator-subagent-execution
status: active
---
## 2026-06-28 — build runs as an orchestrator dispatching disposable per-task workers

**Decision:** `build` becomes an **orchestrator** that, when `tasks.md` has ≥ 2
open tasks AND the runtime exposes a subagent mechanism, dispatches **sequential,
disposable per-task workers** over the existing delegation transport; otherwise
it runs the unchanged **inline** task loop (explicit fallback). The orchestrator
holds only the plan/contract (`design.md` / the AC Matrix), never the per-task
implementation transcript; each worker starts from a fresh context (a context
pack), reads the repo freely, writes within the task's `Files` (plus its own
`tasks.md` checkbox line), runs `default` verification, commits one task, and
returns only a **compact report**. The delegation threshold (≥ 2 open tasks) is
**decoupled from `risk`** — `risk` keeps owning reviewer cadence only.

**Why:** Inline build accumulated every task's transcript on the main thread, so
context grew monotonically and late tasks got slow/expensive (roughly quadratic
re-processing), observed while dogfooding `post-build-pr-close-flow`. mochiflow
already keeps state in files (spec/design/tasks/AC Matrix), which is the
precondition that makes safe delegation possible. Isolating the *transcript*
(not the filesystem) bounds orchestrator context while keeping implementation
quality identical.

**Key sub-decisions:**
- **One shared transport, two roles.** The `risk.md ## Review transport`
  selection (`delegated` → `inline`) is generalized into a single shared
  delegation transport used by both the read-only `independent-reviewer` and the
  new write-capable `worker`. No second transport (SSOT); roles differ by prompt
  + tool scope + permissions, mirroring a single dispatch primitive with
  role-specific definitions.
- **git is the accumulator.** The mandatory risk-cadence review reconstructs the
  full diff from git (`git diff origin/{base}...HEAD`, or a task's own commit for
  `critical`) and never uses compact reports (or conversation history) as
  evidence — keeping the reviewer clean-context while guaranteeing full-diff
  coverage.
- **Principle 5 refined, not relaxed.** Judgment / gates / integration / fold
  stay single-threaded on the top model (invariant, strengthened); only
  *execution* of a verified code-change task fans out.
- **No model downgrade (v1).** Workers run on the top model; context isolation is
  the only lever taken.
- **Reused, not redefined.** `open` (QA-`FAIL` rework) and `update` (PR-feedback)
  reuse the worker mechanism for their bounded fix (no `tasks.md` task, no
  `Task:` trailer, host-verb commit convention); `close` and `patch` delegate
  nothing. A worker that hits a stop condition returns `blocked` and the host
  phase decides the route (build → plan; open/update → host verb, a genuine new
  design decision → plan).
- **Verdict freshness.** A recorded reviewer verdict is valid only for the diff it
  reviewed; a later code change at `risk ≥ elevated` (incl. open/update rework)
  makes it stale and requires a fresh review before accept/push.
- **Worker-recoverability is an authoring rule, not a lint.** Every fact a task
  needs must be recoverable from `design.md` + the task row + committed code;
  enforced by plan authoring + reviewer Stage 1 judgment.

**Rejected:** a second, worker-only transport (duplicates selection/fallback
logic; the clean-context generator-verifier separation already comes from
distinct roles under one transport); reusing the read-only reviewer role as the
worker (a worker must write/verify/commit); `risk ≥ elevated` as the delegation
trigger (a single-task elevated spec gains nothing; mixes cadence with
delegation); per-task full review or a separate lighter evaluator (explodes
review count, duplicates the `risk.md` cadence SSOT); selective per-task model
downgrade (observed pain is isolation, not price; would need a buy-back review);
parallel workers / git worktrees (sequential-only also structurally avoids the
parallel-conflict failure mode); orchestrator self-compaction / Ralph-loop
(possible because the AC Matrix is the true state — deferred to a follow-up).
