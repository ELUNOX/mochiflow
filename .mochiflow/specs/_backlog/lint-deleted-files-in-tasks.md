---
slug: "lint-deleted-files-in-tasks"
title: "Lint fires on deleted files listed in tasks.md Modified Files"
surface: "cli"
type_hint: "bug"
maturity: "seed"
source: "conversation"
source_spec: "kiro-adapter-default-agent"
source_phase: "build"
created: "2026-06-24"
updated: "2026-06-24"
---

# Lint fires on deleted files listed in tasks.md Modified Files

## Signal

When a task's Modified Files column lists a file that was deleted (not just
modified), lint still warns "task has modified Files entries and is not checked".
The warning is confusing because there is no way to mark a deletion differently
from a modification, and the file no longer exists to inspect.

## Why It Matters

During a refactor spec that deletes many files (e.g. removing deprecated
templates), the lint output becomes noisy with warnings that cannot be resolved
by checking the task — the file is gone by design.

## Evidence

- `kiro-adapter-default-agent` T-001 listed
  `engine/adapters/kiro/agents/spec-builder.json.tpl` and 8 deleted `.tpl` files
  in Modified Files. Doctor emitted 7 WARN lines about unchecked tasks for files
  that were intentionally removed.

## Open Questions

- Should `~path` or `-path` prefix indicate a planned deletion, suppressing the
  lint warning once the file is confirmed absent?
- Or should lint skip the warning when the listed file does not exist on disk and
  the task is otherwise complete?
