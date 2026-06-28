---
id: 2026-06-23-commit-lifecycle-unification
date: 2026-06-23
area: [cli]
spec: commit-lifecycle-unification
status: active
---
## 2026-06-23 — commit-lifecycle-unification: one branch, commits from discuss onward

**Decision:** All four spec-lane phases commit to the same `{prefix}/{slug}`
branch. Branch creation moves from build to discuss (at agreement). discuss now
owns `status: draft` and writes `spec.yaml (draft)` + a new durable `pitch.md`
as its commit content. `pitch.md` (Problem, Appetite, Solution, Rabbit Holes,
No-gos, Alternatives Considered, Open Questions) persists through to
`_done/{slug}/` and is plan's durable input. discuss/plan use `docs(spec): ...`
Conventional Commits with a `Spec:` trailer.

**Why:** Commit timing was inconsistent — discuss wrote only an uncommitted
`_backlog` file, plan wrote spec files without committing, and the branch was
created at build start. Session loss between discuss and build destroyed all
artifacts, and first-commit ownership on the feature branch was unclear.
Committing from discuss onward makes every phase's contribution durable and
visible in history.

**Rejected:** The `_backlog` ready-for-plan handoff format
(`maturity: ready-for-plan`) — abolished because it duplicated what a committed
draft spec now carries; `_backlog` is seed-only (`maturity: seed`) raw input
again. `develop-branch-workflow` (a long-lived integration branch) — not
adopted; all commits still flow through a single feature branch + PR. Keeping
branch creation in build — rejected because it left discuss/plan output
uncommitted and orphaned from the branch.

**Consequence:** lint became dual-mode for `draft`: a pitch-only draft validates
with just `spec.yaml` + `pitch.md`, but once `spec.md` exists the normal
plan-time checks (including required `design.md`) apply. `backlog validate` now
rejects `maturity: ready-for-plan`. Router no longer resolves a ready-for-plan
handoff for `{slug} plan`; plan requires an existing discuss-created draft
folder. build verifies/switches the branch and error-stops if it is missing.
