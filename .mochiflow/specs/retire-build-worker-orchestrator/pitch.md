# Retire the build worker/orchestrator

## Problem

MochiFlow currently describes `build` as an orchestrator that can dispatch
sequential disposable workers for individual `tasks.md` units, and it reuses the
same worker mechanism for `open` QA-`FAIL` rework and `update` PR-feedback fixes.
That model is now woven through the engine procedure docs, the shared delegation
transport, the Kiro adapter output, and conformance tests.

Dogfooding showed the worker model solves the wrong problem. It reduces the main
thread transcript, but each worker cold-starts, re-discovers overlapping code,
re-runs heavy verification, and returns a lossy compact report. The project
already has a better context-boundary mechanism: committed artifacts plus session
boundaries. The durable state is in files, so resuming from a new session can
recover from `spec.md`, `design.md`, `tasks.md`, git history, and committed code
without adding a write-capable subagent role.

Delegation still matters for independence. The independent reviewer should stay
separate because its value comes from a fresh read-only judgment, not from
context-budget management.

## Appetite

This is worth a design-level cleanup rather than a narrow text edit. The change
removes a recently added execution path and generated adapter output, and it must
keep existing lifecycle, task commit cadence, AC Matrix settlement, review
cadence, and PR feedback flows coherent. Treat it as `critical` because it
reverses an engine contract and adapter generation behavior.

## Solution

Retire the write-capable worker model completely:

- Make `build` inline-only again. The main agent reads the approved artifacts,
  executes each task in order, verifies, marks the task checkbox, commits per the
  existing commit cadence, and records the AC Matrix. When context pressure is
  high, the supported escape hatch is a session boundary with a resume prompt,
  not a worker handoff.
- Keep independent review delegated when available. `reference/risk.md` should
  describe review transport only for `agents/independent-reviewer.md`, with
  inline reviewer mode as the fallback when subagents are unavailable or fail for
  tooling reasons.
- Rewrite `open` and `update` rework paths so bounded QA/PR feedback code
  changes run through the same inline build discipline, without a worker role,
  context pack, compact report, `unit_kind`, or `spec-worker.json`.
- Delete `engine/agents/worker.md` and remove the Kiro
  `.kiro/agents/spec-worker.json` generated target from source templates,
  adapter manifests, adapter drift/model-preservation logic, generated outputs,
  and conformance tests.
- Re-anchor the useful planning rule from "worker-recoverable" to
  "session-recoverable": every fact needed to resume implementation from a fresh
  session must live in durable artifacts, task rows, committed code, or git
  trailers. Do not keep the worker-specific context-pack framing.
- Supersede the active worker/orchestrator ADRs with a new decision record during
  PR preparation. Do not rewrite the already accepted historical specs.

## Rabbit Holes

- Do not keep a worker only for `open` or `update` rework. That preserves the
  write-capable subagent, context pack, compact report, Kiro generated agent, and
  most of the complexity being removed.
- Do not replace workers with another execution subagent name. The desired
  boundary is session recovery plus committed artifacts, not a renamed worker.
- Do not downgrade or weaken independent review while removing workers.
  Independent review remains the delegated role.
- Do not hand-edit generated adapter outputs as the source of truth. Change the
  repo-root `engine/` sources, run the required freeze / dogfood sync commands,
  and let generation update outputs.

## No-gos

- No changes to the two human delivery gates.
- No change to `risk` semantics except narrowing delegation transport to review.
- No change to the task-based commit cadence or `Task:` trailer requirement.
- No change to accepted historical specs except referencing them as history.
- No new adapter capability or runtime-specific subagent API.

## Alternatives Considered

- Keep the worker only for `open` / `update` rework. Rejected because it keeps
  the write-capable worker contract and generated Kiro agent while removing only
  part of the complexity.
- Keep the orchestrator but raise the delegation threshold. Rejected because the
  observed cost is structural: cold starts, redundant discovery, repeated heavy
  verification, and lossy reports still exist whenever delegation triggers.
- Add a richer compact report or more context to workers. Rejected because it
  moves the model toward reconstructing the main thread, while session
  boundaries already preserve state losslessly through files and commits.
- Remove delegated review too. Rejected because review delegation is about
  independent judgment, and that independence remains valuable.

## Open Questions

- None - ready for plan.
