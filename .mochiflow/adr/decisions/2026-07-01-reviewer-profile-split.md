---
id: 2026-07-01-reviewer-profile-split
date: 2026-07-01
area: [cli]
spec: qa-attack-dimensions-redesign
status: superseded
superseded_by: 2026-07-10-reviewer-core-composition
supersedes: 2026-07-01-grounded-independent-reviewer
---
## 2026-07-01 - Split independent reviewer into plan-auditor and change-reviewer

**Decision:** The canonical reviewer contract is split into two named profiles:
`plan-auditor` for pre-approval plan-quality review and `change-reviewer` for
post-implementation review. The old `independent-reviewer` label is retained
only as a legacy wrapper/alias. Both profiles keep the grounded adversary
contract: verify spec claims against repository reality, inspect whole-tree
impact targets, load relevant ADR records, and remain read-only auditors.

**Why:** A single `independent-reviewer` name hid two different jobs. Plan review
should focus on specification quality, risk coverage, task executability, and
whether QA dimensions are represented before implementation starts. Change
review should focus on the actual diff, behavioral regressions, test evidence,
and refactor safety after implementation. Separate names make those boundaries
obvious while preserving the grounding guarantees that caught real cross-file
issues in dogfood use.

**Key sub-decisions:**
- Legacy terms such as plan-quality mode and post-implementation mode map to the
  new profiles as aliases, not as the canonical public contract.
- `change-reviewer` treats refactors as first-class review material: it
  distinguishes mechanical changes from semantic changes and asks for
  behavior-preservation evidence when the diff can change behavior.
- Findings across severities include concrete remediation guidance so the
  orchestrating agent can act without re-deriving the full reviewer context.
- Kiro adapter outputs generate separate read-only agents for the two profiles;
  the retired generated `spec-independent-reviewer` target is not restored.

**Rejected:** keeping one reviewer with two modes as the primary name (continues
the ambiguous "independent" role); allowing reviewers to apply fixes or stage
changes (breaks the read-only audit boundary); making refactor review a separate
third reviewer (more ceremony than signal); removing legacy aliases immediately
(unnecessary compatibility break for existing docs and callers).
