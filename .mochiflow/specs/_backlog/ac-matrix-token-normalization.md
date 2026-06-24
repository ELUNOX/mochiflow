---
slug: "ac-matrix-token-normalization"
title: "Normalize AC Matrix result tokens to ASCII-only"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "ship"
source_spec: "ship-qa-experience"
created: "2026-06-23"
updated: "2026-06-23"
---

# Normalize AC Matrix result tokens to ASCII-only

## Signal

AC Matrix done-eligible tokens are asymmetric: `PASS` and `FAIL` are English
ASCII, but `人間確認済み` and `対象外（<reason>）` are Japanese. This forces
all projects — regardless of `artifact_language` — to write Japanese in the
Matrix. English-speaking users encounter unexpected non-ASCII tokens in an
otherwise English workflow.

## Why It Matters

- Cognitive friction for non-Japanese users encountering fixed Japanese tokens.
- Inconsistency with the Stable Identifiers design principle (machine-readable
  tokens should be locale-independent).
- `lint` hardcodes these strings in Rust; any normalization touches CLI code +
  all existing archived specs under `_done/`.

## Evidence

- `reference/workflow.md ## AC Matrix`: defines `PASS`, `人間確認済み`,
  `対象外（<reason>）`, `FAIL`, `PENDING_HUMAN`, `UNVERIFIED`.
- `reference/language.md ## Stable Identifiers`: lists these as fixed tokens.
- `cli/crates/mochiflow-core/src/lint.rs`: validates tokens against hardcoded
  strings.
- All specs under `_done/` contain `人間確認済み` in their AC Matrix.

## Open Questions

- What should the normalized tokens be? Candidates: `HUMAN_CONFIRMED` /
  `NOT_APPLICABLE(<reason>)`, or `CONFIRMED` / `N/A(<reason>)`.
- Should existing `_done/` specs be migrated, or should lint accept both old and
  new tokens during a transition period?
- Is `N/A: <reason>` (already accepted as an ASCII input equivalent by lint)
  sufficient, or should the canonical form itself change?
