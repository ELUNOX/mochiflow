# Suppress unchecked-task warnings for planned file deletions

## Problem

`mochiflow lint` warns when an approved spec has an unchecked task whose
`Files:` entries are dirty in the worktree. That guardrail is useful for normal
edits, but it becomes noisy when the task intentionally deletes files: deleted
paths are dirty, yet they are not modified files that can be inspected or
completed in the same way as changed source files.

This surfaced while dogfooding MochiFlow, but it is not MochiFlow-specific. Any
project can hit it during template removal, configuration cleanup, file
consolidation, deprecated API removal, or refactors that delete tracked files.
If lint reports unavoidable warnings, users learn to discount the warning class.

## Appetite

Small CLI fix. This should stay within lint parsing and tests, with no workflow
state change, no new dependency, and no broad task model redesign.

## Solution

Add an explicit planned-deletion notation for `tasks.md` `Files:` entries and
teach lint to respect it.

The expected shape is:

```md
- Files:
  - `src/kept.rs`
  - deleted: `src/old.rs`
```

When an unchecked task lists a file with the deletion marker and git reports
that path as deleted, lint should not emit the existing "modified Files entries
and is not checked" warning for that path. Normal file entries should keep the
current behavior. A missing file without the deletion marker should not be
silently ignored.

Plan should choose the final marker spelling, but it should be explicit and
human-readable rather than inferred from a missing path.

## Rabbit Holes

- Do not redesign task completion semantics or make lint infer task progress.
- Do not suppress all warnings for absent files; that would hide typos and
  unexpected deletes.
- Do not introduce a separate `Deleted Files:` task block unless plan finds
  that extending `Files:` is too ambiguous.

## No-gos

- No change to spec lifecycle state.
- No change to `done` eligibility.
- No automatic task checkbox updates.
- No git staging, restore, or file deletion side effects from lint.

## Alternatives Considered

- Skip warnings whenever a listed file does not exist on disk. Rejected because
  it would hide misspelled paths and accidental deletes.
- Treat a leading `-path` or `~path` as deletion syntax. Rejected for now because
  it is terse and easier to confuse with Markdown list syntax or shell-ish
  conventions.
- Require users to check the task immediately after deletion. Rejected because
  a task can delete one file early while other work in the same task remains
  incomplete.

## Open Questions

- None - ready for plan.
