---
slug: "commit-lifecycle-unification"
title: "Unify commit timing across discuss/plan/build/ship on a single branch"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Unify commit timing across discuss/plan/build/ship on a single branch

## Signal

Commit timing is currently ambiguous across phases. discuss produces no file,
plan generates files but build commits them, and branch creation happens at
build start. This creates volatility (session loss loses discuss/plan output)
and unclear ownership of the first commit.

## Why It Matters

- Session loss between discuss and build loses all artifacts.
- No single source of truth for "when does the branch exist?"
- Spec files appear in the first build commit, conflating "spec approved" with
  "implementation started" in git history.
- pitch.md (discuss output) has no persistent home until this is resolved.

## Proposed Solution

All phases commit to the same feature branch `{prefix}/{slug}`. No commits
to main/develop — everything goes through a single PR at ship.

### Commit sequence

| Phase | Commit | Content |
|-------|--------|---------|
| discuss | `discuss({slug}): record pitch` | `spec.yaml` (status: draft) + `pitch.md` |
| plan | `plan({slug}): approve spec` | `spec.md` + `design.md` + `tasks.md` + `spec.yaml` (status: approved) |
| build | `{type}(scope): summary` (1–N per risk) | Implementation code + `tasks.md` checkbox updates |
| ship | `chore: complete {summary}` | `spec.yaml` (done) + AC Matrix + ADR fold + `_done/` move + INDEX |

### Branch lifecycle

```
{prefix}/{slug} created ← discuss agreement
  │
  ├─ discuss commit
  ├─ plan commit
  ├─ build commit(s)
  └─ ship close-out commit
       └─ PR → merge → main
```

### Key rules

- Branch creation: at discuss agreement (first commit).
- main/develop: never touched directly. All work goes through PR.
- pitch.md: persists through the entire lifecycle, archived in `_done/{slug}/`.
- Draft PR: not used (PR is created at ship only).
- Trailer: `Spec: {slug}` on all commits; `Task: T-XXX` on build commits.

## Supersedes

This seed consolidates and supersedes:
- `discuss-pitch-persistence` (discuss output → pitch.md in spec dir)
- `plan-approval-commit` (commit spec at plan approval)

Those seeds should be merged into this one if planned together.

## Decisions (tentative)

- Engine docs changes: `commands/discuss.md` (branch + commit), `commands/plan.md`
  (commit only, no branch creation), `commands/build.md` (remove branch creation
  from step 2), `reference/git.md` (commit lifecycle table).
- `mochiflow lint` must accept `status: draft` with only spec.yaml + pitch.md.
- pitch.md uses a template (structured, not freeform).
- No CLI code change — branching and committing are git operations guided by
  engine docs.
