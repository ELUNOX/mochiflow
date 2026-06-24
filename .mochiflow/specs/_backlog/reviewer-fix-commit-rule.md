---
slug: "reviewer-fix-commit-rule"
title: "Define commit rule for reviewer finding fixes"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "build"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Define commit rule for reviewer finding fixes

## Signal

When a reviewer returns fail during plan or build, the agent must decide whether
to amend the phase commit or create a separate fix commit. No rule exists in
plan.md or build.md, so the choice varies per session. build.md has implicit
guidance (risk.md says "fix, verify, commit as follow-up") but plan.md has none.

## Why It Matters

- Inconsistent git history: sometimes fixes are amended in, sometimes separate.
- Amending hides what was changed after review, making audit harder.
- Agent decision overhead on every review fix.

## Proposed Solution

Add a rule to plan.md (and confirm in build.md) stating:

> When fixing reviewer findings after a phase commit, create a separate
> `docs(spec): ...` commit with the same `Spec:` trailer. Do not amend the
> phase commit.

This matches build.md's existing pattern for post-reviewer follow-up commits
and keeps the "what was planned" vs "what was fixed after review" distinction
visible in history.

Patch-eligible: plan.md + optionally build.md/git.md wording addition, no logic.

## Open Questions

- None.
