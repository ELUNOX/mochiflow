# Present next-step choices after plan approval

## Background and Design Rationale

After plan sets `status: approved` and commits, the current step 10 presents
only two paths: a new-session handoff card or "continue with implementation".
There is no structured moment offering a spec/design review before build. This
is a natural decision point where surfacing options (especially review for
elevated/critical specs) helps users make informed choices.

## Problem / Goal

- Problem: Users are not presented with the option to review spec/design quality
  before committing to implementation. Elevated/critical specs proceed without
  the nudge.
- Goal: Replace plan.md step 10 with a 3-choice card that surfaces review as an
  option, ordered by risk.
- Non-goal: Making review mandatory. Changing router, risk.md, or review.md.

## Scope

- In: `engine/commands/plan.md` step 10 rewrite.
- Out: CLI code, router.md, risk.md, review.md, templates, new triggers.

## Requirements / Acceptance Criteria

| AC | Type | Priority | Requirement | Verification |
| --- | --- | --- | --- | --- |
| AC-01 | functional | Must | After plan commit, the agent SHALL present three choices: `review`, `build`, `later`. | manual review of plan.md |
| AC-02 | functional | Must | When `risk >= elevated`, `review` SHALL appear first with a recommended marker. When `risk = standard`, `build` SHALL appear first. | manual review of plan.md |
| AC-03 | functional | Must | Choosing `review` SHALL trigger the existing ad-hoc review flow (`mochiflow-review`). On pass/pass-with-comments, build/later are re-presented. On fail, findings are reported and the agent stops. | manual review of plan.md |
| AC-04 | functional | Must | Choosing `build` SHALL proceed to `mochiflow-build` in the same session (existing behavior). | manual review of plan.md |
| AC-05 | functional | Must | Choosing `later` SHALL output the handoff card from `templates/handoff/build-session-prompt.md` and stop (existing behavior). | manual review of plan.md |
| AC-06 | non-functional | Must | Choice keywords `review`/`build`/`later` SHALL be stable identifiers. Surrounding labels SHALL follow conversation language. | manual review of plan.md |

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | manual | review plan.md step 10 text | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-02 | cli | manual | review plan.md step 10 risk condition | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-03 | cli | manual | review plan.md step 10 review flow | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-04 | cli | manual | review plan.md step 10 build path | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-05 | cli | manual | review plan.md step 10 later path | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-06 | cli | manual | review plan.md step 10 language handling | `engine/commands/plan.md` | UNVERIFIED | | |
