# Batch bounded in-scope fixes: hold locally, review once before push/accept

## Background and Design Rationale

`reference/risk.md`'s "verdict freshness" rule is correct in principle: for
`risk >= elevated`, any code change after the recorded reviewer verdict makes it
stale, and a fresh `change-reviewer` run is required before the next push
(`update.md` step 4) or `mochiflow accept` (`open.md` step 3e QA-`FAIL`
rework). The rule is triggered at "every code change" granularity instead of at
the granularity of the boundary it actually protects. `commands/update.md` runs
apply-fix → commit → review (if stale) → push → PR metadata as one atomic
procedure per invocation, and its natural-language triggers (`修正依頼`, `PR
feedback`, `PRを直して`) re-run that whole procedure on every user utterance.
N separate small PR-feedback items in one sitting become N full reviewer runs
and N pushes to the same open PR. The same undocumented gap exists in the
window after `build` finishes its task loop but before `open` runs: there is no
defined lightweight path for "one more small in-scope touch-up" there.

Key design decisions (see `pitch.md` for the full discussion and rejected
alternatives):

- The **in-scope vs. out-of-scope judgment is not new** — `build.md` already
  stops for "an out-of-scope change or a new design decision" and routes to
  `plan`; `update.md` already routes unrelated feedback to a new spec. This
  spec states that judgment once (in `reference/risk.md`) and has
  `build.md` / `open.md` / `update.md` reference it, rather than redefining it
  three times.
- **In-scope requests get new handling: apply, commit, hold.** No push, no
  `mochiflow accept`, no reviewer re-run at that moment — regardless of whether
  the request lands before or after `open`.
- **One explicit finalize signal, separate from the existing soft hints.** The
  explicit `mochiflow-update` command already exists distinctly from the
  natural-language hints in router terms (`router.md`'s "a `mochiflow-<verb>`
  token is an explicit command; every other entry is a natural-language hint");
  this spec reuses that existing distinction as the hold/finalize boundary
  instead of inventing new trigger machinery.
- **`Reviewed through: <sha>` makes freshness mechanically traceable** without
  any Rust code change: it is a textual convention recorded next to `Verdict:`
  in `design.md ## Review Results`, computable against `HEAD` with a plain
  `git log` range from the recorded sha.
- The reviewer's diff input is unchanged (still the full diff from git, never a
  compact report or conversation history) — only how often it runs changes.

## User Story

As someone driving a `risk >= elevated` mochiflow spec — through PR feedback
after `open`, or through post-build touch-ups before `open` — I want the
mandatory reviewer to run once for a batch of small fixes I make in a row,
instead of once per fix, so that repeated small feedback does not multiply
reviewer and push overhead while every diff that is actually pushed or accepted
still gets a fresh review.

## Scope

