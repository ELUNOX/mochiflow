---
slug: "lint-residue-and-multiline-ears"
title: "Lint: detect template residue and multi-line EARS ACs"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "discuss"
created: "2026-06-22"
updated: "2026-06-22"
---

# Lint: detect template residue and multi-line EARS ACs

## Signal

Two gaps in `mochiflow lint` surfaced while authoring a spec in plan:

1. The plan procedure relies on a human/AI eyeball pass to remove template
   residue (`{...}` placeholders, `TBD`, template-only HTML comments,
   example-only rows). `lint` does not mechanically enforce this, so residue can
   slip past to the approval gate.
2. EARS keyword detection appears to operate on the AC declaration line. An AC
   written across multiple lines (`IF ... / THEN ... SHALL ...` wrapped onto
   continuation lines) was flagged as "AC without EARS keyword" even though the
   keywords were present on the following lines. Reformatting to put the keyword
   on the AC head line cleared the warning.

## Why It Matters

The plan gate's quality depends on these checks. Mechanizing residue detection
removes a class of "forgot to fill a placeholder" defects before approval.
Fixing the multi-line EARS false-positive prevents authors from contorting AC
wording just to satisfy the linter, which otherwise pushes against readable
multi-line acceptance criteria.

## Evidence

- `commands/plan.md` step 6 lists residue removal as an authored checklist item
  ("no `{...}` placeholder remains", "no `TBD` remains except ...", "no
  template-only HTML comment remains") — enforced by procedure, not by `lint`.
- 2026-06-22 session: AC-04 written as `IF ... THEN ... SHALL ...` across lines
  produced `WARN: AC without EARS keyword (SHALL/WHEN/WHILE/WHERE/THEN): AC-04`;
  rewriting it as a single-line-head `WHEN ... THE SYSTEM SHALL ...` cleared it.
- `authoring.md` documents EARS keywords but does not state that the keyword must
  sit on the AC head line.

## Open Questions

- Should residue detection be a FAIL or a WARN at `draft`, and which patterns are
  safe to hard-fail (placeholders) vs warn (e.g. `TBD` in matrix cells owned by
  build)?
- Should EARS detection scan the whole AC block (head + continuation lines) until
  the next AC ID, instead of just the head line?
- Are there legitimate `{...}` uses in authored specs (e.g. code-like content)
  that residue detection must not false-positive on?
