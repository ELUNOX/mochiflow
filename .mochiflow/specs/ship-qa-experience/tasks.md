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

- [x] T-001 [AC-08] Update spec.standard.md QA Scenarios table to add Type column
  - Depends on: none
  - Files: `engine/templates/spec/spec.standard.md`
  - Done: QA table has columns `| QA | Scope | Type | Steps | Expected result |`
  - Stop: if template change breaks lint on existing `_done/` specs

- [x] T-002 [AC-01, AC-02, AC-03, AC-04] Rewrite ship.md Acceptance section with round-trip protocol
  - Depends on: none
  - Files: `engine/commands/ship.md`
  - Done: steps 1–3 define: present numbered QA list → accept per-item or numbered batch response → map to QA item by number → record Matrix token → rework loop on FAIL (re-present failed items + regressed items) → resume on all pass
  - Stop: if rewrite conflicts with ship.md PR Feedback Loop or close-out logic

- [x] T-003 [AC-05] Add PR Feedback Loop triggers to ship.md and router.md
  - Depends on: T-002
  - Files: `engine/commands/ship.md`, `engine/router.md`
  - Done: `{slug} feedback` / 「修正依頼」 / 「PR feedback」 in ship.md trigger_patterns; router Decision Flow step 3 adds a `_done/{slug}` resolution exception for feedback patterns (mirroring the merged-event exception); router routes these to `## PR Feedback Loop`
  - Stop: if new triggers conflict with existing trigger_patterns in other commands

- [x] T-004 [AC-06] Add `## Testing` section to pr-description.md template
  - Depends on: none
  - Files: `engine/templates/delivery/pr-description.md`
  - Done: template contains `## Testing` with instruction to derive from spec.md QA Scenarios
  - Stop: if the addition conflicts with existing PR body generation in ship.md step 6

- [x] T-005 [AC-07] Delete qa-instructions.md template and update all references
  - Depends on: T-002, T-004
  - Files: `engine/templates/delivery/qa-instructions.md`, `engine/commands/ship.md`, `engine/reference/workflow.md`, `engine/reference/authoring.md`, `engine/adapters/kiro/agents/spec-builder.json.tpl`
  - Done: template file deleted; ship.md frontmatter `artifacts:` and `references:` lines removed; all prose references point to spec.md QA Scenarios; Kiro adapter template line removed; `mochiflow adapter generate` regenerates `.kiro/agents/spec-builder.json` cleanly
  - Stop: if deletion breaks `mochiflow freeze` in a way not fixable by re-running freeze

- [ ] T-006 [AC-07] Update conformance tests that reference qa-instructions template
  - Depends on: T-005
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: all 3 call sites replaced: (1) `branch_placeholders_use_prefix_slug` — remove qa read (git.md assertion already covers the guarantee); (2) `english_template_headings_are_present` — remove qa heading assertions (headings are gone; spec.standard.md QA table and pr-description.md `## Testing` are already tested via their own template reads); (3) `ac_matrix_pending_human_is_canonical_provisional_token` — move the token-mapping assertion to ship.md (`ship.contains("人間確認済み")` confirms the round-trip protocol preserves the mapping). `cargo test` passes.
  - Stop: if test semantics depend on qa-instructions content beyond file existence

- [ ] T-007 [AC-09] Sync vendored engine, regenerate MANIFEST, and run full verification
  - Depends on: T-005, T-006
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`
  - Done: `mochiflow upgrade --source engine` exits 0 (vendored copy synced); `mochiflow freeze` exits 0; `mochiflow freeze --check` exits 0; `mochiflow adapter generate --check` exits 0; `cargo test` passes; `mochiflow doctor` exits 0; `mochiflow lint --spec ship-qa-experience` passes
  - Stop: if MANIFEST contains entries that should not be there