- In:
  - `engine/reference/risk.md`: state the shared in-scope/out-of-scope judgment
    once; add the hold-then-finalize boundary and the `Reviewed through: <sha>`
    recording convention to the "Verdict freshness" paragraph.
  - `engine/templates/spec/design.md`: extend the `## Review Results` comment to
    mention recording `Reviewed through: <sha>` on its own line below
    `Verdict:`.
  - `engine/commands/update.md`: split the existing natural-language triggers
    (hold-only: apply, commit locally, report status, no push) from a distinct
    explicit finalize signal (`mochiflow-update`, a match against its own
    `{slug} <pattern>` trigger patterns, or an unambiguous "that's everything /
    push it / update the PR" statement: review-if-stale once, then push, then
    PR metadata) — while preserving every phrase pinned by the existing
    `pr_feedback_routes_to_update_without_restore` conformance test. The
    frontmatter `description` ("update re-verifies, pushes, updates PR
    metadata...") is reworded to the same hold-by-default / finalize-pushes
    contract so it does not contradict the procedure it summarizes.
  - `engine/commands/open.md`: move the fresh-review trigger to the
    accept-transition step 6 (immediately before `mochiflow accept` runs) as an
    unconditional precondition for `risk >= elevated` — "a commit that changes
    code exists beyond the recorded `Reviewed through` sha" (still exactly one
    re-review before accept) — so it fires whether the QA round-trip (step 3)
    ran or was skipped entirely, and a held post-build bounded fix is caught by
    the same gate as a QA-`FAIL` rework fix. A non-code commit (e.g. the
    optional `docs(context)` commit) does not trigger it.
  - `engine/commands/build.md`: document the post-all-tasks/pre-`open` window —
    an in-scope request uses the same apply-commit-hold handling; an
    out-of-scope request still routes to `plan`, unchanged.
  - `cli/crates/mochiflow-cli/tests/conformance.rs`: new tests pinning the
    hold-vs-finalize contract in `update.md`, the generalized freshness trigger
    in `open.md`, the post-completion window in `build.md`, and the shared
    judgment + `Reviewed through` convention in `risk.md`. Existing pinned
    tests (`pr_feedback_routes_to_update_without_restore`,
    `build_ends_at_approved_without_pr_or_move`,
    `open_orders_acceptance_fold_accept_pr_gate`, and others) continue to pass
    unmodified.
  - Engine sync per `constitution.md ## Dogfood` (`mochiflow freeze` →
    `mochiflow upgrade --source engine` → `mochiflow adapter generate --check`)
    and the surface `default` verify + `mochiflow lint` + `mochiflow doctor`.
  - `engine/router.md` (two spots only): (1) the "PR Feedback Loop Routing"
    sentence describing `update` currently reads "applies bounded inline
    fixes, re-verifies, pushes, and updates the PR body when needed"; (2) the
    Verb Delegation table's `update` row reads "applies a bounded inline code
    fix, then re-verifies, pushes, and updates PR metadata." Both assert an
    unconditional immediate push that contradicts `update.md`'s new
    hold-by-default behavior. Reword both so neither asserts an immediate
    push, while keeping the routing decision itself (PR feedback →
    `update.md`) and the pinned substring "applies bounded inline fixes"
    unchanged. These are the only router.md edits in scope; the trigger/routing
    *decision logic* stays untouched (that broader redesign is tracked
    separately as the `trigger-routing-redesign` backlog seed).
- Out:
  - `build.md`'s task-structure mutation rule
    (`prevent-build-phase-spec-mutation`) — task additions/removals/reordering
    still require returning to `plan`.
  - The two delivery approval gates and review's status as a quality assist,
    never a gate.
  - `agents/change-reviewer.md`'s S0-S4 review contract and diff scope — the
    reviewer still always reads the full diff from git; no incremental,
    since-last-`sha` scoping in this pass.
  - Any Rust behavior change (`accept.rs`, the reviewer transport, or any CLI
    flag). `Reviewed through: <sha>` is a textual convention only.
  - Any further edit to `router.md` beyond the one line above, and the general
    trigger/routing overhaul tracked separately as `trigger-routing-redesign`.
  - Retrofitting specs authored before this convention or archived under
    `_done/`.

## Edge Cases

- A single one-off fix request now needs an explicit finalize follow-up before
  it is pushed (one extra turn versus today's immediate push). This is an
  intentional, discussed tradeoff: the alternative (auto-push on the first fix,
  batch-hold afterward) would need the agent to guess whether a fix is "the
  last one," which it cannot do reliably.
- `mochiflow-update` is invoked with no held commits and no new fix in the same
  message: report that the PR is already up to date; no review or push runs.
- A finalize signal fires and the fresh review returns `fail`: do not push;
  report findings and fix, re-verify, and re-commit before re-attempting
  finalize — the reviewer gate is never bypassed on a bad verdict.
- A PR-body-only correction with no held code commits: no reviewer re-run is
  needed even at finalize, matching the existing "PR-body-only corrections skip
  the code loop" rule.
- Post-build/pre-`open` window: an in-scope touch-up is applied and held, then
  `open` runs immediately after. `open`'s accept-gate freshness check (AC-06,
  at step 6, independent of whether the QA round-trip ran at all) must catch
  the held, unreviewed commit and re-review before accept — the same mechanism
  that already covers QA-`FAIL` rework.
