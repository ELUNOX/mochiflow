---
slug: "ac-matrix-template-default-unverified"
title: "Pre-fill AC Matrix Result column with UNVERIFIED in spec template"
surface: "cli"
type_hint: "fix"
maturity: "seed"
source: "conversation"
source_phase: "build"
created: "2026-06-23"
updated: "2026-06-23"
---

# Pre-fill AC Matrix Result column with UNVERIFIED in spec template

## Signal

When creating the AC Matrix during plan, the agent repeatedly writes empty cells
or `TBD` in the Result column. `lint` rejects both — only valid tokens
(`UNVERIFIED`, `PASS`, `PENDING_HUMAN`, etc.) are accepted. This causes a
lint-fix cycle every time.

## Why It Matters

- Avoidable round-trip: plan writes matrix → lint fails → fix to UNVERIFIED.
- Memory-dependent: the correct token is documented in workflow.md but not in
  the template the agent copies from.

## Proposed Solution

Change `engine/templates/spec/spec.standard.md` (and `spec.md` if applicable)
AC Matrix example row to have `UNVERIFIED` as the default Result value. Agent
copies the template and gets a valid token without thinking.

Patch-eligible: single file + freeze, no design decision, reversible.

## Open Questions

- None.
