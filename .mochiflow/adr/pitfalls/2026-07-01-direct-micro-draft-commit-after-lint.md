---
id: 2026-07-01-direct-micro-draft-commit-after-lint
date: 2026-07-01
area: [cli]
spec: retire-patch-for-micro-spec-depth
status: active
---
## Direct micro draft commit must happen after spec.md and draft lint (2026-07-01)

**Applies to:** `engine/commands/plan.md` direct micro intake and conformance
guards that pin direct micro branch/draft-commit ownership.

**Signal:** A direct micro procedure says to commit `spec.yaml` + `spec.md`
immediately after metadata confirmation and branch creation, before the steps
that create/refine `spec.md`, remove template residue, and run draft lint.

**Cause:** When moving branch durability from `discuss` to direct `plan`, it is
easy to attach "commit the draft artifacts" to the branch-creation sentence.
That makes the durability rule appear before the second artifact exists and
before the draft has passed the checks that make it safe to hand to the user.

**Guardrail:** For direct micro, first confirm metadata, write `spec.yaml`,
create/switch the branch, author/refine `spec.md`, run the consistency and lint
checks, then commit the draft micro artifacts before presenting
approve-to-build. Conformance should check the relative order, not only the
presence of "commit the draft" wording.

**Check:** `direct_micro_plan_is_pitchless_and_branch_durable` verifies the
draft commit instruction appears after the first draft lint and before the
approval card.

**Status:** Active.
