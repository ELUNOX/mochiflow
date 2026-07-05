# Add review budget loops with optional automatic fixes

## Background and Design Rationale

`engine/commands/review.md` currently defines ad-hoc review as report-only: it
runs `plan-auditor` before implementation or `change-reviewer` after
implementation, reports findings, and does not edit files. That report-only
path is still needed when the user wants to hand the review result to another
agent or human.

The missing path is a bounded "review and fix" workflow where the main agent
uses reviewer findings to apply straightforward in-scope fixes. The command
surface should remain centered on `review`, and the budget should be small and
explicit so review cannot turn into an unbounded fix-until-pass loop.

Key design decisions:

- Keep `{slug} review` as read-only and report-only.
- Add `{slug} review fix [N]` as the automatic-fix path, with `N` meaning the
  maximum number of fix rounds. If omitted, `N = 1`.
- End `review fix N` after the final requested fix round. The command should not
  print a caveat that a final post-fix review is absent.
- Keep reviewers read-only. The main agent applies fixes, verifies according to
  the current lifecycle context, and decides when to stop.
- Keep review cycles fresh and independent: each reviewer receives the current
  artifacts or current full diff, plus cycle-local changed files or diff as
  focus input when useful, but never receives prior findings, prior verdicts,
  prior reviewer summaries, or conversation history.
- Persist review-fix loop recovery in a local state ledger under
  `{install_dir}/state/{slug}/`, for the main agent only. The ledger records the
  requested fix budget, completed fix rounds, current phase/profile, touched
  files, verification evidence, and stop reason; it is never reviewer input.
- Reuse the existing shared bounded-fix judgment: if a finding requires a task
  structure change, new AC, new design decision, spec split, or human judgment,
  stop instead of fixing automatically.

Rejected alternatives from the pitch:

- Adding a second public verb such as `revise` or `refine`.
- Using `{slug} review 2`, which cannot distinguish two read-only opinions from
  one automatic-fix budget.
- Ending every fix budget with a final review pass.
- Adding severity/gate flags such as `--gate medium`.

## User Story

As a developer using MochiFlow, I want to choose between a one-pass review report
and bounded automatic review fixes, so that I can either hand findings to
another agent or let the current agent fix straightforward issues without
opening an unbounded review loop.

## Scope

- In:
  - `engine/commands/review.md`: extend the non-phase review command with
    `{slug} review fix`, `{slug} review fix 2`, and `{slug} review fix 3`;
    keep `{slug} review` report-only.
  - `engine/reference/risk.md`: define review-fix loop rules, automatic-fix
    boundaries, fresh independent cycle inputs, and stop conditions in the
    review reference section.
  - `engine/commands/plan.md`: update choice-card wording so elevated+
    pre-approval review can offer result-only review and review+fix without
    making review a delivery gate.
  - `engine/commands/build.md`, `engine/commands/open.md`, and
    `engine/commands/update.md`: define how `review fix` applies when code
    exists or when an in-review branch receives requested fixes.
  - `engine/router.md`: update review trigger pattern coverage and review
    delegation summary without introducing a new verb.
  - `engine/agents/plan-auditor.md` and `engine/agents/change-reviewer.md`:
    make explicit that review-loop focus input may include cycle-local changed
    files/diff, while previous findings/verdicts/summaries remain hidden from
    the reviewer.
  - `cli/crates/mochiflow-cli/tests/conformance.rs`: pin the command grammar,
    report-only vs fix-mode split, fresh independent review rule, no-worker
    boundary, choice-card mapping, and conformance with existing mandatory
    review cadence.
  - User-facing docs where `mochiflow-review` is described (`README.md`,
    `README.ja.md`, and `docs/concepts.md` if the local docs mention ad-hoc
    review behavior).
  - Dogfood sync after engine edits: `mochiflow freeze`,
    `mochiflow upgrade --source engine`, and
    `mochiflow adapter generate --check`.
- Out:
  - A new public verb (`revise`, `refine`, etc.).
  - New severity/gate flags or CLI-style options.
  - Write-capable reviewer or worker agents.
  - Changing the two delivery approval gates.
  - Changing mandatory risk-cadence review requirements for build completion.
  - Making Low / nit / optional findings part of automatic stop conditions.
  - Passing previous reviewer findings to the next reviewer.

## Edge Cases

- `{slug} review fix` with no number: use one fix round.
- `{slug} review fix 1`: accept as equivalent to `{slug} review fix`.
- `{slug} review fix 0` or `fix 4+`: reject with a concise correction that
  allowed fix rounds are 1, 2, or 3.
- `{slug} review 2`: reject as ambiguous and point to `{slug} review fix 2` for
  automatic fixes.
- A report-only review finds High/Critical issues: report findings and stop,
  preserving the current read-only behavior.
- A fix-mode review finds only non-blocking or non-fixable findings: report the
  remaining findings and stop without making unrelated edits.
