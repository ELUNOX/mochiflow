# Guide delivery through conversational next actions

## Background and Design Rationale

MochiFlow's delivery flow is intended to be agent-led, but the current PR
handoff can still feel CLI-led. After `open` creates a PR, the agent and
`mochiflow pr` can stop at the PR URL even though the user still needs to merge
the PR externally and return for local cleanup. New users can miss that return
path because they do not normally think in terms of running MochiFlow commands.

The improvement is to make delivery guidance durable and conversational:
show the next human action after PR creation, keep it visible while the work is
in review, surface local cleanup when the PR is already merged, and let simple
merge reports route to cleanup when the active context is unambiguous.

This spec folds the useful delivery-specific concern from the removed
`phase-completion-guidance` backlog seed into a concrete delivery flow. It stays
separate from `trigger-routing-redesign`, which may later replace the broader
trigger/frontmatter model.

## User Story

As a developer using an AI agent, I want PR delivery to tell me what to do next
in normal conversation, so that I can merge externally and return for cleanup
without memorizing MochiFlow commands or exact trigger syntax.

## Scope

- In:
  - PR-created success guidance from `mochiflow pr` and `commands/open.md`.
  - Contextual merge-report routing in `engine/router.md`.
  - In-review and local-cleanup-pending next-action hints in status / board
    rendering.
  - Conversational `close` start and completion guidance.
  - Language-aware English/Japanese coverage for the new guidance.
- Out:
  - Replacing command frontmatter triggers or the full activation model.
  - Adding a new cleanup CLI command.
  - Persisting cleanup completion in spec metadata.
  - Writing `status: done` or moving active specs into `_done/`.
  - Adding local cleanup instructions to external PR descriptions.

## Edge Cases

- The user says only "merged" / "マージした" while exactly one accepted
  in-review or cleanup-pending spec is available.
- The user gives a bare merge report while multiple accepted or in-review specs
  could match.
- The user gives a bare merge report while no accepted in-review or
  cleanup-pending spec exists to match.
- The current branch points at the just-opened spec when the user reports the
  merge.
- Provider status is unavailable, but `origin/main` already contains a commit
  with the spec's `Spec:` trailer.
- A PR is merged but the local feature branch or gitignored delivery scratch
  still exists.
- `conversation_language = auto` is configured in a non-interactive CLI-only
  context.

## Acceptance Criteria (EARS)

- AC-01: WHEN `mochiflow pr` successfully creates or hands off a PR, THE SYSTEM
  SHALL show a conversation-language next action telling the user to merge the
  PR and then report the merge in chat.
- AC-02: WHEN `commands/open.md` describes PR creation completion, THE SYSTEM
  SHALL require the agent's final PR-created response to include the PR URL
  when one is available; WHEN the PR backend uses manual handoff or another
  URL-less path, THE SYSTEM SHALL require the response to describe the handoff
  and still include the conversational post-merge next action.
- AC-03: WHEN a user gives a simple merge-report intent and there is one
  unambiguous accepted in-review or cleanup-pending spec, THE SYSTEM SHALL route
  to `close`; IF multiple candidates exist, THEN THE SYSTEM SHALL ask one
  disambiguation question instead of guessing; IF no accepted in-review or
  cleanup-pending candidate exists, THEN THE SYSTEM SHALL NOT route to cleanup
  and SHALL fall through to normal routing.
- AC-04: WHILE an accepted spec is in review, THE SYSTEM SHALL render a next
  action in status / board output that tells the user to merge the PR and report
  the merge in conversation.
- AC-05: WHEN a spec is derived as done but local delivery cleanup is still
  pending, THE SYSTEM SHALL render a `local cleanup pending` next action without
  writing a new lifecycle state.
- AC-06: WHEN `close` starts or completes, THE SYSTEM SHALL present the work as
  post-merge local cleanup in conversation-language wording while preserving the
  rule that close writes nothing to the base branch.
- AC-07: THE SYSTEM SHALL keep delivery guidance language-aware: conversational
  guidance follows `conversation_language`, durable artifacts and PR
  descriptions follow `artifact_language`, and merge-report examples remain
  intent examples rather than fixed trigger strings.

## QA Scenarios

