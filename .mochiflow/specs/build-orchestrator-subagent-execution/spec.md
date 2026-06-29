# Run build as an orchestrator dispatching disposable per-task workers to bound context

## Background and Design Rationale

`build` runs entirely inline on the main agent (router principle 5). For a
multi-task spec the main thread accumulates the full implementation transcript of
every task, so context grows monotonically — late tasks slow down and token cost
balloons (each turn reprocesses the accumulated history, roughly quadratically).
Observed while dogfooding `post-build-pr-close-flow`: the back half slowed purely
from context growth. mochiflow already keeps state in files
(`spec.md` / `design.md` / `tasks.md` / AC Matrix), which is the precondition that
makes safe delegation possible — yet build is the one phase still carrying
everything inline.

Origin: backlog seed `build-orchestrator-subagent-execution`
(source `post-build-pr-close-flow`, close phase).

Key decisions agreed in discuss (full rationale in `pitch.md` and `design.md`):

- **One shared delegation transport, two roles.** Reuse the existing
  delegated→inline transport discipline (today owned by
  `risk.md ## Review transport`); generalize it into a shared mechanism used by
  both the read-only `independent-reviewer` and a new write-capable `worker`
  role. No second transport. Mirrors how the Claude Agent SDK uses one dispatch
  primitive with role-specific definitions (`prompt` + `tools` + `model`).
- **Isolate the transcript, not the filesystem.** Context isolation removes the
  accumulated history from the main thread, not the worker's ability to read the
  repo. A worker starts fresh (the task contract only) with repo-wide read; its
  write scope is bounded to the task contract.
- **git is the accumulator.** The mandatory risk-cadence review reconstructs the
  full diff from git and never uses compact reports as evidence — keeping the
  reviewer clean-context (the configuration that makes a generator-verifier loop
  most effective) while still guaranteeing full-diff coverage.
- **Principle 5 is refined, not relaxed.** Judgment / gates / integration / fold
  stay single-threaded (invariant, strengthened); only *execution* of a verified
  code-change task fans out.
- **No model downgrade in v1.** Take context isolation only; workers run on the
  top model, so implementation quality is unchanged.

## User Story

As an AI coding agent implementing a multi-task mochiflow spec, I want build to
dispatch each task to a disposable worker that returns only a compact report, so
that the orchestrator's context stays bounded and late tasks stay fast and
cheap without losing implementation quality.

## Scope

- In:
  - Refine `router.md` principle 5 and the Verb Delegation table.
  - Generalize the delegation transport in `risk.md` and add the
    reports-are-not-evidence / full-diff-from-git rules.
  - Add a new `engine/agents/worker.md` write-capable role doc.
  - Rewrite the `build.md` task loop as an orchestrator + sequential disposable
    per-task workers, with the ≥2-open-task delegation threshold and inline
    fallback.
  - Reuse the worker mechanism from `open` (QA-`FAIL` rework) and `update`
    (PR-feedback code change); state `close` delegates nothing.
  - Add the plan-time worker-recoverability authoring rule (`plan.md` /
    `authoring.md`).
  - Generate a kiro `spec-worker` agent (template + manifest + the
    `adapter.rs` full-file-agent predicate + tests).
  - Freeze, re-vendor, and regenerate adapters.
- Out:
  - Model downgrade / selective per-task downgrade (No-go for v1).
  - Parallel / concurrent workers and git worktrees (sequential only).
  - Orchestrator self-compaction (Ralph-loop) — deferred to a follow-up backlog
    seed; the AC Matrix already makes it safe to add later.
  - A new deterministic lint for worker-recoverability (review judgment owns it).
  - Any new CLI subcommand or change to the `mochiflow pr` / `ready` / `status`
    contracts.

## Edge Cases

- Runtime without a subagent mechanism: build degrades to today's inline path
  (no behavior change).
- Single-task or taskless / micro spec (`< 2` open tasks): stays inline; never
  dispatches.
- Worker discovers it needs an out-of-scope edit or a new design decision: it
  returns `blocked: <reason>` instead of widening scope; the orchestrator routes
  back to plan.
- Worker over-claims a PASS in its report: for `standard` risk, caught by the
  worker's deterministic verification plus the orchestrator's final `default`
  verification; for `risk >= elevated`, additionally by the mandatory full-diff
  review reading the real diff from git.
