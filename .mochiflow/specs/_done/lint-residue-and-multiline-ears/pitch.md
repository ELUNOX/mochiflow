# Lint: detect template residue and multi-line EARS ACs

## Problem

`mochiflow lint` does not yet enforce every authored quality check that the plan
procedure requires before implementation approval. Template residue such as
unfilled `{...}` placeholders, template-only HTML comments, example-only rows,
or bare `TBD` text can survive until human review. At the same time, the current
EARS warning checks only the line containing an AC ID, so a readable multi-line
AC can be warned as missing EARS keywords even when `THEN` / `SHALL` appears on
the continuation line.

This matters because the approval gate should catch unfinished spec prose
mechanically, while still allowing authors to format acceptance criteria
readably.

## Appetite

Small CLI hardening. The change is worth a focused implementation in the lint
module plus conformance tests, not a broader rewrite of spec parsing or markdown
analysis.

## Solution

Expand `mochiflow lint` in two ways:

- Treat each acceptance criterion as a block from its `AC-XX` declaration through
  the line before the next AC declaration or section heading. The EARS warning
  should pass when the block contains `SHALL`, `WHEN`, `WHILE`, `WHERE`, or
  `THEN`, even if the keyword is on a continuation line.
- Add template residue checks for expanded spec documents (`spec.md`,
  `design.md`, and `tasks.md` when they exist). Residue that indicates an
  unfilled template should fail before approval: unreplaced template
  placeholders, template-only HTML comments, example-only rows, and bare `TBD`.

The checks should avoid obvious false positives by ignoring fenced code blocks
and inline code spans for placeholder-like text. Build-owned AC Matrix result
tokens such as `UNVERIFIED` and `PENDING_HUMAN` are not residue.

## Rabbit Holes

- Do not build a complete Markdown parser unless the existing line-oriented
  approach cannot support the needed checks.
- Do not make every brace pair invalid; code snippets and command examples can
  legitimately contain `{...}`.
- Do not use residue checks to enforce writing style beyond unfinished template
  artifacts.

## No-gos

- No changes to the spec artifact schema.
- No new lifecycle state or approval gate.
- No change to the canonical AC Matrix result tokens.
- No broad lint rewrite outside the residue and EARS detection path.

## Alternatives Considered

- Leave residue detection as a plan checklist only. Rejected because the point of
  this change is to make the approval gate mechanically enforceable.
- Make residue warnings non-blocking. Rejected because warnings can still pass
  the approval path and do not prevent unfinished templates from reaching build.
- Require EARS keywords on the AC declaration line. Rejected because it forces
  less readable AC formatting and caused the observed false positive.
- Fail on every `{...}` occurrence. Rejected because it would false-positive on
  legitimate code and structured examples.

## Open Questions

- None - ready for plan.
