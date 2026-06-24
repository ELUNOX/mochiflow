# Normalize AC Matrix result tokens to ASCII-only

## Problem

AC Matrix result tokens are asymmetric: `PASS`, `FAIL`, `PENDING_HUMAN`, and
`UNVERIFIED` are English ASCII, but `人間確認済み` and `対象外（<reason>）` are
Japanese. This forces all projects — regardless of `artifact_language` — to
write Japanese in the Matrix. English-speaking users encounter unexpected
non-ASCII tokens in an otherwise English workflow, violating the principle that
machine-readable stable identifiers should be locale-independent.

## Appetite

Small-medium. Rust lint logic change (2 functions + 2 error messages),
engine docs updates (workflow.md, ship.md, language.md, 2 templates), and
conformance test updates. No migration of existing `_done/` specs.

## Solution

Introduce ASCII canonical tokens:
- `CONFIRMED` replaces `人間確認済み` as the preferred done-eligible human QA token.
- `N/A: <reason>` is promoted from "ASCII input equivalent" to the canonical
  not-applicable token (replacing `対象外（<reason>）` as the preferred form).

The old Japanese tokens (`人間確認済み`, `対象外（<reason>）`) become permanent
deprecated aliases — lint continues to accept them so archived `_done/` specs
remain valid. New specs use ASCII only (templates and docs show only the new
tokens).

Lint changes:
- `is_canonical_matrix_result`: accept both old and new tokens.
- `is_done_matrix_result`: accept both old and new tokens.
- Error messages: show new tokens as primary, note old tokens as "also accepted".

Doc/template changes:
- `workflow.md`: update canonical token definitions, done-eligible list.
- `ship.md`: round-trip protocol maps to `CONFIRMED` / `N/A: <reason>`.
- `language.md`: Stable Identifiers list uses new tokens, notes deprecated aliases.
- `spec.md` / `spec.standard.md` templates: Completion Conditions use new tokens.

## Rabbit Holes

- Do not migrate existing `_done/` specs. The deprecated alias keeps them valid
  with zero churn.
- Do not introduce a third form for not-applicable (`N/A(<reason>)` with
  parentheses). `N/A: <reason>` (colon-space) is already accepted by lint and
  widely used; promote it as-is.
- Do not rename `PENDING_HUMAN` — it is already ASCII and unambiguous.

## No-gos

- No removal of deprecated alias acceptance (would break archived specs).
- No `_done/` file rewrites.
- No changes to `PASS`, `FAIL`, `PENDING_HUMAN`, `UNVERIFIED` tokens.
- No transition period with eventual removal — aliases are permanent.

## Alternatives Considered

- **`HUMAN_CONFIRMED` / `NOT_APPLICABLE(<reason>)`** — too long; compresses
  Matrix columns; inconsistent length with `PASS`/`FAIL`. Rejected.
- **Migrate `_done/` specs** — rewrites archived history, pollutes git blame,
  no user-facing benefit (specs are read-only). Rejected.
- **Remove deprecated aliases after transition** — `_done/` is lint-checked; removal
  would break archived specs. Rejected.

## Open Questions

- None — ready for plan.