- Resume in a new session mid-orchestration: the true state is `tasks.md`
  checkboxes + `Task:` trailers in git (the existing build resume reconciliation
  is unchanged).

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL state in `router.md` that judgment / gates / integration
  / fold stay single-threaded (invariant) while a verified code-change task's
  execution MAY fan out to disposable workers, and the Verb Delegation `build`
  row SHALL read inline-or-delegated rather than inline-only.
- AC-02: THE SYSTEM SHALL document the delegation transport as a single shared
  delegated→inline mechanism reused by both the reviewer and the worker, with no
  second transport defined.
- AC-03: THE SYSTEM SHALL provide `engine/agents/worker.md` defining a
  write + verify + commit worker role distinct from the read-only reviewer,
  covering context-pack consumption, compact-report output, and STOP bubble-up.
- AC-04: WHEN `tasks.md` exists with at least two open tasks AND the runtime
  exposes a subagent mechanism, THE SYSTEM SHALL run build as an orchestrator
  dispatching sequential disposable per-task workers; otherwise it SHALL run
  build inline.
- AC-05: THE SYSTEM SHALL specify that a worker receives a context pack
  (relevant `design.md` slice, the single `tasks.md` row, the verify command,
  and constitution / standards / pitfalls pointers), reads the repo freely, and
  is bounded to a contract-scoped write (an out-of-scope edit returns `blocked`).
- AC-06: THE SYSTEM SHALL define the compact report fields (files changed, verify
  result + evidence pointer, commit ref, done/blocked + reason) and state the
  orchestrator settles the AC Matrix from the report without reading the worker
  transcript.
- AC-07: THE SYSTEM SHALL specify that the worker performs the existing build
  per-task commit cadence (mark the `tasks.md` checkbox, then one `Task:`-trailer
  commit per task), keeping one task per commit.
- AC-08: THE SYSTEM SHALL state that the mandatory risk-cadence review
  reconstructs the diff from git and never uses compact reports as evidence,
  while the `risk.md` reviewer cadence itself is unchanged.
- AC-09: THE SYSTEM SHALL state that workers run on the top model with no model
  downgrade in v1.
