# Retire the build worker/orchestrator — Design

## Design Decisions

- **Build execution becomes inline-only.** The `build` procedure keeps the
  existing phase entry checks, branch discipline, task ordering, task checkbox
  update, per-task commit cadence, final verification, reviewer cadence, and AC
  Matrix settlement. It removes only the worker/orchestrator selection and
  handoff machinery.
- **Delegation transport becomes reviewer-only.** `reference/risk.md ## Review
  transport` remains the named section used by citations, but its content should
  describe only `agents/independent-reviewer.md` delegated/inline reviewer
  selection. The risk-cadence table and verdict freshness rule stay owned by
  `risk.md`.
- **Rework uses inline build discipline, not a worker unit.** `open` and
  `update` should say that bounded QA/PR feedback code changes reuse the build
  discipline: read, edit, verify, commit, refresh reviewer verdict when needed,
  and leave acceptance/PR metadata judgments inline. They should not mention
  worker context packs, compact reports, `unit_kind`, or worker dispatch.
- **Kiro worker output is deprecated, not silently orphaned.** Removing the
  `spec-worker.json` manifest entry is not enough. The adapter must treat
  markered `.kiro/agents/spec-worker.json` as deprecated generated residue:
  write mode removes it, check mode reports it as drift, and markerless files are
  preserved as user-owned. The real generated `.kiro/agents/spec-worker.json`
  should be removed by adapter generation during dogfood sync, not by a manual
  planned deletion in the source-edit task.
- **Recoverability shifts from worker to session.** The useful rule is that
  future execution can recover from durable artifacts and committed code. The
  plan authoring reference and reviewer checks should use
  session-recoverability language instead of worker-specific context-pack
  language.
- **Historical records are superseded forward.** Accepted specs remain as
  history. During PR preparation, add a new ADR decision that supersedes
  `2026-06-28-build-orchestrator-disposable-workers`,
  `2026-06-28-kiro-adapter-adds-worker-agent`, and
  `2026-06-28-worker-unit-kind-discriminator`; note that
  `2026-06-28-kiro-agent-tools-are-coarse-categories` is obsolete for the
  removed worker agent; and explain that the two-file Kiro adapter shape from
  `2026-06-24-kiro-adapter-always-on-steering` is restored while independent
  review remains.

Primary source basis: this change follows MochiFlow's own artifact-state
contract in `engine/router.md` and `engine/reference/workflow.md`; no external
framework or dependency API is introduced.

Risk basis: this is `elevated`, not `critical`. It changes engine and adapter
generation contracts across multiple files, so design and independent review are
required. It does not introduce a migration, schema break, auth/security impact,
or user-data loss path, and failure recovers by reverting the branch.

## Architecture

After implementation, there should be only one delegated role in the active
engine contract:

- `agents/independent-reviewer.md`: read-only review role, delegated when the
  runtime exposes subagents, inline reviewer role as fallback.

Implementation roles should be described as main-agent inline work:

- `build`: main agent implements approved tasks in order.
- `open`: main agent handles acceptance, fold, PR body, approval, and any bounded
  QA rework.
- `update`: main agent interprets PR feedback, applies bounded fixes, verifies,
  refreshes review when needed, and updates PR metadata.

The Kiro adapter generated output set returns to:

- `.kiro/steering/mochiflow.md`
- `.kiro/agents/spec-independent-reviewer.json`

The deprecated generated worker output is removed only when it carries the
MochiFlow generated marker.

## Data Model / Interfaces

- `engine/adapters/kiro/manifest.toml` no longer maps
  `.kiro/agents/spec-worker.json`.
- `cli/crates/mochiflow-core/src/adapter.rs` should identify only
  `.kiro/agents/spec-independent-reviewer.json` as a managed Kiro agent JSON
  with model preservation. The worker-specific "top model no preserve" branch is
  removed because the worker target is no longer generated.
- `DEPRECATED_KIRO_PATHS` should include `.kiro/agents/spec-worker.json` so old
  generated output is removed or reported consistently.
- `engine/MANIFEST.json` changes because repo-root `engine/` changes. The
  vendored `.mochiflow/engine/` copy changes after `mochiflow upgrade --source
  engine`.
- No Rust schema or persisted project metadata change is expected.

## Error Handling

- If adapter self-heal cannot remove a markered deprecated
  `.kiro/agents/spec-worker.json`, report the existing adapter error path rather
  than ignoring it.
- If a markerless `.kiro/agents/spec-worker.json` exists, preserve and report it
  as preserved; do not delete user-owned content.