| QA | Persona | Scope | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | P1 new user, P7 spec skeptic | cli | Create a PR from an accepted spec using the configured PR path. | Output and agent guidance tell the user to merge the PR and then report the merge in chat; no command memorization is required. |
| QA-02 | P2 power user | cli | Render status / board while one accepted spec is in review. | The spec remains in review and includes the conversational next action. |
| QA-03 | P2 power user, P3 malicious user | cli | Present a bare merge report when multiple accepted or in-review specs could match, and separately when no accepted in-review or cleanup-pending spec exists. | With multiple candidates the router asks one disambiguation question and does not run cleanup against an arbitrary spec; with no candidate it does not route to cleanup and falls through to normal routing. |
| QA-04 | P4 data integrity | cli | Inspect accepted / merged lifecycle data after cleanup-pending detection. | No new persisted cleanup state, `status: done`, or `_done/` move is written. |
| QA-05 | P5 migration | cli | N/A: this change adds presentation and derived local hints only; it does not migrate persisted data or schemas. | N/A: no migration path is required. |
| QA-06 | P6 regression | cli | Exercise existing explicit `{slug} merged` routing and post-merge local cleanup rules. | Existing exact slug close routing still works and close remains local hygiene only. |
| QA-07 | P7 spec skeptic | cli | Compare English and Japanese guidance examples against configured language behavior. | Conversation guidance follows conversation language; PR body / durable artifact text remains governed by artifact language. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Mandatory elevated-risk reviewer verdict is recorded before acceptance.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated + AI-observed | QA-01, QA-07; PR command tests for automated and manual paths | `cli/crates/mochiflow-core/src/pr.rs`; `cli/crates/mochiflow-cli/tests/pr.rs` | PASS | `cargo test`: `pr::tests::pr_next_action_is_language_aware`, integration `pr_driver_success_prints_next_action`, `pr_manual_handoff_prints_next_action`, `pr_next_action_uses_japanese_conversation_language` | English and Japanese output paths covered; helper shared by all four backends. |
| AC-02 | cli | AI-observed + automated | QA-01; conformance assertion over open command presentation for URL and URL-less handoff paths | `engine/commands/open.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test`: `conformance::delivery_guidance_is_conversational_and_language_aware` (asserts URL-when-available + URL-less handoff + never-in-PR-body) | Agent presentation does not rely on PR body text; URL required only when the backend produced one. |
| AC-03 | cli | AI-observed + automated | QA-03, QA-06; router conformance for bare merge-report intent and ambiguity handling | `engine/router.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test`: `conformance::delivery_guidance_is_conversational_and_language_aware` (asserts close routing, one-question disambiguation, no-candidate fallthrough) | Exact slug close routing remains a compatibility path. |
| AC-04 | cli | automated | QA-02, QA-07; status/index rendering tests for in-review next actions and JSON `next_action.kind = "report_merge"` | `cli/crates/mochiflow-core/src/status.rs`; `cli/crates/mochiflow-core/src/index.rs`; tests | PASS | `cargo test`: `status::tests::render_board_shows_next_action_lines`, `index::tests::build_json_exposes_next_action_contract`, `index::tests::generate_index_json_carries_delivery_next_actions` | Status is read-only; index remains generated state; JSON board contract includes `next_action` and `local_cleanup_pending`. |
| AC-05 | cli | automated | QA-04; delivery/status/index tests for local cleanup pending positive and post-cleanup negative cases, including JSON `next_action.kind = "local_cleanup_pending"` and `local_cleanup_pending = true` | `cli/crates/mochiflow-core/src/delivery.rs`; `cli/crates/mochiflow-core/src/status.rs`; `cli/crates/mochiflow-core/src/index.rs`; tests | PASS | `cargo test`: `delivery::tests::done_with_local_branch_is_cleanup_pending`, `delivery_scratch_alone_triggers_cleanup_pending`, `done_after_branch_and_scratch_removed_has_no_next_action`, `legacy_done_status_has_no_cleanup_pending`, `index::tests::generate_index_json_carries_delivery_next_actions` (post-removal `next_action = null`, `local_cleanup_pending = false`) | Hint derived from local branch/scratch; does not alter lifecycle metadata; clears after branch/scratch removal. |
| AC-06 | cli | AI-observed + automated | QA-06; conformance assertion over close presentation and existing close local-only behavior; post-close status/index negative check for cleanup guidance | `engine/commands/close.md`; `engine/reference/git.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test`: `conformance::close_is_local_hygiene_only`, `conformance::delivery_guidance_is_conversational_and_language_aware` (At close start / At close completion); post-cleanup negative via `index::tests::generate_index_json_carries_delivery_next_actions` | No base-branch commit or push during close; local cleanup removes the facts that trigger cleanup-pending guidance. |
| AC-07 | cli | automated + AI-observed | QA-07; i18n-focused rendering tests and language-reference conformance | `engine/reference/language.md`; `cli/crates/mochiflow-core/src/pr.rs`; `status/index` rendering helpers; tests | PASS | `cargo test`: `pr::tests::pr_next_action_is_language_aware`, `delivery::tests::next_action_kind_is_language_aware`, `index::tests::build_json_next_action_message_is_language_aware`, `pr_next_action_uses_japanese_conversation_language`; `conformance::delivery_guidance_is_conversational_and_language_aware` (language.md ownership) | `conversation_language = auto` falls back deterministically via `Config::conversation_output_language()` for CLI-only output. |
