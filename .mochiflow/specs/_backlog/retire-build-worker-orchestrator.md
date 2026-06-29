---
slug: "retire-build-worker-orchestrator"
title: "Retire the build worker/orchestrator: build inline, delegate only review"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_spec: "review-gate-and-context-timing"
source_phase: "build"
created: "2026-06-29"
updated: "2026-06-29"
---

# Retire the build worker/orchestrator: build inline, delegate only review

## Signal

The build orchestrator + disposable per-task worker model
(`build-orchestrator-subagent-execution`, `worker-unit-contract-split`) trades
main-agent context for a worse build profile: each worker cold-starts and
re-discovers the codebase, re-runs heavy verification, and the compact-report
handoff is lossy. Dogfooding it on `review-gate-and-context-timing` (3 highly
overlapping engine-doc tasks) made the redundant re-reading and per-task
`freeze`/MANIFEST churn obvious.

## Why It Matters

The context problem the worker solved is already solved — better and losslessly —
by **session boundaries** (resume prompts), because "artifacts are the state"
(router principle 2). Delegation should be reserved for **independence**
(independent review), not context budget. Proposed direction:

- `build` becomes **inline-only** again (warm context reuse, no cold start, no
  lossy handoff). Context budget is managed by session boundaries at task
  checkpoints.
- **Independent review stays a mandatory separate subagent** (independence is the
  point; it caught a High that an inline self-review would have missed). The
  review subagent git/cwd incident in this session was a host-tooling fluke, not
  an engine defect — no engine change needed there.
- `critical` per-task review still works: build inline, run the reviewer
  subagent against each task's commit.

## Evidence

- Worker woven into ~14 engine files: `engine/agents/worker.md` (30),
  `engine/commands/build.md` (9), `engine/reference/risk.md` (7),
  `engine/reference/authoring.md` (6), `engine/router.md` (4),
  `engine/commands/open.md` (4), `engine/commands/update.md` (4),
  `engine/adapters/kiro/agents/spec-worker.json.tpl` (4), plus
  `git.md`/`plan.md`/`close.md`/`independent-reviewer.md`/`MANIFEST.json`.
- `open.md` / `update.md` reuse the build worker mechanism for QA-`FAIL` rework /
  PR-feedback fixes, so removal must rewrite those reuse points too.
- Active ADRs to supersede: `2026-06-28-build-orchestrator-disposable-workers`,
  `2026-06-28-worker-unit-kind-discriminator`,
  `2026-06-28-kiro-adapter-adds-worker-agent`.
- Accepted specs (immutable history, not to be reset):
  `build-orchestrator-subagent-execution`, `worker-unit-contract-split`.

## Open Questions

- Scope: rewrite `build.md` to inline-only; remove `agents/worker.md` + kiro
  `spec-worker` adapter (+ manifest.toml/schema); rewrite `open.md`/`update.md`
  reuse to "inline build loop"; trim `risk.md ## Review transport` to review-only
  delegation; update `router.md` Verb Delegation; re-pin conformance fixtures.
- Re-anchor (do not delete) the `authoring.md` "worker-recoverable" rule as
  **session-recoverable** (resume reads only artifacts + committed code).
- Forward supersede via a new ADR (not git revert; the work is merged and woven).
- Likely risk: elevated–critical (schema/contract reversal + multi-surface +
  adapter removal); design.md + independent review required.
