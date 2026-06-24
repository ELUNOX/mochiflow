# Unify QA experience in ship — Tasks

Implementation Summary: rewrite ship QA acceptance as round-trip protocol, add PR Testing section, remove qa-instructions template, update references.
risk: elevated
Critical Stop Conditions:
- Conformance tests fail after template deletion (investigate cause before continuing)
- MANIFEST hash diverges unexpectedly (re-run `mochiflow freeze` only once; if still fails, stop)
- Engine lint breaks on any spec under `_done/` (existing specs must not be invalidated)

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-08] Update spec.standard.md QA Scenarios table to add Type column
  - Depends on: none
  - Files: `engine/templates/spec/spec.standard.md`
  - Done: QA table has columns `| QA | Scope | Type | Steps | Expected result |`
  - Stop: if template change breaks lint on existing `_done/` specs

- [ ] T-002 [AC-01, AC-02, AC-03, AC-04] Rewrite ship.md Acceptance section with round-trip protocol
  - Depends on: none
  - Files: `engine/commands/ship.md`
  - Done: steps 1–3 define: present numbered QA list → collect free-form response → record Matrix token → rework loop on FAIL → resume on all pass
  - Stop: if rewrite conflicts with ship.md PR Feedback Loop or close-out logic

- [ ] T-003 [AC-05] Add PR Feedback Loop triggers to ship.md and router.md
  - Depends on: T-002
  - Files: `engine/commands/ship.md`, `engine/router.md`
  - Done: `{slug} feedback` / 「修正依頼」 / 「PR feedback」 in ship.md trigger_patterns; router handles them by routing to PR Feedback Loop
  - Stop: if new triggers conflict with existing trigger_patterns in other commands

- [ ] T-004 [AC-06] Add `## Testing` section to pr-description.md template
  - Depends on: none
  - Files: `engine/templates/delivery/pr-description.md`
  - Done: template contains `## Testing` with instruction to derive from spec.md QA Scenarios
  - Stop: if the addition conflicts with existing PR body generation in ship.md step 6

- [ ] T-005 [AC-07] Delete qa-instructions.md template and update all references
  - Depends on: T-002, T-004
  - Files: `engine/templates/delivery/qa-instructions.md`, `engine/reference/workflow.md`, `engine/reference/authoring.md`, `.kiro/agents/spec-builder.json`, `engine/adapters/kiro/agents/spec-builder.json.tpl`
  - Done: template file deleted; all prose references point to spec.md QA Scenarios; agent file lists no longer include the deleted path
  - Stop: if deletion breaks `mochiflow freeze` in a way not fixable by re-running freeze

- [ ] T-006 [AC-07] Update conformance tests that reference qa-instructions template
  - Depends on: T-005
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: all 3 `read_repo_file("engine/templates/delivery/qa-instructions.md")` call sites replaced or removed; `cargo test` passes
  - Stop: if test semantics depend on qa-instructions content beyond file existence

- [ ] T-007 [AC-09] Regenerate MANIFEST and run full verification
  - Depends on: T-005, T-006
  - Files: `engine/MANIFEST.json`
  - Done: `mochiflow freeze` exits 0; `mochiflow freeze --check` exits 0; `cargo test` passes; `mochiflow doctor` exits 0; `mochiflow lint --spec ship-qa-experience` passes
  - Stop: if MANIFEST contains entries that should not be there
