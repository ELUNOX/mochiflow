---
id: 2026-06-23-plan-to-build-transition-ux
date: 2026-06-23
area: [cli]
spec: plan-to-build-transition-ux
status: active
---
## 2026-06-23 — plan-to-build-transition-ux: 3-choice card after plan approval

**Decision:** plan.md step 10 presents a 3-choice card (`review`/`build`/`later`)
after the plan commit, with display order driven by risk level. `review` triggers
the existing ad-hoc `mochiflow-review`; on pass, build/later are re-presented; on
fail, findings are reported and the agent stops.

**Why:** The previous 2-option handoff (build or new-session prompt) did not
surface spec/design review as an option. First-time users missed it entirely, and
elevated/critical specs could proceed without the nudge.

**Rejected:** Making review mandatory for elevated/critical (conflicts with
risk.md's "ad-hoc review is optional"); adding new router triggers for the
choice keywords (unnecessary — handled inline within plan's completion); always
fixed display order (loses the risk-driven nudge).

**Consequence:** plan.md step 10 and its stop conditions updated. No changes to
router.md, risk.md, review.md, or templates. No CLI code changes.