- A fix requires new AC, task restructuring, a design decision, or spec split:
  stop and route back to plan/discussion rather than auto-fixing.
- A later review cycle rediscovers the same issue after a fix: stop and report
  the repeated issue instead of spending the remaining budget.
- Code exists but the current lifecycle state is in review: use update-style
  bounded fix discipline and avoid pushing unless the active command flow
  explicitly reaches its push/finalize boundary.
- A final fix round leaves no post-fix review. This is expected by contract; do
  not add user-facing caveat wording solely to point this out.
- A review-fix loop is interrupted between rounds: recover requested budget,
  completed fix rounds, touched files, verification evidence, and stop reason
  from the local state ledger. Continue to keep previous findings and ledger
  contents out of reviewer input.

## Acceptance Criteria (EARS)

- AC-01: WHEN the user invokes `{slug} review`, THE SYSTEM SHALL run exactly
  one read-only ad-hoc review using the state-appropriate reviewer profile and
  SHALL NOT edit files, stage, commit, push, change status, or create PR
  metadata.
- AC-02: WHEN the user invokes `{slug} review fix` or `{slug} review fix 1`,
  THE SYSTEM SHALL run one review, apply at most one in-scope fix round on the
  main agent, verify according to the current lifecycle context, and stop after
  that fix round.
- AC-03: WHEN the user invokes `{slug} review fix 2` or `{slug} review fix 3`,
  THE SYSTEM SHALL run up to the requested number of independent review/fix
  rounds, with the number meaning maximum fix rounds, and SHALL stop after the
  final fix round rather than requiring a clean post-fix review.
- AC-04: WHEN any review cycle after the first is started, THE SYSTEM SHALL pass
  the reviewer the current artifacts or current full diff plus cycle-local focus
  input when useful, and SHALL NOT pass previous findings, previous verdicts,
  previous reviewer summaries, local state ledger contents, or conversation
  history.
- AC-05: WHEN a review finding requires a task-structure change, new AC, new
  design decision, spec split, human judgment, unrelated work, or repeated
  failure after a prior fix, THE SYSTEM SHALL stop the review-fix loop and
  report the decision point instead of applying an automatic fix, and SHALL
  record the stop reason in the local review-fix state ledger.
- AC-06: WHEN a review-fix loop runs before implementation, THE SYSTEM SHALL use
  `plan-auditor` and restrict edits to spec artifacts; WHEN code exists, THE
  SYSTEM SHALL use `change-reviewer` and apply fixes using the current
  build/open/update bounded-fix discipline; in both cases, THE SYSTEM SHALL
  update the local review-fix state ledger when a fix round starts or completes.
- AC-07: WHEN a plan/build/open/update choice card offers review, THE SYSTEM
  SHALL distinguish "Review results" from "Review and fix" while mapping both
  labels to the same `review` command family and SHALL NOT introduce another
  public verb.
- AC-08: THE SYSTEM SHALL reject ambiguous or out-of-range forms such as
  `{slug} review 2`, `{slug} review fix 0`, and `{slug} review fix 4`, and SHALL
  explain the allowed result-only and fix-budget forms.
- AC-09: THE edited engine contracts SHALL preserve reviewer read-only status,
  main-agent implementation ownership, the two delivery approval gates, and the
  existing mandatory risk-cadence review requirements.
- AC-10: THE implementation SHALL add conformance tests that pin the review
  grammar, report-only behavior, fix-budget behavior, fresh independent review
  inputs, choice-card mapping, and no-worker/no-extra-verb boundaries.
- AC-11: THE edited engine SHALL be synced via `mochiflow freeze` →
  `mochiflow upgrade --source engine` → `mochiflow adapter generate --check`,
  and SHALL pass the surface `default` verification, `mochiflow lint`, and
  `mochiflow doctor`.

## QA Scenarios

