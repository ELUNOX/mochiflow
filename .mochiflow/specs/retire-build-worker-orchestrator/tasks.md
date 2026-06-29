# Retire the build worker/orchestrator — Tasks

Implementation Summary: Remove the write-capable build worker/orchestrator path, keep independent review delegation, and sync adapter/generated contracts.
risk: elevated
Stop Conditions:
- A replacement build subagent or second delegation transport appears necessary.
- Removing `.kiro/agents/spec-worker.json` cannot preserve markerless user files.
- Reviewer cadence or verdict freshness would need to be weakened.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Review cadence: run the independent reviewer once after all tasks complete,
  using the full branch diff. T-001 was already reviewed under the earlier
  critical-risk plan; that result remains recorded, but no further per-task
  reviewer runs are required.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02] Rewrite build and review transport contracts
  - Depends on: none
  - Files:
    - `engine/commands/build.md`
    - `engine/router.md`
    - `engine/reference/risk.md`
    - `engine/commands/review.md`
  - Done: `build.md` describes inline-only implementation with no worker/orchestrator dispatch branch; `router.md` routing principle #5 removes the execution fan-out invariant and states that judgment stays single-threaded while implementation stays inline and review may delegate; `risk.md ## Review transport` names only `agents/independent-reviewer.md`; reviewer cadence and verdict freshness remain intact; any frontmatter `delegate_to` entry for `agents/worker.md` is removed.
  - Stop: A second delegation transport or replacement build subagent appears necessary.
- [x] T-002 [AC-03] Rewrite open/update rework language for inline fixes
  - Depends on: T-001
  - Files:
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/reference/git.md`
  - Done: QA-`FAIL` and PR-feedback code changes are described as bounded inline fixes using build discipline; no worker context pack, compact report, `unit_kind`, checkbox tick, or `Task:` trailer is referenced; `accepted` in-review state is preserved; stale reviewer verdict refresh still applies for `risk >= elevated`.
  - Stop: Rework needs a new lifecycle state or a new task type.
- [x] T-003 [AC-06] Replace worker-recoverability with session-recoverability
  - Depends on: T-001
  - Files:
    - `engine/reference/authoring.md`
    - `engine/commands/plan.md`
    - `engine/agents/independent-reviewer.md`
  - Done: Plan authoring guidance says tasks and design must be recoverable from durable artifacts, committed code, and git trailers after a session boundary; reviewer plan-quality checks use the same session-recoverability concept; no active instruction requires a disposable worker's context pack.
  - Stop: The replacement wording cannot give a reviewer a concrete source set to check.
- [x] T-004 [AC-04] Remove Kiro worker generation and add deprecated-output cleanup
  - Depends on: T-001
  - Files:
    - deleted: `engine/agents/worker.md`
    - deleted: `engine/adapters/kiro/agents/spec-worker.json.tpl`
    - `engine/adapters/kiro/manifest.toml`
    - `cli/crates/mochiflow-core/src/adapter.rs`
  - Done: Kiro generation has no `spec-worker.json` target; adapter model-preservation logic applies only to the independent reviewer; markered `.kiro/agents/spec-worker.json` is listed as deprecated generated output and is removed/reported like other deprecated Kiro paths; markerless files at that path are preserved; the working-tree generated `.kiro/agents/spec-worker.json` is left for adapter generation in T-006 to remove.
  - Stop: Removing the generated worker target would delete markerless user content or require a schema change.
- [x] T-005 [AC-01, AC-02, AC-03, AC-04, AC-05, AC-06] Update conformance coverage
  - Depends on: T-001, T-002, T-003, T-004
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-core/src/adapter.rs`
  - Done: Tests that previously asserted worker/orchestrator behavior are removed or inverted to assert inline build, review-only delegation, inline rework, deprecated worker output self-heal, and session-recoverability; reviewer/lifecycle/adapter regression coverage remains.
  - Stop: Existing tests reveal an undocumented active worker dependency outside the planned files.
- [x] T-006 [AC-05] Sync engine, generated adapter output, and verification artifacts
  - Depends on: T-005
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - `.kiro/`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `.mochiflow/specs/retire-build-worker-orchestrator/spec.md`
    - `.mochiflow/specs/retire-build-worker-orchestrator/design.md`
    - `.mochiflow/specs/retire-build-worker-orchestrator/tasks.md`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, adapter generation/check, `mochiflow lint --spec retire-build-worker-orchestrator`, and the full `cli` default verification profile pass; adapter generation removes the markered working-tree `.kiro/agents/spec-worker.json` rather than a manual source-edit deletion doing it; the independent reviewer runs once on the full branch diff and the result is recorded in `design.md ## Review Results`; AC Matrix rows are updated with implementation paths, results, and evidence; `design.md ## Integration Log` includes a note that `open` must fold the ADR supersession records named in `spec.md ## Completion Conditions` and run ADR validation during PR preparation; no generated `INDEX.md` is staged.
  - Stop: `freeze --check` or adapter generation reports drift that cannot be explained by the planned worker retirement.
