---
slug: "spec-contract-lint-hardening"
title: "Harden lint checks for approved task scope and verification evidence"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "doctor-freeze-coherence"
source_phase: "build"
created: "2026-06-24"
updated: "2026-06-24"
---

# Harden lint checks for approved task scope and verification evidence

## Signal

During `doctor-freeze-coherence`, T-003 needed an additional test file that was
not listed in the approved task's `Files:` row. The correct response was to stop
build, return to plan, update task scope, and get user approval before
continuing.

The same plan review also found an AC labeled `automated` even though it would
have been documentation review unless a concrete doc-content assertion test was
added.

## Why It Matters

Approved `tasks.md` structure is now treated as an implementation contract, but
the guardrail is mostly procedural. Verification labels also need to match the
real evidence shape; otherwise ship evidence can overstate coverage.

## Evidence

- ADR `prevent-build-phase-spec-mutation` says approved `tasks.md` structure must
  not be materially changed during build without returning to plan.
- The T-003 correction required a plan commit before implementation continued.
- The final build made the automated label true by adding
  `docs_explain_doctor_freeze_boundaries_and_root_usage`.

## Decisions (tentative)

- Add lint or ready-time checks for task additions, deletions, renumbering, and
  meaningful `Files:` / `Done:` edits after approval.
- Add lint heuristics for `Type: Automated` rows whose verification/evidence
  does not name a runnable command, test, or assertion.
- Prefer warnings over hard failures for evidence-shape checks at first because
  evidence prose varies by surface.

## Open Questions

- Should `mochiflow lint` store or compare an approved task fingerprint?
- Is a lightweight "task scope changed since approval" check enough, or should
  lint understand task intent more deeply?
- What labels should be canonical for documentation-only ACs when no assertion
  test exists?

