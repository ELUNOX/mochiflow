---
slug: "plan-approval-commit"
title: "Commit spec files at plan approval, not build start"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Commit spec files at plan approval, not build start

## Signal

Currently spec files (spec.md, design.md, tasks.md, spec.yaml) are authored
during plan but only committed when build starts (build.md step 2: "stage this
spec's own files with build's first commit"). This means approved specs exist
only as uncommitted local files between plan and build — invisible to other
collaborators and lost if the session dies.

## Why It Matters

- Approval is a meaningful lifecycle event that deserves its own git record.
- Other agents/humans cannot reference the spec until build starts.
- Session loss between plan and build loses the approved spec entirely.
- Git history conflates "spec approved" with "implementation started" in one
  commit.

## Proposed Solution

When `status: approved` is set at the end of plan:

1. Create the feature branch (`{prefix}/{slug}`) immediately.
2. Commit the spec files with a message like `plan({slug}): approve spec`.
3. Build step 2 no longer needs to handle spec staging — it just verifies the
   branch exists and spec is committed.

This is an engine docs change (`commands/plan.md` gains a commit step;
`commands/build.md` step 2 simplifies).

## Decisions (tentative)

- The plan-approval commit contains only `{specs_dir}/{slug}/**`.
- Branch creation moves from build to plan.
- build.md step 2 becomes: verify branch exists and worktree is clean (except
  spec files already committed).
