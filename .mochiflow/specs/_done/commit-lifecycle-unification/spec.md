# Unify commit timing across discuss/plan/build/ship on a single branch

## Background and Design Rationale

Origin: `_backlog/commit-lifecycle-unification.md` (discuss handoff, 2026-06-23).

Commit timing across the four spec-lane phases is currently inconsistent:
discuss writes only a `_backlog` file (uncommitted persistence), plan writes spec
files but does not commit them, and the feature branch is created at build start.
This creates volatility (session loss between discuss and build loses all
artifacts) and unclear ownership of the first commit on the feature branch.

Key decisions from discuss:

- Branch creation moves from build to discuss (at agreement). All four phases
  commit to the same `{prefix}/{slug}` branch; nothing touches main directly.
- `pitch.md` is introduced as a new durable discuss artifact. It replaces the
  `_backlog` ready-for-plan handoff file and persists through the entire lifecycle
  to `_done/{slug}/`.
- `status: draft` ownership moves from plan to discuss. discuss creates
  `spec.yaml (draft)` + `pitch.md` as its commit content.
- The `_backlog` ready-for-plan handoff format (`maturity: ready-for-plan`) is
  abolished. Raw seeds (`maturity: seed`) remain as discuss input.
- Commit messages for discuss/plan use `docs` type (Conventional Commits) with
  `Spec:` trailer. No custom `discuss`/`plan` type.
- lint accepts `status: draft` with only `spec.yaml` + `pitch.md` (no `spec.md`).
- `develop-branch-workflow` is not adopted; all commits go through PR.

Risk / integration rationale: `elevated` with `integration: workflow` because
the change rewires lifecycle ownership across router, phase commands, workflow
reference, git rules, lint validation, and conformance guards. It modifies the
frozen engine surface (requiring manifest regeneration), changes CLI lint
validation logic, and relocates a state-transition ownership (`draft` from plan
to discuss). However, no data migration, no external contract break, no
auth/security impact.

## User Story

As an AI coding agent, I want every spec-lane phase to commit its artifacts to
the same feature branch so that session loss never destroys discuss/plan output
and the git history clearly shows each phase's contribution.

## Scope

- In: engine source docs (`engine/router.md`, `engine/commands/discuss.md`,
  `engine/commands/plan.md`, `engine/commands/build.md`,
  `engine/reference/authoring.md`, `engine/reference/git.md`,
  `engine/reference/workflow.md`, `engine/templates/handoff/build-session-prompt.md`),
  new template (`engine/templates/spec/pitch.md`), template removal
  (`engine/templates/backlog/discuss-handoff.md`), CLI lint code
  (`cli/crates/mochiflow-core/src/lint.rs`), backlog validation code
  (`cli/crates/mochiflow-core/src/backlog.rs`), conformance tests
  (`cli/crates/mochiflow-cli/tests/conformance.rs`), `mochiflow freeze`,
  generated index check, adapter check, vendored engine sync.
- Out: CLI branch-management code (branching is git operations guided by docs).
  `commands/ship.md` procedure changes beyond keeping its single-branch close-out
  references consistent. PR workflow changes. `develop-branch-workflow` adoption.

## Edge Cases

- discuss without a pre-existing `_backlog` seed: branch + pitch.md + spec.yaml
  only (no deletion step).
- discuss that does not reach agreement: no branch created; raw seed remains if
  one existed.
- plan resumed in a new session: switch to `{prefix}/{slug}` before starting.
- build on a branch that does not exist: error-stop (discuss/plan were not run).
- Multiple specs being planned concurrently: each lives on its own branch;
  switching is required between them.
- A stale `maturity: ready-for-plan` backlog file created by an older engine
  should no longer route directly to plan after this change; it must either be
  treated as legacy input for discuss or converted by a human decision.
- A draft spec may be either discuss-created pitch-only form or plan-expanded
  form. Lint must allow pitch-only draft regardless of `risk` / `integration`,
  but once `spec.md` exists it must enforce the normal plan-time checks,
  including required `design.md`.

## Acceptance Criteria (EARS)

- AC-01: WHEN discuss reaches agreement, THE SYSTEM SHALL create branch `{prefix}/{slug}` from `origin/{base_branch}`, write lint-valid `spec.yaml (status: draft)` and `pitch.md`, run `mochiflow lint --spec {slug}` successfully in pitch-only draft mode, commit those artifacts, and delete `_backlog/{slug}.md` if it exists.
- AC-02: THE SYSTEM SHALL define `pitch.md` template at `engine/templates/spec/pitch.md` with sections: Problem, Appetite, Solution, Rabbit Holes, No-gos, Alternatives Considered, Open Questions.
- AC-03: WHEN plan completes approval, THE SYSTEM SHALL commit `spec.md` (+ `design.md` / `tasks.md`) and `spec.yaml (status: approved)` on the existing `{prefix}/{slug}` branch.
- AC-04: WHEN build starts, THE SYSTEM SHALL verify branch `{prefix}/{slug}` exists, switch to it, and error-stop if it does not exist.
- AC-05: THE SYSTEM SHALL update `reference/workflow.md` to state that `status: draft` is set by discuss (not plan).
- AC-06: THE SYSTEM SHALL update `reference/git.md` with a commit lifecycle table showing discuss/plan/build/ship commit sequence on a single branch.
- AC-07: THE SYSTEM SHALL update lint logic to accept a pitch-only `status: draft` spec with only `spec.yaml` + `pitch.md` (no `spec.md` / `design.md` required), while still enforcing normal `spec.md` / `design.md` / `tasks.md` checks for draft specs once `spec.md` exists.
- AC-08: THE SYSTEM SHALL remove `engine/templates/backlog/discuss-handoff.md`.
- AC-09: THE SYSTEM SHALL update `commands/discuss.md` to allow writes to `{specs_dir}/{slug}/**`, add branch creation and commit procedure, and remove `_backlog` handoff output.
- AC-10: THE SYSTEM SHALL update `commands/plan.md` to read `pitch.md` as input, add commit procedure at approval, and remove `_backlog` deletion step.
- AC-11: THE SYSTEM SHALL update `commands/build.md` to replace branch creation with branch existence check + switch.
- AC-12: THE SYSTEM SHALL update `engine/router.md` so `{slug} plan` no longer resolves a `maturity: ready-for-plan` backlog handoff and instead relies on an existing `{specs_dir}/{slug}/` draft spec created by discuss.
- AC-13: THE SYSTEM SHALL update `reference/authoring.md` and `templates/handoff/build-session-prompt.md` so durable artifacts and new-session handoff include `pitch.md`.
- AC-14: THE SYSTEM SHALL update backlog validation so `_backlog/{slug}.md` accepts only `maturity: seed` and rejects `maturity: ready-for-plan`.
- AC-15: THE SYSTEM SHALL update conformance tests that pin ready-for-plan handoff behavior to pin the new pitch/draft lifecycle, seed-only backlog validation, and lint behavior instead.
- AC-16: THE SYSTEM SHALL regenerate `engine/MANIFEST.json` via `mochiflow freeze` and sync vendored engine via `mochiflow upgrade --source engine`.
- AC-17: THE SYSTEM SHALL pass `cargo test`, `mochiflow lint --spec commit-lifecycle-unification`, `mochiflow freeze --check`, `mochiflow doctor`, `mochiflow adapter generate --check`, and `mochiflow index --check` after all changes.

