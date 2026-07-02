# Support local-only MochiFlow specs when .mochiflow is gitignored

## Background and Design Rationale

MochiFlow currently has one delivery persistence assumption: spec artifacts are
reviewed through git. That assumption is correct for tracked `.mochiflow/`
repositories, where `accept` creates an accepted close-out commit and `pr`
requires that commit plus a `Spec:` trailer before pushing.

Some repositories intentionally gitignore `.mochiflow/`. In those repositories,
the code changes still live on the feature branch, but the spec, AC Matrix, and
ADR/fold artifacts remain local. The current flow fails late: `accept` may pass
verification and then fail while staging ignored artifacts, and `pr` blocks on a
committed accepted spec that the repository deliberately does not have.

The chosen v1 design keeps `.gitignore` as the source of truth. If the concrete
spec artifact path is ignored, the repository is in local mode for that spec; if
it is not ignored, tracked mode keeps the existing close-out and preflight
contract. This avoids a new config knob that could disagree with the repository
policy, while still making the two modes explicit in CLI messages, engine
guidance, and docs.

## User Story

As a developer using MochiFlow in a repository that gitignores `.mochiflow/`, I
want acceptance and PR handoff to preserve verification and review evidence
without forcing ignored spec artifacts into git, so that I can keep local-only
specs while still producing reviewable PRs.

## Scope

- In:
  - Shared CLI detection of tracked versus local spec persistence.
  - Local-mode `accept` success path that verifies and updates local artifacts
    without staging or committing ignored `.mochiflow/` paths.
  - Local-mode `pr` preflight that uses local accepted state and evidence
    instead of committed spec/trailer evidence.
  - Local-mode derived delivery state and post-merge cleanup when no committed
    `Spec:` trailer exists.
  - Tracked-mode regression coverage for the existing accepted close-out commit
    and `Spec:` trailer preflight.
  - Engine guidance and user docs for tracked mode, local mode, constraints,
    recommended usage, and migration.
  - PR body guidance so local-mode handoffs include verification evidence,
    review result, and durable decision summary.
- Out:
  - A configurable `specs.mode` override.
  - A remote spec store, database, or provider-side persistence backend.
  - Changes to `contracts/pr-request.schema.json`.
  - Any change to `status: done`, `_done/` moves, or generated `INDEX.md`
    staging rules.
  - Forcing `.mochiflow/` artifacts into git in local mode.

## Edge Cases

- `.mochiflow/` is ignored at the repository root.
- Only `.mochiflow/specs/` is ignored while other `.mochiflow/` paths are
  tracked.
- Linked ADR records are ignored in local mode.
- Local mode has ignored spec changes while tracked source files are clean.
- Local mode has unrelated tracked changes when `accept` or `pr` starts.
- The current branch is the base branch, detached, or does not match the source
  branch expected for the spec.
- Head and base differ, but head is not ahead of base.
- A local-mode PR is manually merged with no `Spec:` trailer reachable from
  base.
- The local/remote source branch is deleted before local cleanup.
- Local `spec.yaml` is not `accepted` at PR handoff.
- Accepted evidence is incomplete, a Matrix row is provisional, or the required
  review result is missing.
