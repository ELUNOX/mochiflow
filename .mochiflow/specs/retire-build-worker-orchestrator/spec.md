# Retire the build worker/orchestrator

## Background and Design Rationale

MochiFlow currently describes `build` as an orchestrator that can dispatch
sequential disposable workers for individual `tasks.md` units. The same worker
role is reused by `open` for QA-`FAIL` rework and by `update` for PR-feedback
fixes. That contract is present in engine procedures, the shared delegation
transport, Kiro adapter generation, generated Kiro output, and conformance tests.

Dogfooding showed that worker delegation is a poor fit for context-budget
management. It gives the main agent a smaller transcript, but each worker starts
cold, re-reads overlapping code and artifacts, re-runs expensive verification,
and returns a compact report that cannot carry all implementation reasoning. The
artifact model already provides the durable boundary: a fresh session can resume
from `spec.yaml`, `pitch.md`, `spec.md`, `design.md`, `tasks.md`, committed code,
and git trailers without introducing a write-capable subagent.

The chosen design removes the worker/orchestrator execution path completely and
keeps only independent review delegation. That preserves the useful separation:
review delegation exists for independent judgment, while implementation stays on
the main agent with normal task-by-task commits and session handoff as the
context-pressure release valve.

Rejected alternatives:

- Keeping the worker only for `open` / `update` rework was rejected because it
  keeps the write-capable subagent, context pack, compact report, Kiro generated
  agent, and most of the maintenance cost.
- Raising the delegation threshold was rejected because the cold-start,
  redundant-discovery, repeated-verification, and lossy-report costs remain
  whenever delegation triggers.
- Adding a richer compact report was rejected because it moves toward rebuilding
  the main thread through report fields, while committed artifacts already carry
  state losslessly.
- Removing delegated review was rejected because review independence is still
  valuable and is a separate concern from build execution.

Origin: backlog seed `retire-build-worker-orchestrator`, created from dogfooding
the `review-gate-and-context-timing` build.

## User Story

As a MochiFlow maintainer, I want implementation work to run inline while
delegation remains reserved for independent review, so that the workflow is
simpler, less redundant, and easier to recover from durable artifacts.

## Scope

- In:
  - Rewrite `build` as inline-only task execution with the existing task commit
    cadence, final verification, AC Matrix settlement, and risk-cadence review.
  - Rewrite `open` and `update` rework paths to use inline bounded code-change
    discipline without a worker role, context pack, compact report, or
    `unit_kind`.
  - Narrow `reference/risk.md ## Review transport` to independent-reviewer
    transport only while preserving reviewer cadence and verdict freshness.
  - Remove `engine/agents/worker.md`, Kiro `spec-worker.json` template/output,
    adapter handling that special-cases the worker, and worker-specific tests.
  - Add deprecated-output self-heal for markered `.kiro/agents/spec-worker.json`
    while preserving markerless user files.
  - Reframe the plan authoring rule from worker-recoverability to
    session-recoverability.
  - Sync generated/frozen artifacts after editing repo-root `engine/`.
- Out:
  - No change to the two delivery approval gates.
  - No change to task-based commit cadence or `Task:` trailer requirements.
  - No change to accepted historical specs other than referencing them as
    historical context.
  - No new runtime-specific subagent API or replacement build subagent.

## Edge Cases

- Existing projects may already have a generated `.kiro/agents/spec-worker.json`;
  adapter generation must remove the markered generated file but preserve a
  markerless file that may be user-owned.
- `risk >= elevated` review must still use the delegated reviewer when
  available, including code-less plan-quality review and stale-verdict refresh
  after later code changes.
- `open` / `update` code rework runs after build is complete and may operate on
  an `accepted` in-review spec; the inline rework path must not call the
  `mochiflow ready` build-entry gate or revert status to `approved`.
- Source-engine edits must update `engine/MANIFEST.json`, the vendored
  `.mochiflow/engine/` copy, and generated adapter outputs according to the
  constitution.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL define `build` as inline-only implementation work with
  no worker/orchestrator dispatch path, while preserving task ordering,
  checkbox updates, per-task commits, final verification, and AC Matrix
  settlement.
- AC-02: THE SYSTEM SHALL keep independent-reviewer delegation available and
  SHALL preserve the existing risk-cadence reviewer requirements, plan-quality
  review mode, and verdict freshness rule.
