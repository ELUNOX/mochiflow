---
slug: "tasks-checkbox-enforcement"
title: "Enforce tasks.md checkbox updates during build"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Enforce tasks.md checkbox updates during build

## Signal

During the version-ssot-freeze build, all 8 tasks were completed and verified
but `tasks.md` checkboxes were never updated (`- [ ]` → `- [x]`). The drift
was only caught at ship when `mochiflow lint` rejected `status: done` with
unchecked tasks. The fix was a bulk `sed` — losing per-task traceability.

## Why It Matters

`tasks.md` is the spec's official progress record. If checkboxes are only
enforced at `status: done`, the file is stale throughout build and provides no
value as a progress indicator to humans or AI agents reading the spec mid-flight.

## Proposed Solution

1. **Engine `build.md` hard rule** — step 3d (commit): require the current
   task's checkbox to be `[x]` before staging the commit. Unchecked = do not
   commit.

2. **`mochiflow lint` WARN during `status: approved`** — when `tasks.md` exists
   and the spec is in build (`status: approved`), emit a WARN for any task that
   has implementation evidence (e.g. files in the task's `Files:` list are
   modified in the working tree or recent commits) but remains unchecked. This
   catches drift without blocking the entire build on an edge case.

## Decisions (tentative)

- Lint severity: WARN (not FAIL) during approved, FAIL only at done (current
  behavior preserved).
- Engine rule is tool-agnostic — works across all adapters.
- No Kiro-specific hook; the enforcement lives in engine docs + lint.
