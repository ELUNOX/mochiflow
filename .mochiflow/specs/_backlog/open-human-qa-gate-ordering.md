---
slug: "open-human-qa-gate-ordering"
title: "Resolve the open human-QA vs accept-gate ordering for gate-behavior scenarios"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "open"
created: "2026-06-27"
updated: "2026-06-27"
---

# Resolve the open human-QA vs accept-gate ordering for gate-behavior scenarios

## Signal

While driving `open` for `post-build-pr-close-flow`, AC-13 was `PENDING_HUMAN`
and backed by QA-08 ("drive open to the approve-PR gate, decline, then approve").
But `accept` (step c) requires every AC Matrix row to be done-eligible, and
`lint` rejects `PENDING_HUMAN`. So AC-13 must become `CONFIRMED` in step (a),
even though the behavior QA-08 verifies only actually occurs at the approve-PR
gate in step (e) — after `accept`. The order creates a small chicken-and-egg:
the evidence's natural moment is after the step that demands the evidence.

## Why It Matters

`open.md` does not state how to handle human QA items that verify the open
procedure's own gate behavior (as opposed to feature behavior). A driver has to
improvise (confirm the gate-behavior item up front), which is easy to get wrong
or to mistake for skipping the gate.

## Evidence

- `engine/commands/open.md` step (a) 3 (QA round-trip) precedes step (c)
  `accept`; `accept`/`lint` require done-eligible rows (`lint.rs`
  `is_done_matrix_result` excludes `PENDING_HUMAN`).
- spec.md QA-08 is `Human-operated` and describes the approve-PR gate round-trip
  exercised in step (e).
- In this session AC-13 was set to `CONFIRMED` in step (a) as a pre-confirmation
  of the gate behavior, then the live gate still ran in (e).

## Decisions (tentative)

- Add one line to `open.md`: gate-behavior human QA items may be pre-confirmed
  in step (a) (CONFIRMED) because the live approve-PR gate in step (e) still
  enforces the behavior; the pre-confirmation is not a substitute for the gate.
- Or introduce a distinct token / handling for "verified by the open gate
  itself" so the matrix is honest about provenance.

## Open Questions

- Should gate-behavior scenarios be modeled as AC Matrix rows at all, or as a
  procedure-contract assertion separate from feature QA?
- Is a doc clarification enough, or does `accept`/`lint` need a notion of
  "settled-at-gate" rows?
