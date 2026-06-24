---
slug: "unify-commit-granularity-per-task"
title: "Unify commit granularity to per-task across all risk levels"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "build"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Unify commit granularity to per-task across all risk levels

## Signal

Current risk.md ties commit granularity to risk level (standard=1 commit,
elevated=per logical step, critical=per task). This conflates two independent
concerns: how often to commit (a traceability decision) and how often to review
(a quality decision). For standard risk, squashing all tasks into 1 commit makes
git history unreadable, revert/bisect impossible per-task, and session resumption
harder.

## Why It Matters

- standard-risk specs with multiple tasks produce a single opaque commit.
- `git log` cannot show what changed per task; `Task:` trailers are meaningless
  in a single commit containing all tasks.
- Revert is all-or-nothing; bisect cannot isolate a single task's regression.
- Session resumption must diff files instead of reading commit history.
- Agents already auto-commit — increasing granularity costs nothing.

## Proposed Solution

Unify commit granularity to **per task** for all risk levels. Risk determines
only reviewer cadence (when to invoke independent-reviewer), not commit timing.

risk.md Consequences table becomes:

| risk | reviewer cadence | commit granularity |
| --- | --- | --- |
| standard | none (AC Matrix only) | per task |
| elevated | once after all tasks | per task |
| critical | after each task | per task |

The "commit granularity" column could be removed entirely and replaced with a
single statement: "Commit once per completed task, regardless of risk."

Exception: micro specs without tasks.md naturally produce 1 commit (the change
is one logical unit).

## Open Questions

- Should the commit-granularity column be removed from the table, or kept for
  documentation clarity even though all values are identical?
- Does build.md step 3d need rewording beyond "when the commit unit is reached"?