- Tracked mode operates in a repository whose `.mochiflow/state/` is not ignored.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL expose one shared CLI decision for a spec's persistence mode, resolving tracked mode when the configured spec artifact path is not ignored and local mode when the configured `.mochiflow/` or specs path is ignored.
- AC-02: WHEN `mochiflow accept <slug>` runs in local mode with final verification, lint, AC Matrix, and required review result complete, THE SYSTEM SHALL update local acceptance artifacts, set `spec.yaml` to `status: accepted`, skip close-out commit/spec staging/ADR staging, print the skipped-git reason, and exit successfully.
- AC-03: IF `mochiflow accept <slug>` runs in tracked mode, THEN THE SYSTEM SHALL preserve the existing behavior of staging only the target spec directory plus linked ADR records and creating the accepted close-out commit with a `Spec: <slug>` trailer.
- AC-04: WHEN `mochiflow pr --spec <slug>` runs in local mode, THE SYSTEM SHALL NOT require a committed accepted spec or `Spec:` trailer and SHALL instead require a clean tracked working tree, current source branch, `base != head`, head ahead of base, local accepted state, complete verification evidence, and required review result.
- AC-05: WHEN `mochiflow pr --spec <slug> --dry-run` runs in a local-mode fixture, THE SYSTEM SHALL NOT fail because the accepted spec is uncommitted or lacks a `Spec:` trailer.
- AC-06: WHEN `mochiflow pr --spec <slug>` runs in tracked mode, THE SYSTEM SHALL preserve the existing preflight requirement for a committed accepted spec with a `Spec: <slug>` trailer.
- AC-07: THE SYSTEM SHALL NOT print guidance in local mode that suggests `git add -f` or otherwise force-tracking ignored `.mochiflow/` artifacts.
- AC-08: WHEN the open guidance prepares local-mode PR content, THE SYSTEM SHALL include final verification evidence, review result, and durable decision summary in the PR body material.
- AC-09: THE SYSTEM SHALL document tracked mode and local mode, including mode detection, constraints, recommended usage, and the migration path from local mode to tracked mode.
- AC-10: WHEN a local-mode spec has no committed `Spec:` trailer, THE SYSTEM SHALL still derive post-merge delivery state for provider-backed PRs and provider-none/manual handoffs where the spec branch tip is reachable from base, without weakening tracked-mode trailer derivation.
- AC-11: THE SYSTEM SHALL retain all generated/frozen engine contract checks after engine guidance changes, including freeze, vendored engine sync, and adapter output consistency.

## QA Scenarios

