---
slug: "build-orchestrator-subagent-execution"
title: "Run build as an orchestrator dispatching disposable per-task workers to bound context"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "close"
created: "2026-06-27"
updated: "2026-06-27"
---

# Run build as an orchestrator dispatching disposable per-task workers to bound context

## Signal

`build` runs entirely inline on the main agent (router principle 5: "implementation
is inline; the main agent holds the whole picture"). For a multi-task spec, the
main thread accumulates the full implementation transcript of every task, so the
context grows monotonically — late tasks are slow and the token cost balloons
(every turn reprocesses the whole accumulated history, ~quadratically). Observed
directly while dogfooding `post-build-pr-close-flow`: the back half of the work
was slow and token-heavy purely from context growth, not task difficulty.

discuss / plan must stay inline on a high-tier model (design judgment), but the
*implementation* context-growth problem is unsolved.

## Why It Matters

Context bloat during build is the dominant cost/latency driver for non-trivial
specs, and it gets worse the longer the spec. mochiflow already keeps *state in
files* (spec.md / design.md / tasks.md / AC Matrix), which is exactly the
precondition that makes safe delegation possible — yet build is the one phase
that still carries everything inline. Closing this gap aligns build with the rest
of mochiflow's "artifacts are the state" design.

## Key insight: two separate levers

- **Lever A — context isolation:** run each task in a fresh/separate worker
  context that returns only a compact report. This is what removes the bloat and
  most of the cost (cost scales with tokens *processed*; isolation slashes the
  per-call context from "whole accumulated history" to "this task only").
- **Lever B — model downgrade:** run the worker on a cheaper model. Saves
  per-token price but risks implementation quality.

The observed pain is almost entirely Lever A. So take A without B by default —
the worker stays on the top model and quality is untouched, while bloat and cost
are solved structurally.

## Scope

Change only `build` execution (and reuse the existing reviewer delegation). Make
`build` an **orchestrator** that dispatches **disposable, sequential per-task
workers**:

```
build orchestrator (top model, main thread, holds only the plan/contract)
  for each task T in tasks.md (dependency order, ONE AT A TIME):
    dispatch worker(T) -> separate context:
      context pack = design.md + the T row + the task's target files + verify cmd
      worker implements T, runs verification, commits the task
      worker returns ONLY a compact report (files changed / AC evidence / verify result)
    orchestrator records the report in the AC Matrix, moves to next T
  after all tasks: full verification; independent review reads the FULL diff
```

- Main-thread growth becomes `O(num_tasks x small_report)` instead of
  `O(total_implementation_tokens)`.
- The orchestrator holds the *plan (contract)*, never the implementation
  transcript — that is the right granularity for "the whole picture".
- discuss / plan / open(integration, fold, PR judgment) stay inline on the top
  model. Only build-task execution and the (already-delegated) review move off
  the main thread.

## Why this is safe here when naive multi-agent fails

Cognition's failure mode (dispersed agents make conflicting implicit design
decisions — the Flappy Bird example) is pre-mitigated in mochiflow: `design.md`
is the *shared, plan-approved contract*, so a worker executes a contract rather
than re-deciding design. `tasks.md` (dependency-ordered, file-scoped, with
Done/Stop) is already the Anthropic "initializer -> coding agent" handoff. Review
is already a separate procedure.

## Phase boundaries (build / open / update / close)

The delegation unit is exactly one thing: **a verified code-change task**. `build`
owns that worker mechanism; the other verbs do not get their own separate
delegation — they reuse it only where code changes happen, and keep all
judgment / human gates / integration / fold inline on the top model.

- **build** — owns the orchestrator + per-task worker mechanism (the scope above).
- **open** — mostly inline (the part that must not be dispersed): the human QA
  round-trip (a), the fold/ADR authoring (b), the PR body synthesis (d), the
  approve-PR gate (e), and the deterministic `accept`/`pr` CLI steps (c/f). The
  **only** delegated part is the rework loop in step (a) 3e (a QA `FAIL` triggers
  a build-equivalent modify -> verify -> commit), which reuses the build worker
  mechanism.
- **update** — the interpretation of PR feedback and the decision of what to
  change stay inline; the actual code change reuses the build worker mechanism
  (update already routes code changes through the build loop); PR-metadata
  updates stay inline.
- **close** — no delegation. It is deterministic local hygiene (git/CLI) with no
  model reasoning or implementation; there is nothing to delegate.

Net: "execution fans out into workers; judgment / gates / integration stay
single-threaded." No verb gets a bespoke delegation path beyond the one build
owns, and the independent evaluator check applies uniformly to any worker code
change (build tasks, open rework, update feedback).

## Decisions (tentative)

1. **Default worker = the same top model as the orchestrator.** The value is
   context isolation, not model downgrade. Do not downgrade by default — this
   keeps implementation quality unchanged while still solving bloat and cost.