- `open` step (c) may add a `docs(context)` commit after the fold/context-check
  and before the accept-gate check in step 6. That commit does not, by itself,
  make a prior verdict stale — AC-06's trigger is scoped to commits that change
  code, matching AC-05's existing PR-body-only carve-out for `update`.
- `risk = standard` specs: there is no mandatory reviewer at all
  (`reference/risk.md ## Consequences`), so hold/finalize still separates
  commit timing from push timing for a consistent experience, but there is
  never a reviewer run to gate on.
- Ambiguous intent ("OK", a short acknowledgement) after a hold-mode fix: the
  agent must not guess finalize from an ambiguous reply; it holds by default
  and asks one clarifying question only when the reply could plausibly mean
  either hold or finalize.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL state, in exactly one place (`reference/risk.md`), the
  judgment that distinguishes an in-scope code change (no task-structure
  change, no new AC, no new design decision) from an out-of-scope change, and
  `build.md`, `open.md`, and `update.md` SHALL reference that shared judgment
  rather than each defining it separately.
- AC-02: WHEN a requested code change is judged in-scope, whether it occurs
  after `build` completes and before `open` runs, or as PR feedback handled by
  `update`, THE SYSTEM SHALL apply and commit the change locally and SHALL NOT
  automatically push, run `mochiflow accept`, or re-run the mandatory reviewer
  as part of that same step.
- AC-03: THE SYSTEM SHALL record `Reviewed through: <sha>` on its own line,
  directly below `Verdict:`, in `design.md ## Review Results` on every
  mandatory reviewer run (build's own risk-cadence run, and any later
  re-review during `open` or `update`), so the commits held since the last
  recorded review are computable from git alone and `accept.rs`'s existing
  exact-match `Verdict:` parsing is never given a combined line to misread.
- AC-04: `update.md` SHALL treat its existing bare natural-language triggers
  (`修正依頼`, `PR feedback`, `PRを直して`, and equivalent hold-only requests
  with no slug qualifier) as hold-only, and SHALL require a distinct explicit
  finalize signal — the explicit `mochiflow-update` command, a match against
  any of `update.md`'s own `{slug} <pattern>` trigger patterns, or an
  unambiguous completion statement — before running the review-if-stale, push,
  and PR-metadata steps. `update.md`'s own frontmatter `description` and its
  Presentation-section reporting line SHALL reflect this hold-by-default /
  finalize-pushes contract and SHALL NOT assert that a bounded inline fix is
  pushed unconditionally.
- AC-05: WHEN update's finalize signal fires and one or more commits that
  change code exist beyond the `Reviewed through` sha recorded for
  `risk >= elevated`, THE SYSTEM SHALL re-run `change-reviewer` exactly once on
  the full diff from git, record the fresh verdict and updated `Reviewed
  through: <sha>`, and only then push; WHEN no such commit exists (for example
  a PR-body-only correction), THE SYSTEM SHALL NOT re-run the reviewer.
- AC-06: `open.md`'s accept-transition step (step 6, immediately before running
  `mochiflow accept`) SHALL trigger a fresh `change-reviewer` run for
  `risk >= elevated` WHEN any commit that changes code exists beyond the
  recorded `Reviewed through` sha, whether that commit came from the QA-`FAIL`
  rework loop (step 3e) or from a held post-build bounded fix, and SHALL run
  the reviewer at most once for that accumulated set. This check fires at the
  accept gate itself, so it still applies when the QA round-trip (step 3) is
  skipped entirely because the spec has no human-operated/visual QA items; a
  non-code commit (for example the `docs(context)` commit from step (c)) does
  not by itself trigger it.
- AC-07: `build.md` SHALL document that, after all tasks (or the single
  logical-unit commit for taskless/micro specs) complete and before `open` runs,
  an in-scope request is applied, committed, and held using the same handling
  as an in-scope PR-feedback fix, and an out-of-scope request continues to
  route back to `plan` for re-approval exactly as today.