| QA | Dimension | Scope | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | In a fixture repository with `.mochiflow/` ignored, run `mochiflow accept <slug>` after preparing complete accepted evidence prerequisites except the final state mutation. | Accept exits `0`, updates local spec artifacts, does not stage or commit ignored `.mochiflow/` paths, and names local mode in output. |
| QA-02 | QA-FUNC, QA-COMPAT | cli | In a fixture repository with `.mochiflow/` tracked, run the existing accepted close-out flow. | Accept creates the close-out commit with the `Spec:` trailer and stages no unrelated paths. |
| QA-03 | QA-ABUSE | cli | In local mode, leave unrelated tracked source changes in the working tree before `accept` or `pr`. | The command fails preflight/readiness because tracked work is dirty; ignored local spec artifacts do not mask unrelated tracked changes. |
| QA-04 | QA-ABUSE, QA-DATA | cli | In local mode, prepare a spec with `UNVERIFIED`, `PENDING_HUMAN`, `FAIL`, or missing review result where review is required. | `accept` and local-mode `pr` reject the handoff before push/dispatch. |
| QA-05 | QA-COMPAT, QA-REG | cli | In local mode, run `mochiflow pr --spec <slug> --dry-run` with an accepted local spec and no committed spec trailer. | Dry-run does not fail due to missing committed spec/trailer and reports/uses the local-mode path as designed. |
| QA-06 | QA-COMPAT, QA-REG | cli | In tracked mode, run `mochiflow pr --spec <slug>` without an accepted spec commit or trailer. | Preflight still fails with the tracked-mode committed-spec requirement. |
| QA-07 | QA-DATA, QA-COMPAT | cli | In local mode with head equal to base, head not ahead of base, or detached/no source branch, run `mochiflow pr`. | Preflight rejects the handoff before push/dispatch with a specific branch/base reason. |
| QA-08 | QA-UX | docs | Inspect local-mode output and open/docs guidance. | No text tells users to run `git add -f .mochiflow/...`; local-mode skip reasons and constraints are explicit. |
| QA-09 | QA-COMPAT, QA-REG | cli | In provider-none local mode, push a spec branch, merge its tip into `origin/main` without a `Spec:` trailer, and inspect status/index/close eligibility. | The spec is derived as delivered/local-cleanup-pending while tracked-mode trailer behavior remains unchanged. |
| QA-10 | QA-REG | cli | Run the configured CLI verification, `mochiflow freeze --check`, adapter generation check, doctor, and spec lint. | All checks pass after code, docs, and generated engine artifacts are synchronized. |
| QA-11 | QA-RESIL | cli | N/A: the change is command preflight and local file persistence only; no long-running service, retry loop, or capacity path is introduced. | N/A: reliability is covered by deterministic preflight, branch-reachability checks, and regression tests. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Mandatory reviewer result for elevated risk is recorded in `design.md ## Review Results` before acceptance.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | Unit tests for shared mode detector; QA-01, QA-02 | `cli/crates/mochiflow-core/src/spec_mode.rs`; `cli/crates/mochiflow-core/src/lib.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml -p mochiflow-core spec_mode`; final verification command passed | Detector is shared by `accept` and `pr`. |
| AC-02 | cli | automated | CLI fixture test for ignored `.mochiflow/` accept; QA-01, QA-03, QA-04, QA-08 | `cli/crates/mochiflow-core/src/accept.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `behavioral_accept_local_mode_skips_staging_and_commit`; final verification command passed | Proves no staging/commit is attempted. |
| AC-03 | cli | automated | Existing and added tracked-mode accept regression; QA-02 | `cli/crates/mochiflow-core/src/accept.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `behavioral_accept_commits_flat_spec_with_safe_paths`; `behavioral_accept_commits_only_target_adr_records_and_reciprocals`; final verification command passed | Preserves close-out commit and `Spec:` trailer behavior. |
| AC-04 | cli | automated | Local-mode PR preflight fixture tests; QA-03, QA-04, QA-07 | `cli/crates/mochiflow-core/src/pr.rs`; `cli/crates/mochiflow-cli/tests/pr.rs` | PASS | `pr_local_mode_dispatches_without_committed_spec_trailer`; `pr_local_mode_requires_head_ahead_of_base`; final verification command passed | Covers clean tree, branch, base/head, ahead, and local accepted evidence. |
| AC-05 | cli | automated | Local-mode `pr --dry-run` fixture; QA-05 | `cli/crates/mochiflow-core/src/pr.rs`; `cli/crates/mochiflow-cli/tests/pr.rs` | PASS | `pr_local_mode_dry_run_does_not_require_committed_spec`; final verification command passed | Dry-run behavior is explicit in implementation and tests. |
| AC-06 | cli | automated | Tracked-mode PR preflight regression; QA-06 | `cli/crates/mochiflow-core/src/pr.rs`; `cli/crates/mochiflow-core/src/accept.rs`; `cli/crates/mochiflow-cli/tests/pr.rs` | PASS | `behavioral_pr_slug_guard_requires_committed_ship_closeout`; `pr_driver_writes_under_state_slug_not_specs`; final verification command passed | Existing tracked preflight remains required. |
| AC-07 | cli | automated | Output assertions for local-mode accept/pr and docs grep; QA-08 | `cli/crates/mochiflow-core/src/accept.rs`; `cli/crates/mochiflow-core/src/pr.rs`; `engine/commands/open.md`; `docs/` | PASS | `behavioral_accept_local_mode_skips_staging_and_commit`; `accept_guidance_uses_cli_and_persistence_modes`; final verification command passed | No `git add -f` local-mode workaround text. |
| AC-08 | cli | automated | Template/content assertions for PR body guidance; QA-08 | `engine/commands/open.md`; `engine/templates/delivery/pr-description.md`; `engine/reference/git.md` | PASS | `pr_body_template_requires_local_mode_evidence`; final verification command passed | PR body guidance requires verification evidence, review result, and durable decision summary. |
| AC-09 | cli | automated | Docs content checks plus manual read-through; QA-08 | `docs/configuration.md`; `docs/concepts.md`; `engine/reference/git.md`; `engine/reference/workflow.md` | PASS | `accept_guidance_uses_cli_and_persistence_modes`; `pr_body_template_requires_local_mode_evidence`; final verification command passed | Covers mode detection, constraints, usage, and migration. |
| AC-10 | cli | automated | Provider-none local-mode merge derivation fixture plus tracked-mode trailer regression; QA-09 | `cli/crates/mochiflow-core/src/delivery.rs`; `cli/crates/mochiflow-core/src/status.rs`; `cli/crates/mochiflow-core/src/index.rs`; `engine/router.md`; `engine/reference/git.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `delivery::tests::local_mode_branch_tip_in_base_is_done_without_trailer`; `delivery::tests::local_mode_branch_tip_signal_is_lost_after_branch_delete`; `router_merged_event_is_cleanup_only`; final verification command passed | Local mode has a non-trailer delivery signal after manual merge. |
| AC-11 | cli | automated | `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, `mochiflow doctor`, `mochiflow lint --spec local-only-spec-mode`; QA-10 | `engine/MANIFEST.json`; `.mochiflow/engine/`; generated adapter outputs | PASS | `cargo run --manifest-path cli/Cargo.toml -- freeze`; `upgrade --source engine`; `adapter generate --check`; default verification; `doctor`; `lint --spec local-only-spec-mode` all passed | Engine source guidance and generated artifacts are synchronized. |
