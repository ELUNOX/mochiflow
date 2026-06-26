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

- [ ] T-001 [AC-01, AC-02] Define choice-card numbering and label vocabulary in shared routing/workflow guidance
  - Depends on: none
  - Files: `engine/router.md`, `engine/reference/workflow.md`, `engine/reference/language.md`
  - Done: shared guidance states that choice numbers are ephemeral aliases for the most recent unambiguous card, visible labels use conversation-language actions, and compatibility tokens remain secondary stable identifiers.
  - Stop: if the rule would require a persistent state model or conflict with artifact-based lifecycle state.

- [ ] T-002 [AC-03, AC-04, AC-05, AC-06, AC-10] Update discuss and plan choice-card instructions
  - Depends on: T-001
  - Files: `engine/commands/discuss.md`, `engine/commands/plan.md`, `engine/templates/handoff/build-session-prompt.md`
  - Done: discuss completion presents `計画を作る` / `再開用プロンプトを作る`; plan draft confirmation presents `計画を確定` with explicit no-implementation wording; selecting `計画を確定` by label or number dispatches the approve-to-build action; plan-confirmed choices present `レビューする`, `実装を開始する`, and `再開用プロンプトを作る` with risk-aware ordering and numbered replies.
  - Stop: if the plan approval gate becomes ambiguous or appears to start implementation immediately.

- [ ] T-003 [AC-07, AC-08, AC-09, AC-10] Update review, build, and ship follow-up prompts
  - Depends on: T-001
  - Files: `engine/commands/review.md`, `engine/commands/build.md`, `engine/commands/ship.md`
  - Done: review completion presents `実装を開始する` / `再開用プロンプトを作る`; build completion presents `PR準備を始める` / `再開用プロンプトを作る`; ship PR approval presents `PRを作成する`; PR text edits are documented as ordinary feedback before PR creation.
  - Stop: if PR-body correction is conflated with PR Feedback Loop or if PR creation can proceed without an explicit approval action.

- [ ] T-004 [AC-11] Sync generated engine artifacts and run verification
  - Depends on: T-002, T-003
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, `cargo run --manifest-path cli/Cargo.toml -- freeze --check`, and `mochiflow lint --spec choice-card-command-ux` pass.
  - Stop: if generated output changes outside the expected engine manifest or vendored engine sync.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | AI-observed | QA-01, QA-06, QA-07; manual review of changed engine text | `engine/router.md`, `engine/reference/workflow.md`, `engine/reference/language.md`, `engine/commands/*.md` | UNVERIFIED | | |
| AC-02 | cli | AI-observed | QA-02, QA-03; manual review of numbered-choice rules | `engine/router.md`, `engine/reference/workflow.md` | UNVERIFIED | | |
| AC-03 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of discuss completion instructions | `engine/commands/discuss.md` | UNVERIFIED | | |
| AC-04 | cli | AI-observed | QA-01, QA-03, QA-07; manual review of plan approval wording | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-05 | cli | AI-observed | QA-03, QA-07; manual review of plan confirmation numbering rule | `engine/commands/plan.md`, `engine/reference/workflow.md` | UNVERIFIED | | |
| AC-06 | cli | AI-observed | QA-01, QA-02, QA-06, QA-07; manual review of plan-confirmed choices | `engine/commands/plan.md` | UNVERIFIED | | |
| AC-07 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of review completion behavior | `engine/commands/review.md` | UNVERIFIED | | |
| AC-08 | cli | AI-observed | QA-01, QA-02, QA-07; manual review of build completion behavior | `engine/commands/build.md` | UNVERIFIED | | |
| AC-09 | cli | AI-observed | QA-03, QA-07; manual review of PR approval and PR text feedback behavior | `engine/commands/ship.md` | UNVERIFIED | | |
| AC-10 | cli | AI-observed | QA-01, QA-07; manual review of resume-prompt placement | `engine/commands/discuss.md`, `engine/commands/plan.md`, `engine/commands/review.md`, `engine/commands/build.md`, `engine/commands/ship.md` | UNVERIFIED | | |
| AC-11 | cli | automated | QA-04, QA-05, QA-06; configured CLI verification and engine sync checks | `engine/MANIFEST.json`, `.mochiflow/engine/**` | UNVERIFIED | | |
