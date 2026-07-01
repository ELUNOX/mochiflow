---
id: 2026-06-25-qa-attack-matrix
date: 2026-06-25
area: [cli]
spec: qa-attack-matrix
status: superseded
superseded_by: 2026-07-01-qa-attack-dimensions-redesign
---
## 2026-06-25 — qa-attack-matrix: persona QA attack coverage via QA Scenarios + risk.md owner

**Decision:** Adversarial "do not trust that it works" testing is captured as a
seven-persona (P1-P7) dimension inside the existing `spec.md ## QA Scenarios`
table, not a separate section or a metadata-generated checklist.
`reference/risk.md ## QA attack coverage` is the single owner of the
risk->persona/evidence mapping (standard requires P1/P3/P6/P7 with reasoned
`N/A`; elevated requires evidence for relevant personas; critical requires strong
evidence). `plan.md` authoring and the independent reviewer's existing Stage 1
reference that mapping instead of restating thresholds. Attacks stay traceable
through `QA-XX` ids referenced from the AC Matrix; they are never promoted to
formal ACs and gain no parallel attack-id scheme.

**Why:** Adversarial thinking previously happened only at code review, surfacing
defects late. `spec.md` exists for every spec (micro and up), so anchoring the
dimension there avoids leaving standard specs without attack coverage. Reusing
QA Scenarios + AC Matrix keeps attacks evidence-bearing through existing
machinery rather than a parallel artifact.

**Rejected:** A metadata-generated attack checklist (becomes paperwork unless the
reviewer verifies evidence, and is hard to make evidence-bearing); a
`design.md`-only attack section (absent for standard specs, exactly where naive
defects concentrate); a separate "QA Attack Review" reviewer stage or splitting
Stage 1 (large blast radius on the reviewer output contract and adapters,
overlapping responsibility); requiring all seven personas for every spec
(formalizes trivial work, dilutes weight on high-risk changes); promoting attacks
to ACs or an `ATK-XX` scheme (AC bloat / duplicate trace path); adding CLI lint
enforcement now (would pull in `contracts.lock` / `engine/VERSION` churn before
the convention has settled — deferred to a possible follow-up, reviewer Stage 1
is the enforcement for now).

**Consequence:** Docs/templates-only engine change: `risk.md` gains
`## QA attack coverage`; `plan.md` step 2, `independent-reviewer.md` Stage 1
(+ `reference/risk.md` in its References), and the `spec.standard.md` /
`spec.md` QA Scenarios tables (new `Persona` column) are updated. No Rust/CLI
lint rule, no new reviewer stage, no AC Matrix schema change. Risk classified
`standard` (reversible, single surface, integration none), so no `design.md`.
