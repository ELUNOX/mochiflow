---
slug: "prevent-build-phase-spec-mutation"
title: "Prevent tasks.md structural mutation during build phase"
surface: "cli"
type_hint: "fix"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-23"
updated: "2026-06-23"
---

# Prevent tasks.md structural mutation during build phase

## Signal

During build of commit-trailer-traceability, lint failed because AC-08 had no
covering task. The fix was to add T-008 to tasks.md mid-build. This is
effectively a plan-phase correction done inside build, risking spec artifact
mutation to satisfy tooling rather than reflecting genuine implementation work.

## Why It Matters

- Build-phase spec mutation undermines trust in the plan artifact as a stable contract.
- "Adding tasks to pass lint" is indistinguishable from document tampering.
- The plan→approved gate loses meaning if build can freely restructure tasks.

## Proposed Solution

1. Add stop condition to `engine/commands/build.md`: if tasks.md structure
   (task addition/deletion/AC reference change) needs modification, stop and
   route back to plan for re-approval.
2. Allow 1 task to cover multiple ACs via `[AC-07, AC-08]` compound reference
   in tasks.md, so plan can naturally group related verification without
   artificial task splitting.
3. Verify lint accepts compound AC references; if not, update lint logic.

## Evidence

- commit-trailer-traceability build: T-008 added mid-build to cover AC-08.
- Root cause: plan created AC-07 and AC-08 as separate ACs but only one task
  (T-007) whose Done condition already covered both.

## Open Questions

- Does lint currently parse `[AC-07, AC-08]` compound references?
- Should the stop condition allow minor task splits (e.g. adding a sub-step)
  or be absolute?
