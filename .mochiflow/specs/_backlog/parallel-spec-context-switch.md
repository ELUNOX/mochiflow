---
slug: "parallel-spec-context-switch"
title: "Support parallel spec work with explicit context switching"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Support parallel spec work with explicit context switching

## Signal

Current flow assumes 1 session = 1 spec, sequentially. In practice, developers
want to start spec-B while spec-A is in PR review, or handle PR feedback on
spec-A mid-build of spec-B. The router has no explicit spec binding, and there
is no defined procedure for interrupting one spec to work on another.

## Why It Matters

- Blocking on PR review wastes time that could be spent on the next spec.
- PR feedback arrives asynchronously; there's no way to handle it without
  abandoning the current session or manually managing git state.
- Without explicit context switching, agents may accidentally mix spec contexts.

## Current State (mostly works already)

- Branch isolation: each spec gets `{prefix}/{slug}` — no conflict.
- Dirty check: scoped to `{specs_dir}/{slug}/**` — other specs don't interfere.
- Lint: `--spec {slug}` already exists.
- Index: only runs at ship close-out — parallel specs don't collide.
- Session separation: new session + `{slug} build` resumes from git state.

## What's Missing

1. **Router spec binding** — router should explicitly track which slug the
   current session is working on, and reject/confirm cross-spec operations.

2. **Context switch UX** — when the user requests work on a different spec:
   - Announce current spec state (phase, last completed task).
   - Confirm switch intent.
   - `git switch` to the target branch.
   - Resume the target spec's workflow from its current state.

3. **Interrupt/resume guarantee** — document that interrupting mid-build is
   safe as long as the last commit is pushed. Resume is `{slug} build` in a new
   session (reads tasks.md checkboxes + git state to determine where to continue).

## Proposed Solution

### Router spec binding

Add to router.md: "The active spec is the slug resolved at session start or
by an explicit `{slug} <verb>` command. Cross-slug operations require explicit
confirmation."

### Context switch procedure (engine docs)

```
User: "spec-a の feedback 対応して"

Agent (currently on spec-b):
  現在 spec-b (build, T-003 完了) に取り組んでいます。
  spec-b を中断して spec-a に切り替えますか？
  - 進捗は保存済み（最新コミットまで）
  - 再開: 「spec-b build」で続きから
  [switch / continue]
```

On switch: `git switch feat/spec-a`, then resume spec-a's workflow.

### Resume detection

When build is invoked on a spec with checked tasks in tasks.md, skip completed
tasks and resume from the first unchecked task. This requires tasks-checkbox-
enforcement (separate backlog seed) to be reliable.

## Decisions (tentative)

- This is primarily an engine docs change (router.md + reference/workflow.md).
- No CLI code change needed — git branching and spec files already support this.
- tasks.md checkboxes become the resume cursor (depends on tasks-checkbox-enforcement).
- No explicit "pause" command — interruption is implicit (session ends or user switches).
