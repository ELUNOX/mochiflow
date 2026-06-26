# Suppress unchecked-task warnings for planned file deletions - Design

## Design Decisions

- Treat `deleted: ` as a documented task-file authoring contract. This raises
  the change to elevated risk because users and agents will rely on the syntax
  when writing `tasks.md`.
- Keep the marker inside the existing `Files:` block instead of introducing a
  new `Deleted Files:` block. This preserves the current task shape and keeps
  lint changes localized to file-entry parsing and warning selection.
- Require both the marker and git deletion status before suppressing the
  unchecked-task dirty-file warning. The marker alone is not enough, and a
  missing or deleted path without the marker still warns.
- Preserve existing path normalization for backticks, `./` prefixes, and
  backslash replacement so the new notation behaves like normal `Files:`
  entries.

## Architecture

- Extend the internal task file-entry representation so each parsed path carries
  whether it was marked as a planned deletion.
- Replace the dirty-path set used by approved-spec task warnings with a
  repository status map that retains enough git porcelain status to distinguish
  deleted paths from other dirty states.
- Keep the change read-only: lint still reads `tasks.md` and git status, then
  emits issues. It does not mutate files, tasks, git state, or spec metadata.

## Data Model / Interfaces

- Public authoring interface:

```md
- Files:
  - `src/changed.rs`
  - deleted: `src/old.rs`
```

- Internal model:
  - normal file entry: `{ path, planned_deletion: false }`
  - deletion file entry: `{ path, planned_deletion: true }`
  - dirty worktree status: `{ path, deleted: bool }`

The exact Rust type names are implementation details. The behavior contract is
that warning suppression requires `planned_deletion == true` and
`deleted == true`.

For git porcelain v1 status, `deleted` is true when either of the two status
columns for the path is `D`. This treats staged deletions, worktree deletions,
and mixed states such as `MD` or `AD` as deleted for this warning decision.
Untracked, added, modified, copied, and renamed states are not deleted unless
one of their status columns is `D`.

If one `deleted: ` line contains multiple parseable paths, every path parsed
from that line receives `planned_deletion: true`. The task template should still
show the clearer one-line-per-deleted-path style.

## Error Handling

- If git status cannot be read, keep the existing behavior of skipping
  dirty-worktree task warnings rather than failing lint.
- If a `deleted: ` entry cannot produce a path after existing path
  normalization, ignore it the same way empty normal `Files:` entries are
  ignored.
- If a `deleted: ` path is dirty but not deleted, emit the existing warning
  instead of adding a new warning class.

## Test Strategy

- Add conformance coverage that creates a tracked file, deletes it, and verifies
  that an unchecked task using `deleted: ` does not warn.
- Keep existing conformance coverage for normal dirty file entries and checked
  tasks.
- Add coverage where `deleted: ` is used on a non-deleted dirty path and verify
  that lint still warns.
- Run the configured CLI verification profile before build completion.

## Review Results

- 2026-06-26 mandatory build review
  - Reviewer mode: delegated
  - Verdict: pass-with-comments
  - Summary: Implementation matched the lint contract. Reviewer noted unrelated
    backlog/index noise from a prior branch-level cherry-pick; the build cleanup
    removed the unrelated backlog seed from this branch and regenerated
    `.mochiflow/INDEX.md`.
