# Run build as an orchestrator dispatching disposable per-task workers to bound context

## Problem

`build` runs entirely inline on the main agent (router principle 5:
"implementation is inline; the main agent holds the whole picture"). For a
multi-task spec the main thread accumulates the full implementation transcript of
every task, so context grows monotonically — late tasks get slow and token cost
balloons (each turn reprocesses the whole accumulated history, roughly
quadratically). Observed directly while dogfooding `post-build-pr-close-flow`:
the back half of the work was slow and token-heavy purely from context growth,
not task difficulty.

discuss / plan must stay inline (design judgment). The unsolved gap is the
*implementation* context-growth problem, and `build` is the one phase that still
carries everything inline even though mochiflow already keeps state in files
(spec.md / design.md / tasks.md / AC Matrix) — exactly the precondition that makes
safe delegation possible.

## Appetite

A workflow-contract change scoped to execution mechanics: `router.md` principle 5
wording, `build.md` procedure, a new worker role doc, and reuse by `open`/`update`.
`elevated` risk. Worth it because context bloat is the dominant cost/latency
driver for non-trivial specs and gets worse the longer the spec.

## Solution

Make `build` an **orchestrator** that dispatches **disposable, sequential
per-task workers**. The orchestrator holds only the plan/contract (design.md /
AC Matrix), never the implementation transcript; each worker holds one task's
execution and returns a compact report.

Agreed shape:

- **Transport = one shared delegation primitive.** Reuse the existing
  `reference/risk.md ## Review transport` selection discipline (prefer
  `delegated` subagent when the runtime supports it, else degrade to today's
  inline build). Do not build a second transport for workers.
- **Two roles only.** Add a **new worker role** (write + verify + commit),
  distinct from the existing read-only `independent-reviewer`, which is reused
  as-is. No third "evaluator" role.
- **Principle 5 is refined, not relaxed.** Split the two ideas currently fused in
  one sentence: (A) *judgment / gates / integration / fold stay single-threaded
  on the top model* — invariant, made explicit and strengthened; (B) *execution*
  of a verified code-change task fans out to disposable workers via the shared
  transport when available, else inline.
- **Delegation threshold, decoupled from risk.** Orchestrator mode triggers when
  `tasks.md` exists with >= 2 open tasks; otherwise (taskless / micro / single
  task) build stays inline. Task count owns delegation; `risk` keeps owning
  reviewer cadence. Runtimes without a subagent mechanism stay inline regardless.
- **Workers always run on the top model.** Take Lever A (context isolation) only;
  no model downgrade (Lever B). Implementation quality is unchanged.
- **Quality gate is the existing reviewer at the existing cadence.** Reuse
  `independent-reviewer` at the unchanged `risk.md` cadence (standard = none,
  elevated = once after all tasks, critical = after each task). Delegation makes
  review structurally clean-context (the worker writes; a separate role reads),
  which is the configuration that makes a generator-verifier loop most effective.
  No new evaluator, no new cadence.
- **Isolate the transcript, not the filesystem.** Context isolation removes the
  *accumulated conversation history* from the main thread, not the worker's
  ability to read the repo. A worker starts with a fresh context (the task
  contract only) but has full repo read/grep/glob. Discovery (e.g. an `rg` sweep
  to find adjacent files) happens *inside the worker's context and is discarded
  on return*, so it never bloats the orchestrator.
- **Communication contract (principle only; fields go to plan).**
  - *context pack* (orchestrator -> worker): the minimum to execute one task as a
    contract — the relevant `design.md` slice, the single `tasks.md` row
    (`Files` / `Done` / `Stop` / AC refs), the verify command, and pointers to
    constitution / engineering-standards / pitfalls. `Files` is the **write-scope
    anchor and reading start point, not a read jail**. Read is repo-wide; write
    is contract-bounded — a worker needing to edit outside its declared surface
    returns `blocked` (the STOP rule below) instead of widening scope. The pack
    never carries other tasks' transcripts or conversation history.
  - *compact report* (worker -> orchestrator): the minimum to settle the AC Matrix
    and pick the next task without re-reading the worker's transcript — files
    changed, verify result + evidence pointer, commit ref, done/blocked + reason.
    Never the implementation narrative.