- AC-10: THE SYSTEM SHALL add a plan-time worker-recoverability authoring rule
  (every fact needed to implement a task is recoverable from `design.md` + the
  task row + committed code; a file shared by multiple tasks documents its
  shared-state handling in each task's `Done`) enforced by authoring discipline
  and reviewer judgment, not a new lint.
- AC-11: THE SYSTEM SHALL state the phase boundaries: `build` owns the worker
  mechanism, `open` reuses it only for the QA-`FAIL` rework loop, `update` reuses
  it for the PR-feedback code change, and `close` delegates nothing; acceptance,
  fold, PR-body synthesis, and human gates stay inline.
- AC-12: THE SYSTEM SHALL generate a kiro `.kiro/agents/spec-worker.json` agent
  (write-capable tools, top model) from a template + manifest entry, treat it as
  a full-file managed agent in `adapter.rs`, and keep `adapter generate --check`
  green; non-kiro adapters require no change because they load the engine docs
  (including `worker.md`) through their existing references.
- AC-13: THE SYSTEM SHALL keep the frozen surface coherent: `freeze` regenerates
  `engine/MANIFEST.json` + `contracts/contracts.lock`, `upgrade --source engine`
  re-vendors, `adapter generate` regenerates outputs, and the full `default`
  verification passes.

## QA Scenarios

> Coverage and evidence strength per `risk` are owned by
> `reference/risk.md ## QA attack coverage`. risk is `elevated`.

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | cli | Automated | Read build.md with a 2+ task spec: orchestrator mode is entered; assert router principle 5 wording matches the delegated build behavior (no contradiction). | Conformance test asserts build.md describes orchestrator/worker and router principle 5 is the refined judgment-vs-execution split. |
| QA-02 | P5, P6 | cli | Automated | Inline fallback: a `< 2`-task spec and a no-subagent runtime both keep today's inline build; existing reviewer transport still works. | Conformance test asserts the inline-fallback wording and that the existing reviewer cadence text is intact. |
| QA-03 | P3 | cli | Automated / AI-observed | Adversarial worker: out-of-scope write must return `blocked`; an over-claimed PASS must be caught by full-diff review + deterministic verify. | worker.md / build.md specify STOP bubble-up and full-diff-from-git review; assertion on that wording. |
| QA-04 | P4 | cli | Automated | Data integrity: orchestrator settles the AC Matrix from the compact report; `tasks.md` checkbox + `Task:` trailer cadence stays one task per commit. | build.md/worker.md specify the report→AC-Matrix settlement and the unchanged commit cadence; conformance assertion. |
| QA-05 | P6 | cli | Automated | Adapter regression: kiro generates `spec-worker.json`, the reviewer agent still generates, and `adapter generate --check` is green. | `adapter.rs` unit test covers the new full-file agent; `adapter generate --check` passes. |
| QA-06 | P2 | cli | Automated | Large multi-task spec stays sequential (no `[P]` parallel workers, single working tree). | build.md states sequential-only; assertion on that wording. |
| QA-07 | P7 | cli | Automated | Phase boundaries: open.md reuses the build worker only for QA-`FAIL` rework, update.md reuses it for the PR-feedback code change, close.md delegates nothing; no verb defines its own delegation. | Conformance test asserts the open/update reuse wording and the close-delegates-nothing statement. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix (`spec.md ## Verification Plan /
  AC Matrix`) with a done-eligible result token (`PASS`, `CONFIRMED`, or
  `N/A: <reason>`).
- Verification commands and results are recorded.
- The mandatory `independent-reviewer` verdict is recorded (`risk: elevated`).

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01 | `engine/router.md` | PASS | `conformance::router_principle_5_splits_judgment_from_execution` | principle 5 split + build/open/update rows |
| AC-02 | cli | automated | QA-02 | `engine/reference/risk.md` | PASS | `conformance::risk_transport_is_single_shared_delegation_mechanism` | one shared transport, both roles, no second |
| AC-03 | cli | automated | QA-03 | `engine/agents/worker.md` | PASS | `conformance::worker_role_doc_defines_write_verify_commit_contract` | write+verify+commit role distinct from reviewer |
| AC-04 | cli | automated | QA-01, QA-02, QA-06 | `engine/commands/build.md` | PASS | `conformance::build_is_orchestrator_with_inline_fallback_and_commit_cadence` | ≥2-task gate + subagent, else inline; sequential |
| AC-05 | cli | automated | QA-03 | `engine/agents/worker.md` | PASS | `conformance::worker_role_doc_defines_write_verify_commit_contract` | context pack, repo-wide read, contract-bounded write |
| AC-06 | cli | automated | QA-04 | `engine/agents/worker.md` | PASS | `conformance::worker_role_doc_defines_write_verify_commit_contract` | compact report fields; orchestrator settles Matrix |
| AC-07 | cli | automated | QA-04 | `engine/commands/build.md` | PASS | `conformance::build_is_orchestrator_with_inline_fallback_and_commit_cadence` | one task per commit, `Task:` trailer, write ownership |
| AC-08 | cli | automated | QA-03 | `engine/reference/risk.md` | PASS | `conformance::risk_transport_is_single_shared_delegation_mechanism` | full-diff-from-git; reports never evidence; cadence intact |
| AC-09 | cli | automated | worker.md no-downgrade assertion (conformance) | `engine/agents/worker.md` | PASS | `conformance::worker_role_doc_defines_write_verify_commit_contract` | top model, no downgrade |
| AC-10 | cli | automated | QA-04 | `engine/commands/plan.md`, `engine/reference/authoring.md` | PASS | `conformance::worker_recoverability_is_authoring_rule_not_lint` | authoring rule, reviewer-judged, not a lint |
| AC-11 | cli | automated | QA-07 | `engine/commands/open.md`, `engine/commands/update.md`, `engine/commands/close.md` | PASS | `conformance::phase_boundaries_reuse_build_worker_and_close_delegates_nothing` | open/update reuse build worker; close delegates nothing |
| AC-12 | cli | automated | QA-05 | `cli/crates/mochiflow-core/src/adapter.rs`, `engine/adapters/kiro/agents/spec-worker.json.tpl`, `engine/adapters/kiro/manifest.toml` | PASS | `conformance::behavioral_kiro_generates_spec_worker_agent_deterministically`; `mochiflow-core::adapter::tests::{kiro_agent_json_matches_reviewer_and_worker,kiro_worker_agent_is_full_file_managed_with_model_preserved}` | write-capable agent, full-file managed, `adapter generate --check` green |
| AC-13 | cli | automated | QA-05 | `engine/MANIFEST.json`, `contracts/contracts.lock` | PASS | `freeze --check` clean; `upgrade --source engine` re-vendored; `adapter generate --check` 0 drift; full `default` verification green | frozen surface coherent |
