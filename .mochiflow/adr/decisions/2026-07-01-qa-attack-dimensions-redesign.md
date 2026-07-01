---
id: 2026-07-01-qa-attack-dimensions-redesign
date: 2026-07-01
area: [cli]
spec: qa-attack-dimensions-redesign
status: active
supersedes: 2026-06-25-qa-attack-matrix
---
## 2026-07-01 - QA attack coverage uses dimensions, not personas

**Decision:** QA attack coverage is expressed through a fixed catalog of
industry-standard dimensions rather than seven role-play personas. The QA
Scenarios table keeps `QA-XX` scenario IDs as the trace handle and adds
dimension labels such as `QA-FUNC`, `QA-UX`, `QA-ABUSE`, `QA-REG`,
`QA-COMPAT`, `QA-PERF`, and `QA-OBS`. AC Matrix evidence remains the acceptance
record; dimensions do not become separate ACs or a parallel attack-id scheme.

**Why:** The seven-persona model helped expose blind spots, but it mixed test
intent with fictional users. Dimensions map more directly to accepted QA and
software-quality practice: functional correctness, usability, abuse/security,
regression/behavior preservation, compatibility/integration, performance, and
observability. That makes the guidance easier to apply across product work,
engine changes, and reviewer audits without asking authors to role-play.

**Key sub-decisions:**
- `reference/risk.md` remains the single owner of risk-scaled QA attack
  expectations so plan authoring, reviewers, and templates do not fork the
  threshold logic.
- QA scenarios stay evidence-bearing through the existing AC Matrix. This keeps
  QA lightweight for standard specs and stronger for elevated or critical specs
  without adding another lifecycle artifact.
- Human-operated or visual QA remains explicitly confirmed during `open`; the
  matrix uses canonical result tokens such as `CONFIRMED`, `PASS`, and
  `N/A: <reason>`.

**Rejected:** keeping P1-P7 personas as canonical labels (memorable but less
standard and easy to confuse with user research); adding a new attack checklist
artifact (creates duplicate traceability); promoting each dimension to a formal
AC (bloats acceptance criteria and hides scenario-level evidence); adding CLI
lint enforcement immediately (premature until the dimension convention proves
stable in dogfood usage).