- If implementation discovers that removing `spec-worker.json` requires a new
  adapter contract outside deprecated-output cleanup, stop and return to plan.
- If `open` / `update` rework needs a new design decision rather than a bounded
  fix, route back to plan as today.

## Test Strategy

- Update conformance tests from asserting worker/orchestrator behavior to
  asserting inline build, review-only delegation, inline rework, deprecated
  worker output cleanup, and session-recoverability.
- Update adapter unit tests to remove the worker model-preservation exception and
  to verify markered `spec-worker.json` self-heal.
- Run dogfood sync after editing repo-root `engine/`:
  `mochiflow freeze`, `mochiflow upgrade --source engine`, then adapter
  generation/check.
- Run the independent reviewer once after all tasks complete, using the full
  branch diff. T-001 was already reviewed while the plan still called for
  critical per-task review; keep that result as historical evidence, but do not
  continue per-task reviewer dispatch.
- Run the configured `cli` default verification profile:
  `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`.

## Workstreams

| Workstream | Surface | Responsibility | Depends on | Verification |
| --- | --- | --- | --- | --- |
| Engine procedure contract | cli | Rewrite active engine instructions for inline build/rework and reviewer-only delegation. | none | Conformance assertions plus text search for no active worker dispatch contract. |
| Adapter contract | cli | Remove generated Kiro worker output and add deprecated-output self-heal. | Engine procedure contract | Adapter unit tests and CLI adapter generation/check. |
| Recoverability wording | cli | Replace worker-recoverability with session-recoverability in authoring and reviewer checks. | Engine procedure contract | Conformance assertions for source set and no worker-specific task rule. |
| Generated/frozen sync | cli | Regenerate manifest, vendored engine, and adapter output after source edits. | All source edits | Full `cli` verification profile and `mochiflow adapter generate --check`. |

## Integration Contract

- Contract owner: MochiFlow engine and adapter generation.
- Request: generated adapters and agent instructions should expose only the
  standing MochiFlow instructions plus the read-only independent reviewer role.
- Response: Kiro generation writes `.kiro/steering/mochiflow.md` and
  `.kiro/agents/spec-independent-reviewer.json`; it no longer writes
  `.kiro/agents/spec-worker.json`.
- Compatibility: markered historical `.kiro/agents/spec-worker.json` is treated
  like other deprecated Kiro outputs and removed during generation; markerless
  files are preserved.
- Failure handling: check mode reports lingering markered worker output as drift;
  write mode reports removal errors through adapter errors.
- Verification: adapter unit tests, CLI conformance, dogfood adapter generation,
  and `freeze --check`.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Date: 2026-06-29
- Mode: plan-quality review, no implementation diff.
- Findings addressed before approval:
  - Medium: `tasks.md` originally made the critical-risk per-task Integration Log
    and independent-reviewer cadence explicit in `## Defaults`; this was later
    superseded by the approved elevated-risk final-review cadence.
  - Low: `tasks.md` now traces the ADR supersession / PR-prep fold requirement
    through T-006.

- Reviewer mode: delegated
- Verdict: pass
- Date: 2026-06-29
- Mode: post-implementation review for T-001 commit `ed1da75`.
- Reviewed task: T-001 `[AC-01, AC-02]`.
- Findings: none.

- Reviewer mode: inline
- Verdict: pass
- Date: 2026-06-29
- Mode: final full-branch review after T-006 sync; delegated subagent launch
  failed because the runtime usage limit was reached, so the reviewer transport
  fell back to inline review.
- Reviewed scope: branch diff since `origin/main` plus uncommitted T-006 sync
  changes.
- Findings: none.
- Notes: active worker/orchestrator implementation delegation is absent from the
  engine contract; remaining `spec-worker` references are deprecated-output
  cleanup and tests.

The T-001 review above remains recorded because it already happened before the
risk/cadence correction.

## Integration Log

### T-001 — Build and review transport contracts

- Removed the active build execution fan-out contract from `engine/commands/build.md`
  and `engine/router.md`; implementation is now described as inline task units
  on the main agent.
- Narrowed `engine/reference/risk.md ## Review transport` to the read-only
  independent reviewer. The transport still preserves delegated reviewer
  preference, inline reviewer fallback, plan-quality review, risk-cadence review,
  and verdict freshness.
- `engine/commands/review.md` already referenced only the independent-reviewer
  transport, so no source change was needed there.
- Verification before commit: `mochiflow lint --spec retire-build-worker-orchestrator`
  had no failures and only the expected warning for the not-yet-checked T-001;
  targeted search found no worker/orchestrator build-delegation terms in the
  T-001 active contract files.

