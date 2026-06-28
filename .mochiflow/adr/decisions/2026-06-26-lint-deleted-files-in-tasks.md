---
id: 2026-06-26-lint-deleted-files-in-tasks
date: 2026-06-26
area: [cli]
spec: lint-deleted-files-in-tasks
status: active
---
## 2026-06-26 — lint-deleted-files-in-tasks: explicit planned-deletion task entries

**Decision:** Planned file deletions in `tasks.md` use an explicit `Files:`
entry marker, written as ``deleted: `path` ``, rather than inferring intent from
missing files or introducing a separate task block. Lint suppresses the approved
spec unchecked-task dirty-file warning only when the task entry is marked as a
planned deletion and git porcelain status reports `D` in either status column for
that path. A `deleted:` line with multiple parsed paths marks all paths from
that line, though one deleted path per line remains the recommended style.

**Why:** Deletions are legitimate task work, but git reports them as dirty paths,
which made the existing warning noisy during refactors. Inferring deletion
intent from absence would hide typos and accidental deletes. A compact marker in
the existing `Files:` block keeps task structure stable while giving lint enough
contractual signal to distinguish planned deletion from ordinary dirty work.

**Rejected:** Skipping warnings for every absent file (too easy to hide
misspelled paths and unexpected deletes); terse `-path` / `~path` prefixes
(ambiguous with Markdown and shell conventions); a separate `Deleted Files:`
block (larger task-shape change for the same behavior); auto-checking tasks
after deletion (task completion can include more work than the deletion).

**Consequence:** `lint` now preserves deletion status from git porcelain output,
task file parsing carries a planned-deletion flag, conformance tests cover
planned deleted and non-deleted dirty cases, and task authoring guidance/templates
document the marker. The change is elevated because it adds a user-authored task
notation.