- AC-08: THE edited engine documents SHALL NOT change the two delivery approval
  gates, SHALL NOT alter `build.md`'s existing task-structure mutation rule
  (`prevent-build-phase-spec-mutation`), and SHALL NOT change
  `agents/change-reviewer.md`'s S0-S4 review contract or diff scope.
- AC-09: THE existing conformance tests that pin update/open/build wording
  (including `pr_feedback_routes_to_update_without_restore`,
  `build_ends_at_approved_without_pr_or_move`, and
  `open_orders_acceptance_fold_accept_pr_gate`) SHALL continue to pass
  unmodified, and new conformance tests SHALL pin the hold-vs-finalize contract
  in `update.md`, the generalized freshness trigger in `open.md`, and the
  post-completion bounded-fix window in `build.md`.
- AC-10: THE edited engine SHALL be synced via the `constitution.md ## Dogfood`
  sequence (`mochiflow freeze` → `mochiflow upgrade --source engine` →
  `mochiflow adapter generate --check`) and SHALL pass the surface `default`
  verification, `mochiflow lint`, and `mochiflow doctor`.
- AC-11: `engine/router.md`'s "PR Feedback Loop Routing" sentence and its Verb
  Delegation table row for `update` SHALL both remain consistent with
  `update.md`'s hold/finalize split — neither SHALL assert that a bounded
  inline fix is pushed immediately — while the trigger/routing decision logic,
  and every other router.md line, stays unchanged.

## QA Scenarios