| QA | Dimension | Scope | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | On a draft elevated spec, choose the plan card action for review results. | One `plan-auditor` run reports findings only; no spec files are edited and `status` remains `draft`. |
| QA-02 | QA-FUNC | cli | On a draft elevated spec, choose the plan card action for review and fix. | `plan-auditor` findings are applied only when in-scope for spec artifacts; the flow stops after the requested fix budget. |
| QA-03 | QA-ABUSE | cli | Invoke `{slug} review 2`, `{slug} review fix 0`, and `{slug} review fix 4`. | Each invalid form is rejected with a concise correction; no review or edit runs. |
| QA-04 | QA-DATA | cli | Run `review fix 2` after a first fix changes files, then inspect the second reviewer input contract. | The next reviewer receives current artifacts/diff and cycle-local focus input, not previous findings or verdicts. |
| QA-05 | QA-COMPAT | cli | Read `engine/reference/risk.md`, `engine/commands/build.md`, `engine/commands/open.md`, and `engine/commands/update.md` after implementation. | Mandatory risk-cadence review, verdict freshness, and update hold/finalize semantics remain intact. |
| QA-06 | QA-RESIL | cli | End a session after one review-fix round, then resume from repository files and the local review-fix state ledger. | The next session can recover requested budget, completed fix rounds, touched files, verification evidence, and stop reason without hidden conversation history, while still withholding previous findings and ledger contents from later reviewers. |
| QA-07 | QA-UX | cli | Read the plan/build/open/update choice-card labels and invalid-form correction wording after implementation. | Result-only review and review-and-fix are visibly distinct, compatibility keywords stay secondary, unsupported numeric forms get concise correction wording, and no extra public verb is introduced. |
| QA-08 | QA-REG | cli | Run the full conformance suite and default verification after engine edits and dogfood sync. | Existing reviewer, router, update, build, and adapter tests still pass; new review-fix tests pass. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated + AI-observed | QA-01, QA-08 | `engine/commands/review.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `ad_hoc_review_is_report_only`, `review_fix_budget_grammar_is_pinned`; inline `change-reviewer` pass through `b229100` | report-only review remains read-only |
| AC-02 | cli | automated + AI-observed | QA-02, QA-08 | `engine/commands/review.md`, `engine/reference/risk.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_budget_grammar_is_pinned`, `review_fix_loop_boundaries_are_pinned`; inline `change-reviewer` pass through `b229100` | one fix round |
| AC-03 | cli | automated + AI-observed | QA-02, QA-08 | `engine/commands/review.md`, `engine/reference/risk.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_budget_grammar_is_pinned`, `review_fix_loop_boundaries_are_pinned`; inline `change-reviewer` pass through `b229100` | fix budget semantics |
| AC-04 | cli | automated + AI-observed | QA-04, QA-06, QA-08 | `engine/reference/risk.md`, `engine/agents/plan-auditor.md`, `engine/agents/change-reviewer.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_loop_boundaries_are_pinned`; inline `change-reviewer` pass through `b229100` | fresh independent review cycles |
| AC-05 | cli | automated + AI-observed | QA-03, QA-05, QA-06, QA-08 | `engine/reference/risk.md`, `engine/commands/review.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_budget_grammar_is_pinned`, `review_fix_loop_boundaries_are_pinned`; inline `change-reviewer` pass through `b229100` | stop conditions |
| AC-06 | cli | automated + AI-observed | QA-01, QA-02, QA-05, QA-06, QA-08 | `engine/commands/review.md`, `engine/reference/risk.md`, `engine/commands/build.md`, `engine/commands/open.md`, `engine/commands/update.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_budget_grammar_is_pinned`, `review_fix_choice_cards_and_phase_discipline_are_pinned`; inline `change-reviewer` pass through `b229100` | profile selection and phase discipline |
| AC-07 | cli | automated + AI-observed | QA-01, QA-02, QA-07, QA-08 | `engine/commands/plan.md`, `engine/commands/build.md`, `engine/commands/open.md`, `engine/commands/update.md`, `engine/router.md`, `README.md`, `README.ja.md`, `docs/concepts.md`, `docs/configuration.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_choice_cards_and_phase_discipline_are_pinned`, `public_docs_explain_review_fix_budget`; inline `change-reviewer` pass through `b229100` | choice-card mapping |
| AC-08 | cli | automated + AI-observed | QA-03, QA-07, QA-08 | `engine/commands/review.md`, `engine/router.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_budget_grammar_is_pinned`; inline `change-reviewer` pass through `b229100` | invalid forms |
| AC-09 | cli | automated + AI-observed | QA-05, QA-08 | `engine/reference/workflow.md`, `engine/reference/risk.md`, `engine/commands/build.md`, `engine/commands/open.md`, `engine/commands/update.md`, `engine/agents/*.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; conformance `review_fix_loop_boundaries_are_pinned`, `review_fix_choice_cards_and_phase_discipline_are_pinned`; inline `change-reviewer` pass through `b229100` | no worker, no gate changes |
| AC-10 | cli | automated | QA-08 | `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; new conformance tests `review_fix_budget_grammar_is_pinned`, `review_fix_loop_boundaries_are_pinned`, `review_fix_choice_cards_and_phase_discipline_are_pinned`, `public_docs_explain_review_fix_budget`<br>final verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` | pinned conformance coverage |
| AC-11 | cli | automated | QA-08 | `engine/MANIFEST.json`, `.mochiflow/engine/`, adapters, `cli/`, `.mochiflow/specs/review-fix-budget/*` | PASS | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check` (`0 drift, 0 failed`); `cargo test --manifest-path cli/Cargo.toml`; `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`; `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; `mochiflow lint --spec review-fix-budget`; `mochiflow doctor` exit 0<br>final verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` | dogfood sync and default verification |