- AC-03: WHEN `open` or `update` needs a bounded code fix after build, THE SYSTEM
  SHALL run that fix through inline code-change discipline without worker
  context packs, compact reports, `unit_kind`, checkbox updates, or `Task:`
  trailers.
- AC-04: THE SYSTEM SHALL remove the write-capable worker role and Kiro
  `spec-worker.json` generated target, and SHALL self-heal markered legacy
  `.kiro/agents/spec-worker.json` files while preserving markerless user files.
- AC-05: THE SYSTEM SHALL update conformance coverage and generated/frozen
  artifacts so no source, generated adapter output, or integrity manifest
  requires the retired worker contract.
- AC-06: THE SYSTEM SHALL replace worker-recoverability guidance with
  session-recoverability guidance based on durable artifacts, committed code,
  and git trailers.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | cli | Automated | Run conformance checks that inspect `build`, `router`, `risk`, `open`, and `update` text for the new inline-build/review-only contract. | A maintainer reading or invoking the workflow sees inline implementation and delegated review only, with no contradictory worker/orchestrator dispatch contract. |
| QA-02 | P2 | cli | Automated | Inspect the plan/build resume guidance and task authoring rules after the change. | A long implementation can be resumed from artifacts, committed code, and trailers without relying on a worker transcript or compact report. |
| QA-03 | P3 | cli | Automated | Exercise adapter self-heal with a markered `.kiro/agents/spec-worker.json`, a markerless file at the same path, and unrelated Kiro files. | Markered generated worker output is removed or reported as drift; markerless and unrelated user files are preserved. |
| QA-04 | P4, P5 | cli | Automated | Run adapter generation in a materialized project that previously had the worker output. | Existing generated state migrates cleanly to the two-file Kiro output set without deleting user-owned files or leaving markered residue. |
| QA-05 | P6 | cli | Automated | Run the full `cli` default verification profile after implementation and dogfood sync. | Rust tests, formatting, clippy, and `freeze --check` pass; existing reviewer, lifecycle, accept, PR, and adapter behavior still works. |
| QA-06 | P7 | cli | Automated | Search engine source, generated adapters, and conformance assertions for live worker/orchestrator build-delegation requirements. | Remaining worker references are limited to historical specs/ADRs or explicit deprecated-output handling, not active engine instructions. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- `mochiflow freeze`, `mochiflow upgrade --source engine`, adapter generation,
  consistency checks, and the configured `cli` verification profile are recorded
  as evidence.
- PR preparation records a new ADR decision that supersedes the active
  worker/orchestrator ADRs without rewriting the accepted historical specs:
  `2026-06-28-build-orchestrator-disposable-workers`,
  `2026-06-28-kiro-adapter-adds-worker-agent`, and
  `2026-06-28-worker-unit-kind-discriminator`. It also records that
  `2026-06-28-kiro-agent-tools-are-coarse-categories` is obsolete for the
  removed worker agent and that the earlier two-file Kiro adapter shape from
  `2026-06-24-kiro-adapter-always-on-steering` is effectively restored.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-05, QA-06; conformance assertions for inline-only `build` and router wording | `engine/commands/build.md`, `engine/router.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-02 | cli | automated | QA-01, QA-05; conformance assertions for review-only transport, cadence, and freshness | `engine/reference/risk.md`, `engine/commands/review.md`, `engine/agents/independent-reviewer.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-03 | cli | automated | QA-01, QA-05, QA-06; conformance assertions for inline `open` / `update` rework with no worker unit fields | `engine/commands/open.md`, `engine/commands/update.md`, `engine/reference/git.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-04 | cli | automated | QA-03, QA-04, QA-05; adapter unit tests and CLI conformance for deprecated `spec-worker.json` removal/preservation behavior | `engine/agents/worker.md`, `engine/adapters/kiro/manifest.toml`, `engine/adapters/kiro/agents/spec-worker.json.tpl`, `.kiro/agents/spec-worker.json`, `cli/crates/mochiflow-core/src/adapter.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | Planned deletions are represented in `tasks.md`. |
| AC-05 | cli | automated | QA-05, QA-06; `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, `cargo run --manifest-path cli/Cargo.toml -- freeze --check`, `mochiflow adapter generate --check` | `engine/MANIFEST.json`, `.mochiflow/engine/`, generated adapter outputs, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-06 | cli | automated | QA-02, QA-06; conformance assertions for session-recoverability wording and absence of worker-recoverability requirements | `engine/reference/authoring.md`, `engine/commands/plan.md`, `engine/agents/independent-reviewer.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
