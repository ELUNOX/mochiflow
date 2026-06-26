# Suppress unchecked-task warnings for planned file deletions

## Background and Design Rationale

`mochiflow lint` currently warns when an approved spec has an unchecked task whose
`Files:` entries are dirty in the git worktree. That catches incomplete work, but
it is too blunt for tasks that intentionally delete files. Git reports deletions
as dirty paths, so lint can produce a warning even when the deletion is planned
and the task still has other unfinished work.

This was found while dogfooding MochiFlow, but the issue is project-agnostic:
template removals, configuration cleanup, deprecated API removal, and file
consolidation can all delete tracked files listed in `tasks.md`.

The chosen approach is an explicit `deleted: ` notation inside the existing
`Files:` block:

```md
- Files:
  - `src/kept.rs`
  - deleted: `src/old.rs`
```

Lint should suppress the unchecked-task dirty-file warning only when the task
entry uses that deletion marker and git reports the path as deleted. This keeps
ordinary modified-file warnings intact and avoids silently ignoring typos or
unexpected dirty states.

## User Story

As a developer using MochiFlow, I want lint to understand planned file deletions
in task file lists, so that useful unchecked-task warnings are not mixed with
unavoidable noise.

## Scope

- In: Parse `deleted: ` entries in `tasks.md` `Files:` blocks.
- In: Preserve git status kind for dirty paths used by approved-spec task
  warnings.
- In: Suppress the existing unchecked-task dirty-file warning only for
  deletion-marked paths that git reports as deleted.
- In: Add regression tests for planned deletions and normal dirty files.
- In: Document the notation in task authoring guidance/templates.
- Out: Automatic task checkbox updates.
- Out: Changing `done` eligibility or AC Matrix semantics.
- Out: Any lint side effect that stages, restores, or deletes files.

## Edge Cases

- A `deleted: ` entry whose path is modified, added, renamed, or untracked
  should not be treated as a planned deletion.
- Git porcelain status counts as deleted when either status column for that path
  is `D`, covering staged deletions, worktree deletions, and mixed states such
  as modified-then-deleted.
- A normal `Files:` entry that is deleted should keep warning until the task
  author marks it as `deleted: ` or checks the task.
- If a single `deleted: ` line contains multiple parseable paths, the deletion
  marker applies to every path parsed from that line.
- Checked tasks should keep suppressing this warning class.
- Paths should keep the existing normalization behavior for backticks, `./`
  prefixes, and path separators.
- Rename status should not be collapsed into a planned deletion unless git
  reports the old path as deleted.

## Acceptance Criteria (EARS)

- AC-01: WHEN an approved spec has an unchecked task with a `Files:` entry
  written as `deleted: `path`` and git porcelain status reports `D` in either
  status column for that path, THE SYSTEM SHALL NOT include that path in the
  "modified Files entries and is not checked" warning.
- AC-02: WHEN an approved spec has an unchecked task with a normal `Files:` entry
  and git status reports that path as dirty, THE SYSTEM SHALL continue to emit
  the existing unchecked-task warning for that path.
- AC-03: IF a `deleted: ` entry is dirty but git status does not report that path
  as deleted, THEN THE SYSTEM SHALL treat the path like a normal dirty `Files:`
  entry and warn while the task is unchecked.
- AC-04: THE SYSTEM SHALL support the `deleted: ` notation inside the existing
  multiline `Files:` block without requiring a new task block.
- AC-05: THE SYSTEM SHALL document the `deleted: ` notation in MochiFlow task
  authoring guidance and the generated task template.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | cli | Automated | Run lint on an approved spec with an unchecked task containing `deleted: ` for a path git reports as deleted. | Lint exits successfully and does not print the unchecked-task dirty-file warning for the deletion-marked path. |
| QA-02 | P3 | cli | Automated | Run lint on an approved spec with `deleted: ` for a path git reports as modified, added, or untracked. | Lint keeps the unchecked-task warning, proving the marker cannot hide non-deletion dirty states. |
| QA-03 | P6 | cli | Automated | Run the existing dirty normal-file and checked-task lint cases. | Normal dirty files still warn while unchecked, and checked tasks still do not warn. |
| QA-04 | P7 | cli | Automated | Compare parser behavior with task authoring guidance and task template examples. | The documented `deleted: ` notation matches the supported parser behavior. |
| QA-05 | P2 | cli | Automated | N/A check: this change has no interactive command flow, keyboard handling, or large user input path. | N/A: lint behavior is file/content driven. |
| QA-06 | P4 | cli | Automated | N/A check: inspect scope for persisted state or repository mutation beyond reading git status. | N/A: lint remains read-only and does not alter data. |
| QA-07 | P5 | cli | Automated | N/A check: inspect scope for migrations or legacy data conversion. | N/A: no data migration or stored format migration is introduced. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | Add/run conformance test for deletion-marked unchecked task with git deleted path; QA-01 | `cli/crates/mochiflow-core/src/lint.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED |  |  |
| AC-02 | cli | automated | Keep/run existing normal dirty-file warning test; QA-03 | `cli/crates/mochiflow-core/src/lint.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED |  |  |
| AC-03 | cli | automated | Add/run conformance test for `deleted: ` marker on a non-deleted dirty path; QA-02 | `cli/crates/mochiflow-core/src/lint.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED |  |  |
| AC-04 | cli | automated | Add/run parser coverage for multiline `Files:` block entries using `deleted: `; QA-01 | `cli/crates/mochiflow-core/src/lint.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED |  |  |
| AC-05 | cli | automated (doc/adapter check) | Update task authoring guidance/template and run engine/adaptor checks; QA-04 | `engine/reference/authoring.md`, `engine/templates/spec/tasks.md`, `.mochiflow/engine/**` | UNVERIFIED |  | Verify with `mochiflow freeze --check` / `mochiflow adapter generate --check` after dogfood sync. |
