# Clarify choice-card commands and numbered replies — Tasks

Implementation Summary: update engine workflow instructions so phase choice cards use user-facing labels, safe numbered replies, and limited resume-prompt placement.
risk: elevated
Critical Stop Conditions:
- A numbered reply would require persistent conversation state or CLI runtime support.
- PR creation approval becomes implicit or ambiguous.
- Engine source and vendored engine drift after verification.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02] Define choice-card numbering and label vocabulary in shared routing/workflow guidance
  - Depends on: none
  - Files: `engine/router.md`, `engine/reference/workflow.md`, `engine/reference/language.md`
  - Done: shared guidance states that choice numbers are ephemeral aliases for the most recent unambiguous card, visible labels use conversation-language actions, and compatibility tokens remain secondary stable identifiers.
  - Stop: if the rule would require a persistent state model or conflict with artifact-based lifecycle state.

- [x] T-002 [AC-03, AC-04, AC-05, AC-06, AC-10] Update discuss and plan choice-card instructions
  - Depends on: T-001
  - Files: `engine/commands/discuss.md`, `engine/commands/plan.md`, `engine/templates/handoff/build-session-prompt.md`
  - Done: discuss completion presents localized create-plan / resume-prompt actions; plan draft confirmation presents localized plan-confirmation wording with explicit no-implementation wording; selecting the plan-confirmation action by label or number dispatches the approve-to-build action; plan-confirmed choices present localized review / start-implementation / resume-prompt actions with risk-aware ordering and numbered replies.
  - Stop: if the plan approval gate becomes ambiguous or appears to start implementation immediately.

- [x] T-003 [AC-07, AC-08, AC-09, AC-10] Update review, build, and ship follow-up prompts
  - Depends on: T-001
  - Files: `engine/commands/review.md`, `engine/commands/build.md`, `engine/commands/ship.md`
  - Done: review completion presents localized start-implementation / resume-prompt actions only from a plan-confirmed or `status: approved` context; build completion presents localized PR-preparation / resume-prompt actions; the build-completion resume prompt is inline and points the next session to `{slug} ship`; ship PR approval presents a localized create-PR action; PR text edits are documented as ordinary feedback before PR creation.
  - Stop: if PR-body correction is conflated with PR Feedback Loop or if PR creation can proceed without an explicit approval action.

- [x] T-004 [AC-11] Sync generated engine artifacts and run verification
  - Depends on: T-002, T-003
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, `cargo run --manifest-path cli/Cargo.toml -- freeze --check`, and `mochiflow lint --spec choice-card-command-ux` pass.
  - Stop: if generated output changes outside the expected engine manifest or vendored engine sync.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | AI-observed | QA-01, QA-06, QA-07; manual review of changed engine text | `engine/router.md`, `engine/reference/workflow.md`, `engine/reference/language.md`, `engine/commands/*.md` | PASS | `workflow.md ## Choice cards`, `language.md ## Conversation Language`, command presentation sections | Plain labels are primary; compatibility keywords remain documented. |
| AC-02 | cli | AI-observed | QA-02, QA-03; manual review of numbered-choice rules | `engine/router.md`, `engine/reference/workflow.md` | PASS | `workflow.md ## Choice cards`; `router.md ## Completion Output` | Numbers are scoped to the most recent unambiguous choice card. |
| AC-03 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of discuss completion instructions | `engine/commands/discuss.md` | PASS | `discuss.md` step 9 | Discuss completion uses localized create-plan and resume-prompt actions. |
| AC-04 | cli | AI-observed | QA-01, QA-03, QA-07; manual review of plan approval wording | `engine/commands/plan.md` | PASS | `plan.md` step 7 | Plan confirmation explains `status: approved`, re-check, commit, and no implementation start. |
| AC-05 | cli | AI-observed | QA-03, QA-07; manual review of plan confirmation numbering rule | `engine/commands/plan.md`, `engine/reference/workflow.md` | PASS | `workflow.md ## Delivery approval gates`; `plan.md` step 7 | Choice selection dispatches the approve-to-build action by label or number. |
| AC-06 | cli | AI-observed | QA-01, QA-02, QA-06, QA-07; manual review of plan-confirmed choices | `engine/commands/plan.md` | PASS | `plan.md` step 10 | Plan-confirmed card presents review/build/resume with risk-aware ordering. |
| AC-07 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of review completion behavior | `engine/commands/review.md` | PASS | `review.md ## Presentation` | Build/resume follow-up is limited to approved implementation-ready context. |
| AC-08 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of build completion behavior | `engine/commands/build.md` | PASS | `build.md ## Presentation` | Build completion presents PR-prep/resume and inline `{slug} ship` resume guidance. |
| AC-09 | cli | AI-observed | QA-03, QA-07; manual review of PR approval and PR text feedback behavior | `engine/commands/ship.md` | PASS | `ship.md` PR steps 6-7 | PR approval action is localized create-PR; text corrections re-present the PR card. |
| AC-10 | cli | AI-observed | QA-01, QA-07; manual review of resume-prompt placement | `engine/commands/discuss.md`, `engine/commands/plan.md`, `engine/commands/review.md`, `engine/commands/build.md`, `engine/commands/ship.md` | PASS | command presentation/procedure sections | Resume prompt is displayed at discuss, plan, review, and build handoff points only. |
| AC-11 | cli | automated | QA-04, QA-05, QA-06; configured CLI verification and engine sync checks | `engine/MANIFEST.json`, `.mochiflow/engine/**` | PASS | Build transcript: `mochiflow upgrade --source engine` printed `upgraded engine <- .../engine`; `mochiflow adapter generate --check` printed `Summary: 0 drift, 0 failed`; `cargo test --manifest-path cli/Cargo.toml` passed all suites (`72`, `111`, `7`, `18`, `74` tests plus unit/doc tests); `cargo fmt --manifest-path cli/Cargo.toml --all -- --check` and `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` exited 0; `cargo run --manifest-path cli/Cargo.toml -- freeze --check` printed `freeze: all derived files are up to date`; `mochiflow lint --spec choice-card-command-ux` printed `Summary: 0 fail, 0 warn`. | Adapter check, full default verification, freeze check, and spec lint all passed. |
