# Normalize AC Matrix result tokens — Design

## Design Decisions

- **Token choice `CONFIRMED`**: short (9 chars), pairs with `PASS` (4) / `FAIL`
  (4) without dominating column width. Unambiguous in Matrix context (only human
  QA results use it). Source: pitch.md agreement.

- **`N/A: <reason>` promotion**: lint already accepts this as an ASCII input
  equivalent. Promoting it to canonical avoids introducing a third form. The
  colon-space delimiter is unambiguous and already parsed by
  `strip_prefix("N/A: ")` in lint.rs. Source: existing lint.rs code.

- **Permanent deprecated aliases**: `人間確認済み` and `対象外（<reason>）` stay
  in `is_canonical_matrix_result` and `is_done_matrix_result` forever. Removal
  would break archived `_done/` specs that are lint-checked. Source: pitch.md
  No-gos.

- **Error message format**: show new tokens as primary list, then
  "(also accepted: 人間確認済み, 対象外（<reason>）)" in parentheses. This guides
  new usage while explaining why old tokens don't fail.

## Architecture

| File | Change |
| --- | --- |
| `cli/crates/mochiflow-core/src/lint.rs` | Add `CONFIRMED` to both match arms; add `N/A: <reason>` to `is_done_matrix_result` (it is already in `is_canonical_matrix_result` via `strip_prefix`); update 2 error message format strings |
| `engine/reference/workflow.md` | Redefine canonical tokens; update done-eligible list |
| `engine/commands/ship.md` | Round-trip protocol: `CONFIRMED` replaces `人間確認済み` |
| `engine/commands/build.md` | AC Matrix recording instruction: `N/A: <reason>` replaces `対象外（<reason>）` |
| `engine/reference/language.md` | Stable Identifiers: new tokens primary, deprecated noted |
| `engine/templates/spec/spec.md` | Completion Conditions: use new tokens |
| `engine/templates/spec/spec.standard.md` | Completion Conditions: use new tokens |
| `cli/crates/mochiflow-cli/tests/conformance.rs` | Update assertions to expect new tokens in docs; add test for `CONFIRMED` acceptance; keep deprecated alias test |
| `engine/MANIFEST.json` | Regenerated via `mochiflow freeze` |

## Error Handling

- No new error paths. Deprecated aliases simply remain in the match arms.

## Test Strategy

- Existing `lint_done_passes_with_canonical_final_matrix_results` test uses
  `人間確認済み` — keep it as the deprecated-alias test.
- Add a parallel test using `CONFIRMED` and `N/A: reason` to prove new tokens
  work.
- `engine_templates_are_english_source` test strips `人間確認済み`/`対象外` before
  checking for Japanese — after this change the templates will have no Japanese
  tokens to strip; simplify the exclusion list or remove it.
- Conformance test checking language.md tokens: update expected list to new
  canonical tokens + note deprecated aliases remain in the file.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Findings: 1 Medium (unrelated backlog seed deletions included in branch — intentional, seeds were resolved in PR #24), 1 Low (pre-existing UNVERIFIED inconsistency in language.md — out of scope)
- No required fixes blocking ship.
