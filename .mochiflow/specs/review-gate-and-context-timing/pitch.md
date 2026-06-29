# Pre-approval review gate and PR-shipped context refresh

## Problem

Two parts of the workflow contract are internally inconsistent with the model
the engine already commits to elsewhere.

1. **Review runs after approval, not before.** `commands/plan.md` sets
   `status: approved` and creates the plan commit (steps 8-9), and only then
   offers `mochiflow-review` in the step-10 next-step card. But review is a
   spec/design **quality** review: a `fail` means the spec that was just locked
   to `approved` and committed was not actually sound. For `risk >= elevated`,
   where review is the *recommended* default, this ordering is backwards — the
   human approves before the recommended quality check has a chance to inform
   that decision. The placement was chosen (decision
   `2026-06-23-plan-to-build-transition-ux`) to make review *visible*, not to
   make the ordering correct.

2. **Context refresh is deferred to post-merge, which is base-branch noise.**
   When `commands/open.md` detects a coarse structural shift that makes the
   `[context]` layer stale, it records a *post-merge* `refresh-context`
   follow-up (`open.md` step 4; `reference/git.md ## Living-spec fold`;
   `commands/refresh-context.md`). But `commands/close.md` writes nothing to the
   base branch, so a post-merge refresh needs its own separate PR cycle — pure
   noise — and never rides on the spec branch. This directly contradicts the
   principle decision `2026-06-27-post-build-pr-close-flow` established for the
   living-spec fold: durable, judgment-bearing changes ship **inside the open
   PR**, reviewed and merged atomically, never written to base after merge. The
   context layer simply never migrated to that model. The stated reason for
   deferral ("refresh writes files and does not auto-commit, dirtying the tree
   before PR pre-flight") is a sequencing artifact: the pre-flight clean-tree
   check runs at `mochiflow pr`, which is *after* the accept close-out commit, so
   committing the regenerated context before that point keeps the tree clean.

## Appetite

Small. This is engine-documentation (workflow-contract) editing plus a re-freeze
and lint/conformance pass — no Rust feature code. Bounded to `plan.md`,
`open.md`, `refresh-context.md`, `git.md`, `workflow.md`, the two ADR fold
records, and the engine freeze manifest.

## Solution

**Change A — pre-approval review for `risk >= elevated`.** In `plan.md`, move the
review opportunity to *before* the approve-to-build gate for elevated/critical
specs: the readiness card offers Review (recommended) / Confirm the plan /
revise. Review runs `mochiflow-review` on the draft spec; on pass it re-presents
the approval action, on fail it reports findings and the spec stays `draft` until
the user re-confirms. Approval (`status: approved` + plan commit) happens only
after the user confirms. The post-approval step-10 card then offers only Start
implementation / Create a resume prompt. Review stays **optional** (the user may
confirm directly) and is explicitly *not* a delivery approval gate — the two
gates in `workflow.md` remain approve-to-build and approve-PR. `risk = standard`
keeps today's approve-then-optional-review flow unchanged.

**Change B — ship context refresh in the open PR.** When `open` detects a coarse
structural shift, run `refresh-context` on the feature branch (human confirms
current-state accuracy) and commit the regenerated `[context]` files as a
separate `docs(context): ...` commit on the feature branch **before**
`mochiflow pr`, so the update ships in the PR and base never takes an
out-of-band write. Post-merge refresh is demoted to the fallback for staleness
discovered only at or after merge (routed to a follow-up `fix` spec or backlog
seed, per the existing `git.md` rule). Update `open.md`, `refresh-context.md`,
and `git.md ## Living-spec fold` to describe the in-PR path as primary.

## Rabbit Holes

- Do not make review **mandatory** for elevated/critical — that conflicts with
  `reference/risk.md`'s "ad-hoc review is optional" stance. Keep it recommended
  and skippable.
- Do not fold the `[context]` layer into ADR or treat refresh as a fold — it
  stays a code-derived current-state map, only its *commit timing* moves.
- Do not expand `mochiflow accept`'s staging scope to include `[context]` files
  (option B from discussion). Keep the context commit a separate
  `docs(context)` commit so `accept`'s deterministic staging contract is
  untouched.
- Do not retrofit specs already authored under the old flow or archived under
  `_done/`.

## No-gos

- No change to the two-gate delivery model count (still exactly two gates).
- No Rust behavior change to `accept` / `pr` staging or pre-flight contracts.
- No new lint rule for review ordering (it is agent procedure, not a
  deterministic artifact check).

## Alternatives Considered

- **Keep review post-approval (status quo).** Rejected: leaves the elevated
  ordering illogical; an approved+committed spec can still be found unsound by
  the recommended review.
- **Make review a third delivery gate.** Rejected: over-formalizes an optional
  assist and conflicts with risk.md.
- **Fold context files into the accept close-out commit (option B).** Rejected
  for now: requires widening `mochiflow accept`'s staging contract; a separate
  `docs(context)` commit achieves the same in-PR shipping with no CLI change.
- **Keep post-merge refresh as primary.** Rejected: contradicts the
  ship-in-PR principle and forces a separate base-branch PR cycle (noise).

## Open Questions

- None — ready for plan.