- **git is the accumulator; review reads the full diff from git.** The
  orchestrator never holds the cumulative diff in context, but the spec branch's
  per-task commits do. The mandatory risk-cadence review reconstructs the diff
  from git (`git diff origin/{base}...HEAD` for the completion-gate review; a
  task's own commit for a per-task `critical` review) and reads the changed code
  from scratch. Compact reports are **never** review evidence — extending
  `risk.md`'s "never conversation history as evidence" to the reports. This keeps
  the reviewer clean-context while still guaranteeing full-diff coverage.
- **Worker-recoverability invariant (plan-time authoring rule).** A worker sees
  only `design.md` + its task row + the committed code; it can read a prior
  worker's committed *code* but not its *reasoning*. So every fact needed to
  implement a task correctly must be recoverable from
  (`design.md` + the task row + reading committed code). Cross-task reasoning that
  inline build would carry implicitly is written into `design.md` at plan time
  (the practical form of "share the contract"). When a file appears in more than
  one task's `Files`, each such task's `Done` states how it leaves the shared
  structure consistent. Enforcement is plan authoring discipline + reviewer
  Stage 1 judgment, **not** a new deterministic lint (recoverability cannot be
  decided mechanically).
- **Worker commits its own task.** The worker follows the existing `build.md` 3e
  cadence (mark the `tasks.md` checkbox, then commit with one `Task:` trailer).
  Commit granularity stays one task per commit; the orchestrator records the AC
  Matrix from the report's commit ref.
- **STOP bubbles up, never improvised.** A worker that hits a build stop
  condition (out-of-scope change, new design decision) returns `blocked: reason`
  instead of deciding; the orchestrator routes back to `plan`. This enforces
  "judgment single-threaded" at runtime.
- **Phase boundaries.** `build` owns the worker mechanism. `open` reuses it only
  for the QA-`FAIL` rework loop; `update` reuses it for the PR-feedback code
  change; `close` delegates nothing. Acceptance, fold authoring, PR-body
  synthesis, human gates, and integration judgment stay inline on the top model.

## Rabbit Holes

- Parallel workers / git worktrees — out of scope; sequential-only also
  structurally avoids the parallel-conflict failure mode.
- Auto-scanning the repo to build the context pack — keep file selection grounded
  in the task's `Files`, not repo-wide inference.
- Re-deciding design inside a worker — workers execute the `design.md` contract;
  they do not make design choices (pre-mitigates the Cognition multi-agent
  failure mode).
- Orchestrator self-compaction (Ralph-loop style) for very large specs — possible
  because the AC Matrix is the true state, but treat as an optional follow-up,
  not core v1.

## No-gos

- No model downgrade (Lever B) in v1.
- No new "evaluator" role and no new review cadence; reuse `independent-reviewer`
  at the `risk.md` cadence.
- No parallel / concurrent workers and no git worktree.
- No relaxation of "judgment single-threaded"; only the execution transport
  changes.
- `patch` and `close` get no delegation path.

## Alternatives Considered

- **A fully independent worker transport** (separate from the reviewer
  transport) — rejected: it duplicates the selection / fallback / tool-agnostic
  logic (SSOT violation), and the clean-context separation that makes the
  generator-verifier loop work is already obtained by keeping the roles distinct
  under one shared transport. Industry harnesses (Claude Agent SDK) use a single
  dispatch primitive with role-specific definitions, not a transport per role.
- **Worker = the reviewer role reused** — rejected: the reviewer is read-only by
  contract; a worker must write, verify, and commit.
- **`risk >= elevated` as an independent delegation trigger** (seed decision 6) —
  rejected: a single-task elevated spec gains nothing from isolation, and it
  mixes the cadence axis with the delegation axis.
- **Per-task full review for every task, or a separate lighter evaluator** (seed
  decision 3) — rejected: explodes review count and duplicates the `risk.md`
  cadence SSOT; unnecessary when workers stay on the top model.
- **Selective per-task model downgrade** (seed decision 2) — out of scope: the
  observed pain is Lever A (isolation), not model price; downgrade trades quality
  and would require a per-task buy-back review.
- **"relax" principle 5 wording** — rejected in favor of "refine": the
  judgment-single-threaded invariant is preserved and made explicit; only the
  execution clause changes.

## Open Questions

- Exact compact-report field set and format (the principle is fixed; the schema
  is plan detail).
- The worker role doc shape under `engine/agents/` and its adapter generation
  (a new generated per-tool agent, analogous to `spec-independent-reviewer`).
- Orchestrator compaction fallback (Ralph-loop) for very large specs — include in
  v1 or defer to a follow-up.
- Whether any Rust CLI surface is touched (e.g. `mochiflow ready` / `status`) or
  the change is engine-markdown + adapter-generation only — confirm at plan.
