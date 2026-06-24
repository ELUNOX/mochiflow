# Normalize AC Matrix result tokens to ASCII-only

## Background and Design Rationale

AC Matrix result tokens are asymmetric: `PASS`, `FAIL`, `PENDING_HUMAN`, and
`UNVERIFIED` are English ASCII, while `人間確認済み` and `対象外（<reason>）` are
Japanese. This forces non-Japanese users to write Japanese in the Matrix
regardless of `artifact_language`, violating the principle that machine-readable
stable identifiers should be locale-independent.

Design decisions:
- **`CONFIRMED`** replaces `人間確認済み` as the canonical human QA pass token.
  Short, consistent with `PASS`/`FAIL` length. `HUMAN_CONFIRMED` rejected as
  redundant (Matrix context implies human).
- **`N/A: <reason>`** is promoted from "ASCII input equivalent" to canonical
  not-applicable token. Already accepted by lint; no new form needed.
  `NOT_APPLICABLE(<reason>)` rejected as too long.
- **Deprecated aliases are permanent** — `人間確認済み` and `対象外（<reason>）`
  remain valid in lint. Archived `_done/` specs are not migrated.

Origin: backlog seed `ac-matrix-token-normalization` (source: ship-qa-experience).

## User Story

As a developer using mochiflow with `artifact_language: en`, I want AC Matrix
tokens to be ASCII-only, so that I do not need to input Japanese characters for
machine-readable workflow tokens.

## Scope

- In: lint.rs token validation (2 functions + error messages), workflow.md,
  ship.md, build.md, language.md, spec.md/spec.standard.md templates,
  conformance tests, MANIFEST.json regeneration.
- Out: `_done/` spec migration, `PASS`/`FAIL`/`PENDING_HUMAN`/`UNVERIFIED`
  changes, deprecated alias removal.

## Edge Cases

- Spec uses mixed old+new tokens in the same Matrix (e.g. one row `CONFIRMED`,
  another `人間確認済み`) — lint accepts both; no error.
- `N/A:` without a reason — lint rejects (existing behavior, unchanged).
- `対象外（）` without a reason — lint rejects (existing behavior, unchanged).

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL accept `CONFIRMED` as a done-eligible AC Matrix result
  token in `is_done_matrix_result`.
- AC-02: THE SYSTEM SHALL accept `CONFIRMED` and `N/A: <reason>` as canonical
  AC Matrix result tokens in `is_canonical_matrix_result`. (`N/A: <reason>` is
  already canonical; this AC confirms it is not removed.)
- AC-03: THE SYSTEM SHALL accept `N/A: <reason>` as a done-eligible AC Matrix
  result token in `is_done_matrix_result` (promoting it from canonical-only to
  done-eligible).
- AC-04: THE SYSTEM SHALL continue to accept `人間確認済み` and
  `対象外（<reason>）` as both canonical and done-eligible (deprecated aliases).
- AC-05: THE SYSTEM SHALL display `CONFIRMED` and `N/A: <reason>` as the primary
  tokens in lint error messages, with deprecated aliases noted as "also accepted".
  A conformance test SHALL assert on the error message text.
- AC-06: THE SYSTEM SHALL update workflow.md to define `CONFIRMED` and
  `N/A: <reason>` as the canonical done-eligible tokens.
- AC-07: THE SYSTEM SHALL update ship.md round-trip protocol to map human pass
  intent to `CONFIRMED` (not `人間確認済み`).
- AC-08: THE SYSTEM SHALL update language.md Stable Identifiers to list
  `CONFIRMED` and `N/A: <reason>` as canonical, with deprecated aliases noted.
- AC-09: THE SYSTEM SHALL update spec.md and spec.standard.md template
  Completion Conditions to use `CONFIRMED` and `N/A: <reason>`.
- AC-10: THE SYSTEM SHALL pass `cargo test`, `mochiflow lint`, `mochiflow
  freeze --check`, `mochiflow upgrade --source engine`, and `mochiflow doctor`
  after all changes.
- AC-11: THE SYSTEM SHALL update build.md to use `N/A: <reason>` instead of
  `対象外（<reason>）` in AC Matrix recording instructions.

## QA Scenarios

| QA | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | cli | Automated | Run `cargo test --manifest-path cli/Cargo.toml` | All tests pass |
| QA-02 | cli | Automated | Run `mochiflow lint` on a test spec with `CONFIRMED` and `N/A: reason` in done state | 0 fail |
| QA-03 | cli | Automated | Run `mochiflow lint` on a `_done/` spec containing `人間確認済み` | 0 fail (deprecated alias accepted) |
| QA-04 | cli | Automated | Run `mochiflow freeze --check` | Exit 0 |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Elevated-risk independent-reviewer verdict (`pass` or `pass-with-comments`)
  recorded in `design.md ## Review Results`.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-02 | `cli/crates/mochiflow-core/src/lint.rs` | UNVERIFIED | | |
| AC-02 | cli | automated | QA-01, QA-02 | `cli/crates/mochiflow-core/src/lint.rs` | UNVERIFIED | | |
| AC-03 | cli | automated | QA-01, QA-02 | `cli/crates/mochiflow-core/src/lint.rs` | UNVERIFIED | | |
| AC-04 | cli | automated | QA-01, QA-03 | `cli/crates/mochiflow-core/src/lint.rs` | UNVERIFIED | | |
| AC-05 | cli | automated | QA-01 | `cli/crates/mochiflow-core/src/lint.rs`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-06 | cli | automated | QA-01 | `engine/reference/workflow.md` | UNVERIFIED | | |
| AC-07 | cli | automated | QA-01 | `engine/commands/ship.md` | UNVERIFIED | | |
| AC-08 | cli | automated | QA-01 | `engine/reference/language.md` | UNVERIFIED | | |
| AC-09 | cli | automated | QA-01 | `engine/templates/spec/spec.md`, `engine/templates/spec/spec.standard.md` | UNVERIFIED | | |
| AC-10 | cli | automated | QA-01, QA-04 | all | UNVERIFIED | | |
| AC-11 | cli | automated | QA-01 | `engine/commands/build.md` | UNVERIFIED | | |