> Dimensions per `reference/risk.md ## QA attack coverage`. This is a
> docs/workflow-contract change with no runtime data surface; QA below covers
> what conformance tests cannot on their own — live conversational behavior,
> ambiguous intent, cross-session recovery, and full-suite regression. `Scope`
> is `cli` throughout (the project's only surface).

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | Human-operated | Drive an elevated-risk spec through `update`: send two separate soft feedback messages, then an explicit finalize ("push して"). | Both fixes commit locally without pushing after each message; only the finalize message triggers one reviewer run and one push covering both commits. |
| QA-02 | QA-UX | cli | Human-operated | After each hold-mode fix, read the agent's status line. | The status line states the change is applied and held locally, not yet pushed/reviewed, with no interruptive choice card after every fix. |
| QA-03 | QA-ABUSE | cli | Human-operated | Send an ambiguous reply that could mean either "one more fix" or "that's everything" (e.g. a bare "OK"); separately, send an unambiguous finalize statement. | The ambiguous reply does not trigger a premature push (the agent holds or asks, per the Edge Cases rule); the unambiguous statement is recognized without re-prompting. |
| QA-04 | QA-DATA | cli | Automated | After two held commits and one finalize/push cycle, inspect `design.md ## Review Results` and `git log`. | Exactly one fresh `Verdict:` + `Reviewed through: <sha>` pair is recorded per finalize; the recorded sha matches the pushed `HEAD`; no held commit is dropped. |
| QA-05 | QA-COMPAT | cli | Automated | Run the full `cargo test` suite after the edits, including `pr_feedback_routes_to_update_without_restore`, `build_ends_at_approved_without_pr_or_move`, and `open_orders_acceptance_fold_accept_pr_gate`. | All pre-existing conformance tests pass unmodified; new tests pass alongside them. |
| QA-06 | QA-RESIL | cli | Human-operated | Apply one in-scope held fix, end the session, resume in a new session before finalizing. | The new session recovers the held state from git (`Reviewed through: <sha>` vs. `HEAD`) alone, with no reliance on conversation memory, and can still finalize correctly. |
| QA-07 | QA-REG | cli | Automated | Run the dogfood sync (`mochiflow freeze` → `mochiflow upgrade --source engine` → `mochiflow adapter generate --check`) then the surface `default` verify, `mochiflow lint`, and `mochiflow doctor`. | Sync and verification pass; no drift in the manifest or generated adapters; `doctor` reports no new issue. |
| QA-08 | QA-FUNC | cli | Human-operated | Complete `build` on an elevated spec whose QA Scenarios contain only `Automated` rows (no `Human-operated`/`Visual` item); apply one in-scope post-build touch-up before running `open`; run `open` through to the accept gate. | `open` re-runs `change-reviewer` once before `mochiflow accept`, even though the QA round-trip (step 3) was skipped entirely because there was no human-operated/visual item; the fresh verdict and updated `Reviewed through: <sha>` are recorded before accept proceeds. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded; the engine re-freeze leaves
  `freeze --check` passing.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | AI-observed | QA-05 | `engine/reference/risk.md` | PASS | `risk_defines_shared_bounded_fix_judgment_and_reviewed_through` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed `engine/reference/risk.md` | shared judgment stated once |
| AC-02 | cli | AI-observed | QA-01, QA-06 | `engine/commands/update.md`, `engine/commands/build.md` | PASS | `update_holds_fixes_until_explicit_finalize_signal` and `build_documents_post_completion_bounded_fix_window` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed command docs | apply-commit-hold, no auto push/accept/review |
| AC-03 | cli | automated | QA-04, QA-05 | `engine/reference/risk.md`, `engine/templates/spec/design.md`, `engine/commands/build.md` | PASS | `risk_defines_shared_bounded_fix_judgment_and_reviewed_through` and `build_commit_cadence_is_task_based_not_risk_based` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed design template and build command wording | `Reviewed through: <sha>` on its own line below `Verdict:` |
| AC-04 | cli | automated | QA-01, QA-05 | `engine/commands/update.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `update_holds_fixes_until_explicit_finalize_signal`, `router_update_summary_matches_hold_by_default`, and `pr_feedback_routes_to_update_without_restore` in `cargo test --manifest-path cli/Cargo.toml` | hold vs. explicit finalize split |
| AC-05 | cli | AI-observed | QA-01, QA-04 | `engine/commands/update.md` | PASS | `update_holds_fixes_until_explicit_finalize_signal` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed `engine/commands/update.md` finalize path | review-if-stale once (code commits only), then push |
| AC-06 | cli | AI-observed | QA-08 | `engine/commands/open.md` | PASS | `open_generalizes_freshness_trigger_to_reviewed_through` and `open_orders_acceptance_fold_accept_pr_gate` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed `engine/commands/open.md` step 6 | accept-gate (step 6) trigger, fires even with no QA round-trip; code commits only |
| AC-07 | cli | automated | QA-05 | `engine/commands/build.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `build_documents_post_completion_bounded_fix_window` and `build_ends_at_approved_without_pr_or_move` in `cargo test --manifest-path cli/Cargo.toml` | post-completion window + unchanged plan-routing |
| AC-08 | cli | automated | QA-05 | `engine/reference/workflow.md`, `engine/agents/change-reviewer.md`, `engine/commands/build.md` | PASS | `open_orders_acceptance_fold_accept_pr_gate`, `build_documents_post_completion_bounded_fix_window`, and `canonical_reviewers_grounded_adversary_contract_is_pinned` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed no edits to `engine/agents/change-reviewer.md` | task-structure rule, two delivery gates, and reviewer contract preserved |
| AC-09 | cli | automated | QA-05 | `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` PASS, including pre-existing routing/order guards and new conformance tests | full regression + new pinned tests |
| AC-10 | cli | automated | QA-07 | `engine/MANIFEST.json`, `cli/` | PASS | `mochiflow freeze && mochiflow upgrade --source engine && mochiflow adapter generate --check` PASS; `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` PASS; `mochiflow lint --spec review-cadence-batching` PASS; `mochiflow doctor` 0 fail | dogfood sync + default verify + lint + doctor |
| AC-11 | cli | automated | QA-05 | `engine/router.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `router_update_summary_matches_hold_by_default` and `pr_feedback_routes_to_update_without_restore` in `cargo test --manifest-path cli/Cargo.toml`; AI-observed two router summary edits only | PR-feedback-routing wording aligned with hold/finalize |