### T-002 — Open/update inline rework

- Replaced `open` QA-`FAIL` rework and `update` PR-feedback code-change wording
  with bounded inline fixes that use build discipline without restarting the
  build phase or calling `mochiflow ready`.
- Preserved the state and commit boundaries: no task checkbox, no `Task:`
  trailer, accepted in-review state stays accepted, and stale reviewer verdicts
  still require refresh for `risk >= elevated`.
- Updated `reference/git.md` to describe these commits as bounded inline code
  changes rather than worker-mechanism changes.
- Verification before commit: targeted search found no worker, `unit_kind`,
  compact-report, or context-pack terms in the T-002 files.

### T-003 — Session recoverability

- Replaced worker-recoverability in `engine/reference/authoring.md` with
  session-recoverability: the recoverable source set is now `spec.md`,
  `design.md`, the task row, committed code, and git trailers.
- Updated `engine/commands/plan.md` so plan authoring writes cross-task reasoning
  for a later session rather than for a disposable worker.
- Updated `engine/agents/independent-reviewer.md` so plan-quality review judges
  task executability and session-recoverability.
- Verification before commit: targeted search in the T-003 files found no
  worker-recoverability, disposable-worker, context-pack, or orchestrator
  references.

### T-004 — Kiro worker generation removal

- Deleted the source worker role and Kiro worker template:
  `engine/agents/worker.md` and
  `engine/adapters/kiro/agents/spec-worker.json.tpl`.
- Removed `.kiro/agents/spec-worker.json` from the Kiro source manifest without
  manually deleting the generated working-tree output; T-006 adapter generation
  owns that cleanup.
- Updated adapter logic so only `.kiro/agents/spec-independent-reviewer.json` is
  a managed Kiro agent JSON with model preservation.
- Added `.kiro/agents/spec-worker.json` to deprecated Kiro outputs so markered
  generated residue is removed/reported while markerless files are preserved.
- Verification: `cargo test --manifest-path cli/Cargo.toml -p mochiflow-core adapter`
  passed. Targeted search showed remaining `spec-worker` references only in
  deprecated-output cleanup and tests.

### T-005 — Conformance coverage

- Replaced worker/orchestrator conformance assertions with coverage for inline
  build, reviewer-only transport, bounded inline open/update rework, retired
  Kiro worker generation, deprecated `spec-worker.json` cleanup, and
  session-recoverability.
- Updated the PR feedback routing test to assert bounded inline fixes instead of
  build-worker delegation.
- Verification: targeted conformance tests passed:
  `pr_feedback_routes_to_update_without_restore`,
  `inline_rework_lifecycle_and_adapter_lifecycle_are_specified`,
  `behavioral_kiro_retires_spec_worker_agent_and_self_heals`,
  `build_is_inline_and_review_transport_is_reviewer_only`,
  `session_recoverability_is_authoring_rule_not_lint`, and
  `worker_role_and_template_are_retired`.
- Full `cargo test --manifest-path cli/Cargo.toml -p mochiflow-cli --test conformance`
  was run and reached 142/150 passing; the remaining failures were expected
  `engine/MANIFEST.json` drift from unsynced engine source and are owned by T-006.

### T-006 — Generated sync, verification, and final review

- Ran `mochiflow freeze` to update `engine/MANIFEST.json`, then
  `mochiflow upgrade --source engine` to sync the vendored `.mochiflow/engine/`
  copy and generated adapter outputs.
- Ran adapter generation with the branch CLI:
  `cargo run --manifest-path cli/Cargo.toml -- adapter generate`; it removed the
  markered working-tree `.kiro/agents/spec-worker.json`. The installed
  `mochiflow 1.1.3` did not yet contain this branch's deprecated-output logic,
  so branch-CLI adapter checks are the authoritative evidence for T-006.
- Verified adapter sync with
  `cargo run --manifest-path cli/Cargo.toml -- adapter generate --check`, which
  reported `Summary: 0 drift, 0 failed`.
- Ran the full default verification profile:
  `cargo test --manifest-path cli/Cargo.toml`,
  `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`,
  `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`,
  and `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; all passed.
- Adjusted one CLI test summary expectation after worker retirement reduced the
  Kiro generated target count in a candidate-parent failure case, and formatted
  conformance tests.
- Final review ran as inline fallback because delegated subagent launch hit the
  runtime usage limit; verdict is `pass`.
- PR preparation must still fold durable ADR records named in
  `spec.md ## Completion Conditions` and run ADR validation during `open`; no
  generated `INDEX.md` should be staged.