## QA Scenarios

| # | Scope | Operation | Expected |
| --- | --- | --- | --- |
| QA-01 | cli | Run `cargo test --manifest-path cli/Cargo.toml` | All tests pass |
| QA-02 | cli | Run `mochiflow freeze --check` | Exit 0 |
| QA-03 | cli | Run `mochiflow doctor` | Exit 0 |
| QA-04 | cli | Run `mochiflow lint` on a spec dir with only `spec.yaml (draft)` + `pitch.md` | Lint passes |
| QA-05 | cli | Run `mochiflow lint` on a spec dir with `spec.yaml (approved)` but no `spec.md` | Lint fails |
| QA-06 | cli | Run `mochiflow lint --spec commit-lifecycle-unification` | Exit 0 |
| QA-07 | cli | Run `mochiflow adapter generate --check` and `mochiflow index --check` | Both exit 0 |
| QA-08 | cli | Run `mochiflow backlog validate` on `maturity: ready-for-plan` | Validation fails with seed-only maturity guidance |
| QA-09 | cli | Run `mochiflow lint` on `status: draft` with `spec.md` and `risk: elevated` but no `design.md` | Lint fails |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `人間確認済み`, or `対象外（<reason>）`).
- Verification commands and results are recorded.
- Independent reviewer verdict recorded (risk: elevated).

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | content check: discuss.md contains branch creation + pitch-only lint + commit procedure | `engine/commands/discuss.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-02 | cli | automated | file existence + section check: `engine/templates/spec/pitch.md` | `engine/templates/spec/pitch.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-03 | cli | automated | content check: plan.md contains commit at approval | `engine/commands/plan.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-04 | cli | automated | content check: build.md has branch existence check, no creation | `engine/commands/build.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-05 | cli | automated | content check: workflow.md states draft set by discuss and no ready-for-plan lifecycle remains | `engine/reference/workflow.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-06 | cli | automated | content check: git.md has commit lifecycle table | `engine/reference/git.md` | PASS | inline content review | |
| AC-07 | cli | automated | `mochiflow lint` on draft+pitch-only dir passes; draft+spec.md missing required design fails; approved-without-spec.md fails | `cli/crates/mochiflow-core/src/lint.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-08 | cli | automated | file absence: `engine/templates/backlog/discuss-handoff.md` | removal | PASS | `cargo run --manifest-path cli/Cargo.toml -p mochiflow-cli -- freeze --check` | |
| AC-09 | cli | automated | content check: discuss.md allowed_writes includes spec dir, has commit procedure | `engine/commands/discuss.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-10 | cli | automated | content check: plan.md reads pitch.md, has commit step, no _backlog deletion | `engine/commands/plan.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-11 | cli | automated | content check: build.md step 2 has existence check, no branch creation | `engine/commands/build.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-12 | cli | automated | conformance prose guard + content check: router removes ready-for-plan plan exception | `engine/router.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-13 | cli | automated | content check: authoring durable artifacts and build handoff include `pitch.md` | `engine/reference/authoring.md`, `engine/templates/handoff/build-session-prompt.md` | PASS | inline content review; `cargo test --manifest-path cli/Cargo.toml` | |
| AC-14 | cli | automated | backlog validator accepts only raw seeds and rejects ready-for-plan handoffs | `cli/crates/mochiflow-core/src/backlog.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-15 | cli | automated | conformance tests assert new pitch/draft lifecycle, seed-only backlog validation, and lint cases | `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` | |
| AC-16 | cli | automated | `mochiflow freeze --check` exit 0, `mochiflow doctor` exit 0 | `engine/MANIFEST.json`, `.mochiflow/engine/**` | PASS | `cargo run --manifest-path cli/Cargo.toml -p mochiflow-cli -- freeze --check`; `cargo run --manifest-path cli/Cargo.toml -p mochiflow-cli -- doctor` | |
| AC-17 | cli | automated | final command suite exits 0 | CLI tests + engine/index/adapter checks | PASS | `cargo test --manifest-path cli/Cargo.toml`; lint/freeze/doctor/adapter/index checks | |