2. **Selective downgrade only, per task** — allowed only when ALL hold: the task
   is mechanical (no new design decision, follows an existing pattern, `Files`
   scoped), verification strongly covers it (`default` profile), and the task's
   surface risk is `standard`. Otherwise top model. When in doubt, top model
   (fail safe toward quality). Examples OK to downgrade: rename, file move,
   boilerplate, mechanical test addition.
3. **Quality is enforced, not assumed from the model.** Deterministic
   verification catches *correctness*; it does NOT catch design/maintainability
   (a weak worker can pass tests with over-built or messy code). So every worker
   diff is judged by a *separate* evaluator (the independent reviewer's Stage 2
   lens: maintainability / over-build / minimalism), never self-assessed
   (avoids self-preferential bias). Run an evaluator-optimizer loop (flag -> fix
   -> re-check) before advancing. Scale frequency by risk: every task for
   `elevated`, batched at task boundaries / completion for `standard`.
4. **Plan prescriptiveness gates safe downgrade.** A worker's output quality is
   bounded by how concretely `design.md` / `tasks.md` pin interfaces, adopted
   patterns, file layout, and "follow existing X". The thicker the design, the
   less judgment a worker needs — so design depth is what licenses downgrading a
   task. This is the practical form of "share the contract" (Cognition).
5. **No git worktree -> sequential only.** Workers run one at a time on the
   single working tree (commit, then next). This also structurally avoids the
   parallel-conflict failure mode. No parallel fan-out.
6. **Threshold to delegate at all.** Micro / taskless specs stay inline
   (dispatch overhead outweighs benefit). Delegate when `tasks.md` exists with a
   non-trivial task count, or `risk >= elevated`.
7. **Orchestrator compaction fallback.** On very large specs, since the AC Matrix
   (a file) is the true state, the orchestrator may compact its own thread
   between tasks (Ralph-loop style) without losing state.
8. **One delegation unit, reused across verbs (phase boundaries).** The only
   delegated work is a verified code-change task, owned by `build`. `open` reuses
   it solely for its rework loop (QA `FAIL` fix); `update` reuses it for the
   PR-feedback code change; `close` delegates nothing (deterministic hygiene).
   Acceptance, fold authoring, PR-body synthesis, human gates, feedback
   interpretation, and integration judgment always stay inline on the top model.

## Evidence

- Observed this session: inline build of `post-build-pr-close-flow` slowed and
  grew token cost in its back half from context accumulation alone.
- router principle 5 (`engine/router.md`): "State lives in files; implementation
  is inline ... Review is the only separated procedure." Build is the gap.
- Industry research:
  - Claude Code subagents — separate context window, return only the result:
    https://claude.com/blog/how-and-when-to-use-subagents-in-claude-code
  - Anthropic, harness design for long-running apps (initializer -> coding agent,
    artifacts carry context):
    https://www.anthropic.com/engineering/harness-design-long-running-apps
  - Anthropic, effective harnesses for long-running agents (compaction):
    https://anthropic.com/engineering/effective-harnesses-for-long-running-agents
  - Ralph loop — fresh context each iteration, progress in files + git:
    https://paulhoekstra.substack.com/i/191671173/claude-managed-agents
  - Failure modes (agentic laziness / self-preferential bias / goal drift):
    https://www.theneuron.ai/explainer-articles/claude-code-dynamic-workflows-explained-claude-can-now-build-its-own-workflow-around-a-task/
  - Cognition, "Don't Build Multi-Agents" (single-thread + context engineering;
    share the contract, not parallel guesses):
    http://cognition.ai/blog/dont-build-multi-agents
  - Cognition follow-up, "Multi-Agents: What's Actually Working" (read-only /
    bounded / verifiable tasks delegate well):
    https://old.cognition.ai/blog/multi-agents-working

## Open Questions

- Transport: reuse the existing reviewer subagent dispatch mechanism for workers,
  or define a distinct worker role? How does the inline fallback behave when no
  subagent mechanism is available (degrade to today's inline build)?
- What exactly is in the worker "context pack", and how does the orchestrator
  pick the relevant files without leaking the whole repo into the worker?
- Compact report shape — minimal fields that let the orchestrator settle the AC
  Matrix and decide the next task without re-reading the worker's transcript.
- How is "mechanical vs design-bearing" classified per task — a `tasks.md`
  annotation set at plan time, or inferred by the orchestrator? Mis-classification
  risk argues for plan-time tagging.
- Does `build.md` / `router.md` need to relax principle 5 explicitly, and how is
  the commit-per-task cadence reconciled with worker-side commits?
- Risk level: likely `elevated` (workflow contract change across build, multi-step,
  affects every spec's implementation path). Confirm at plan time.
- Interaction with `reference/risk.md` reviewer cadence — is the per-task
  evaluator the same as the risk-cadence independent review, or a lighter
  in-loop check feeding the mandatory review at the end?
