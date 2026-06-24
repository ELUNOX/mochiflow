# Normalize AC Matrix result tokens — Tasks

Implementation Summary: add `CONFIRMED` token to lint, update engine docs to use ASCII canonical tokens, keep deprecated aliases.
risk: elevated
Critical Stop Conditions:
- Existing `_done/` specs fail lint after change (deprecated aliases must still pass)
- New token `CONFIRMED` not accepted by lint in done state
- `mochiflow freeze --check` fails after engine doc edits

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01, AC-02, AC-03, AC-04] Add `CONFIRMED` to lint.rs and update error messages
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/lint.rs`
  - Done: `is_canonical_matrix_result` accepts `CONFIRMED`; `is_done_matrix_result` accepts `CONFIRMED`; old tokens still accepted; error messages show new tokens primary with deprecated note
  - Stop: if adding the token requires structural changes beyond the match arms

- [ ] T-002 [AC-06] Update workflow.md canonical token definitions
  - Depends on: none
  - Files: `engine/reference/workflow.md`
  - Done: `CONFIRMED` defined as done-eligible; `N/A: <reason>` as canonical not-applicable; deprecated aliases noted; done-eligible list updated
  - Stop: if workflow.md changes conflict with other sections

- [ ] T-003 [AC-07, AC-11] Update ship.md and build.md to use new canonical tokens
  - Depends on: T-002
  - Files: `engine/commands/ship.md`, `engine/commands/build.md`
  - Done: ship.md step 3c/3f maps to `CONFIRMED`; build.md step 6 uses `N/A: <reason>` instead of `対象外（<reason>）`
  - Stop: if change conflicts with ship.md rework loop or build.md Matrix logic

- [ ] T-004 [AC-08] Update language.md Stable Identifiers
  - Depends on: T-002
  - Files: `engine/reference/language.md`
  - Done: Stable Identifiers list shows `CONFIRMED` and `N/A: <reason>` as canonical; deprecated aliases `人間確認済み` and `対象外（<reason>）` noted
  - Stop: if language.md format makes deprecated notation unclear

- [ ] T-005 [AC-09] Update spec templates Completion Conditions
  - Depends on: T-002
  - Files: `engine/templates/spec/spec.md`, `engine/templates/spec/spec.standard.md`
  - Done: Completion Conditions text uses `CONFIRMED` and `N/A: <reason>` instead of Japanese tokens
  - Stop: if template change breaks lint on existing specs

- [ ] T-006 [AC-05, AC-10] Update conformance tests and run full verification
  - Depends on: T-001, T-003, T-004, T-005
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: (1) existing `lint_done_passes_with_canonical_final_matrix_results` kept as deprecated-alias test; (2) new test added using `CONFIRMED` + `N/A: reason`; (3) new assertion confirms lint error message text contains `CONFIRMED` and `N/A: <reason>` as primary and deprecated aliases as "also accepted"; (4) language.md token assertion updated to expect new canonical tokens; (5) ship.md assertion updated to expect `CONFIRMED`; (6) `engine_templates_are_english_source` exclusion simplified (templates no longer contain Japanese tokens); (7) `cargo test` passes; `mochiflow freeze --check` exit 0; `mochiflow upgrade --source engine` exit 0; `mochiflow doctor` exit 0; (8) elevated-risk independent-reviewer run and verdict recorded in `design.md ## Review Results`
  - Stop: if test migration requires more than assertion text changes
