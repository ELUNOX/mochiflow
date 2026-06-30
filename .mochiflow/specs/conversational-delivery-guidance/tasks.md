# Guide delivery through conversational next actions — Tasks

Implementation Summary: Add conversation-language delivery next actions for PR handoff, in-review status, local cleanup pending, and close completion.
risk: elevated
Critical Stop Conditions:
- Stop if the solution requires a new persisted lifecycle state or `status: done`.
- Stop if the solution requires replacing the broader trigger/frontmatter model.
- Stop if delivery guidance needs to be written into external PR descriptions.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope trigger redesign / new persisted schema / configured default verification keeps failing

## Tasks

- [x] T-001 [AC-02, AC-03, AC-06, AC-07] Update engine delivery guidance
  - Depends on: none
  - Files:
    - `engine/commands/open.md`
    - `engine/router.md`
    - `engine/commands/close.md`
    - `engine/reference/language.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Engine guidance requires conversation-language PR-created handoff for URL and URL-less handoff paths, contextual merge-report routing with disambiguation, conversational close presentation, and language ownership. Conformance tests cover the stable contract phrases without overfitting long prose.
  - Stop: The routing change needs a new command-frontmatter schema or the full activation redesign.
- [x] T-002 [AC-01, AC-07] Add PR-created next-action output
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-cli/tests/pr.rs`
  - Done: Successful automated PR creation, custom-driver creation, legacy command success, and manual handoff all print a conversation-language next action. Tests cover English and Japanese behavior where supported.
  - Stop: PR output changes require altering the PR request schema or PR body generation.
- [ ] T-003 [AC-04, AC-05, AC-07] Render delivery next actions in status and board
  - Depends on: T-002
  - Files:
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-core/src/status.rs`
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: In-review specs render a merge-then-report next action and JSON `next_action.kind = "report_merge"`; done-derived specs with local branch or scratch render `local cleanup pending`, JSON `next_action.kind = "local_cleanup_pending"`, and `local_cleanup_pending = true`; after the local branch and scratch are removed, the same done-derived spec no longer renders `local cleanup pending` and JSON returns `next_action = null`, `local_cleanup_pending = false`; `status` remains read-only; generated board JSON/Markdown expose the hint consistently.
  - Stop: Cleanup-pending detection cannot be derived from local branch or scratch state without persisting new metadata.
- [ ] T-004 [AC-01, AC-02, AC-03, AC-04, AC-05, AC-06, AC-07] Regenerate engine artifacts and verify
  - Depends on: T-003
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - generated adapter outputs if `mochiflow adapter generate` updates them
    - `.mochiflow/specs/conversational-delivery-guidance/spec.md`
    - `.mochiflow/specs/conversational-delivery-guidance/design.md`
    - `.mochiflow/specs/conversational-delivery-guidance/tasks.md`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check` have run; the configured `cli.default` verification command from `mochiflow config show` passes; AC Matrix rows are updated with implementation paths, evidence, and results; mandatory elevated-risk review result is recorded in `design.md`.
  - Stop: Verification reveals delivery-state regression or generated adapter drift outside the planned engine guidance changes.
