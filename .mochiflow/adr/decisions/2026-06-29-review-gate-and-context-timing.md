---
id: 2026-06-29-review-gate-and-context-timing
date: 2026-06-29
area: [cli]
spec: review-gate-and-context-timing
status: active
supersedes: 2026-06-23-plan-to-build-transition-ux
---
## 2026-06-29 — pre-approval review for risk>=elevated, and ship context refresh in the open PR

**Decision:** Two workflow-contract corrections.

- **Change A (review ordering).** For `risk >= elevated`, `plan.md` offers
  **Review before** the approve-to-build (confirm-plan) action instead of only
  after it. Review runs `agents/independent-reviewer.md` in a new **plan-quality
  mode** (Stage 1 spec/design/tasks conformance + spec-artifact quality, with no
  diff/changed-files input) on the still-`draft` spec; on `fail` it reports
  findings and leaves `status: draft` with no plan commit until the user
  re-confirms. Review stays **optional** (the user may confirm directly) and is
  explicitly a **quality assist, not a delivery approval gate** — the two gates
  remain approve-to-build and approve-PR, and review never sets `status` by
  itself. `risk = standard` keeps the prior approve-then-optional-review flow.
  This supersedes `2026-06-23-plan-to-build-transition-ux`, whose 3-choice card
  ran *after* approval: the placement made review visible but ordered it after
  the spec had already locked to `approved`, which is backwards for the risk
  level where review is recommended.
- **Change B (context refresh timing).** When `open` detects a coarse structural
  shift that stales the `[context]` layer, it runs `refresh-context` on the
  feature branch under human confirmation and commits the regenerated context as
  a separate `docs(context)` commit, ordered **after** the fold/context-check and
  **before** the `mochiflow accept` close-out commit, so the refresh **ships in
  the PR** and `mochiflow pr` pre-flight still sees a clean tree. Post-merge
  refresh is demoted to the fallback for staleness discovered only at/after
  merge. This aligns the context layer with `2026-06-27-post-build-pr-close-flow`
  (durable, judgment-bearing changes ship inside the open PR, never as a
  post-merge base write); the prior post-merge-primary instruction contradicted
  that principle and forced a separate base-branch PR cycle (noise).

**Why:** Both were internal inconsistencies with models the engine already
commits to. Review-after-approval can find an already-`approved`/committed spec
unsound; the recommended check should inform the human gate, not trail it.
Post-merge context refresh cannot land via `close` (which writes nothing to
base), so it needed its own PR cycle — exactly the post-merge base write
`2026-06-27` removed for the fold.

**Key sub-decisions:**
- **Reviewer gets a plan-quality mode, not a second agent.** A mode on
  `independent-reviewer.md` (widened `phases: [plan, build]`) avoids a second
  reviewer contract; `review.md` and `risk.md ## Review transport`/`## Ad-hoc
  review` note a code-less spec uses the no-diff mode.
- **`open` owns the context commit; `refresh-context` stays no-auto-commit.**
  Because `mochiflow accept` stages only `{specs_dir}/{slug}` + ADR, the context
  files must be committed by `open` in a separate `docs(context)` commit before
  accept; the accept close-out remains the single final state commit. `open`'s
  sequence is now `(a)-(g)`: acceptance → fold + context-check → optional
  `docs(context)` commit → accept close-out → PR body → approve-PR → `mochiflow
  pr`.
- **No new lint, but conformance tests pin both contracts.**
  `plan_offers_pre_approval_review_before_confirm_for_elevated` (Change A
  ordering) and the rewritten `open_ships_context_refresh_in_pr_before_accept`
  (Change B, replacing the old post-merge-primary test) guard against
  regression.

**Rejected:** making review mandatory for elevated/critical (conflicts with
`risk.md`'s "ad-hoc review is optional"); adding a third delivery gate
(over-formalizes an optional assist); a separate `agents/spec-reviewer.md` (a
mode is enough); widening `mochiflow accept` staging to include `[context]`
files (a separate `docs(context)` commit ships them in-PR without touching
accept's deterministic staging); keeping post-merge refresh as primary
(contradicts ship-in-PR and forces a base-branch PR cycle).
