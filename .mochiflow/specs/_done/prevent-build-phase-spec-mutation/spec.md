# Prevent build-phase spec mutation

## Background and Design Rationale

During the `commit-trailer-traceability` build, lint failed at close-out because
one acceptance criterion had no covering task. The build-time fix added a new
task to `tasks.md`, which effectively changed the approved implementation plan
after the approve-to-build gate. This spec closes that workflow gap without
adding structural drift detection.

The chosen boundary is procedural and template-backed: once a plan is approved,
build may record progress and evidence, but it may not reshape task structure.
If the work discovers that tasks need to be added, removed, split, renumbered,
re-associated with AC/NFR/chore references, or semantically changed, build must
stop and route back to plan for re-approval.

The plan also formalizes multi-AC task references. Existing lint code extracts
AC IDs with `AC_RE` from task-line bracket references and `Covers AC:` lines, so
compound references such as `[AC-07, AC-08]` should work. The missing pieces are
explicit authoring guidance, template examples, and tests that lock the behavior
in place.

This work intentionally does not add snapshot/hash-based drift detection. That
may be useful later, but it is heavier than the current problem requires.

This branch includes existing local commit `3621592 docs: add phase completion
backlog seed`; implementation and delivery should preserve that commit in the
branch history.

## User Story

As a MochiFlow maintainer, I want build to treat approved task structure as a
contract, so that implementation cannot silently rewrite the plan it was
approved to execute.

## Scope

- In:
  - Add a build stop condition for structural `tasks.md` changes after approval.
  - Define which `tasks.md` changes are allowed during build.
  - Document compound AC task references in authoring guidance and task templates.
  - Add lint/conformance tests proving compound task references cover every AC
    they mention.
  - Run required engine freeze, dogfood upgrade, adapter check, and CLI
    verification.
- Out:
  - Structural drift detection via hashes, snapshots, or a new lint mode.
  - A one-task-per-AC rule.
  - Broad redesign of phase completion prompts.
  - Direct push or provider PR command changes.

## Edge Cases

- A single implementation task covers multiple related ACs and should be written
  as one task with a compound reference.
- Build discovers that an approved task's `Files`, `Done`, or `Stop` block is
  materially wrong; build must stop instead of editing the task body to fit the
  implementation.
- Build needs to mark completed tasks or fill AC Matrix results and evidence;
  those progress/evidence edits remain allowed.
- `Covers AC:` legacy/task-body references remain supported when lint extracts
  AC IDs for coverage.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL instruct `mochiflow-build` to stop and route back to
  plan when implementation requires structural `tasks.md` changes after
  approval.
- AC-02: THE SYSTEM SHALL define build-allowed `tasks.md` edits as task checkbox
  updates and AC Matrix result/evidence updates, without adding structural drift
  detection.
- AC-03: THE SYSTEM SHALL document that one task may reference multiple ACs in a
  single compound bracket reference.
- AC-04: WHEN lint validates task coverage and unknown AC references, THE SYSTEM
  SHALL treat every AC ID inside a compound task reference as an individual
  reference.
- AC-05: THE SYSTEM SHALL keep source engine, vendored engine, generated adapter
  checks, and engine manifest state consistent after the engine-source edits.

## QA Scenarios

| QA | Scope | Steps | Expected result |
| --- | --- | --- | --- |
| QA-01 | cli | Run `cargo test --manifest-path cli/Cargo.toml lint_accepts_compound_task_ac_references` or the final default verification command. | Compound `[AC-01, AC-02]` task references are accepted and cover both ACs. |
| QA-02 | cli | Run `mochiflow lint --spec prevent-build-phase-spec-mutation`. | The spec remains internally consistent. |
| QA-03 | cli | Run `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check` after engine edits. | Engine manifest, vendored engine, and generated adapters are in sync. |
| QA-04 | cli | Run the configured default verification command for `cli`. | CLI tests, formatting, clippy, and freeze check pass. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Required elevated-risk review result is recorded before ship.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `mochiflow lint --spec prevent-build-phase-spec-mutation`; source review of `engine/commands/build.md` | `engine/commands/build.md` | PASS | `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` passed |  |
| AC-02 | cli | automated | `mochiflow lint --spec prevent-build-phase-spec-mutation`; source review of `engine/commands/build.md` | `engine/commands/build.md` | PASS | default verification passed | No drift detection in scope. |
| AC-03 | cli | automated | `mochiflow lint --spec prevent-build-phase-spec-mutation`; source review of authoring/template docs | `engine/reference/authoring.md`, `engine/templates/spec/tasks.md` | PASS | default verification passed |  |
| AC-04 | cli | automated | Targeted cargo test plus final default verification | `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml lint_accepts_compound_task_ac_references`; `cargo test --manifest-path cli/Cargo.toml lint_rejects_unknown_ac_in_compound_task_reference`; default verification passed | `lint.rs` did not require changes. |
| AC-05 | cli | automated | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check`; default verification | `engine/MANIFEST.json`, `.mochiflow/engine/**` | PASS | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check`; default verification passed | Vendored engine matched source after sync; no tracked vendored diff remained. |
