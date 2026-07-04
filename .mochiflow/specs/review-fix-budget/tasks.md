# Add review budget loops with optional automatic fixes — Tasks

Implementation Summary: Extend the existing review command family so result-only review remains read-only, while `review fix [N]` runs bounded main-agent fix rounds with fresh independent reviewer cycles.
risk: elevated
Critical Stop Conditions:
- Do not add a second public verb or a new write-capable reviewer/worker role.
- Do not change the two delivery approval gates or mandatory risk-cadence review requirements.
- Do not pass prior review findings, verdicts, summaries, or conversation history into later reviewer cycles.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing
- Engine edits target repo-root `engine/...` only; after any engine edit, run `mochiflow freeze` before final verification and never edit `.mochiflow/engine/` directly.

## Tasks

- [ ] T-001 [AC-01, AC-02, AC-03, AC-08] Define review command grammar and result-only vs fix-mode behavior
  - Depends on: none
  - Files:
    - `engine/commands/review.md`
    - `engine/router.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: `review.md` documents `{slug} review` as one read-only report-only pass; documents `{slug} review fix`, `{slug} review fix 1`, `{slug} review fix 2`, and `{slug} review fix 3` as bounded automatic fix-round forms; states the number after `fix` is the maximum fix-round count and the loop ends after the final fix round; rejects `{slug} review 2`, `fix 0`, and `fix 4+` before any review runs. `router.md` recognizes the expanded trigger pattern without adding another verb. Conformance tests pin these command forms, invalid forms, and the unchanged report-only behavior of plain `{slug} review`. Run `mochiflow freeze` and include the regenerated `engine/MANIFEST.json`.
  - Stop: if the implementation requires a second public verb (`revise`/`refine`) or a new Rust CLI subcommand, stop and return to plan.
- [ ] T-002 [AC-04, AC-05, AC-06, AC-09] Define fresh independent review-loop boundaries and recovery ledger
  - Depends on: T-001
  - Files:
    - `engine/reference/risk.md`
    - `engine/agents/plan-auditor.md`
    - `engine/agents/change-reviewer.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: `risk.md` states the review-fix loop boundaries: reviewers remain read-only; the main agent applies fixes; automatic fixes are limited to no task-structure change, no new AC, no new design decision, no spec split, no unrelated work, and no unresolved repeated finding; verification failure stops the loop. It also defines the local review-fix recovery ledger under `{install_dir}/state/{slug}/` for main-agent-only recovery, with requested fix rounds, completed fix rounds, current phase/profile, touched files, verification evidence, and stop reason; it states ledger contents and prior findings are not passed to later reviewers. `plan-auditor.md` and `change-reviewer.md` state that later review cycles may receive cycle-local changed files/diff as focus input, but must not receive previous findings, previous verdicts, previous reviewer summaries, ledger contents, or conversation history. Conformance tests pin fresh-independent review input, the recovery ledger contract, and no-worker/no-write reviewer boundaries. Run `mochiflow freeze` and include the regenerated `engine/MANIFEST.json`.
  - Stop: if preserving reviewer independence conflicts with the remediation guidance needed by the main agent, keep the remediation guidance on the main-agent side and do not pass it to later reviewers.
- [ ] T-003 [AC-06, AC-07, AC-09] Update lifecycle command choice cards and phase-specific fix discipline
  - Depends on: T-001, T-002
  - Files:
    - `engine/commands/plan.md`
    - `engine/commands/build.md`
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/router.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: Plan/build/open/update presentation sections distinguish "Review results" from "Review and fix" while mapping both to the `review` command family. Plan-time `review fix` uses `plan-auditor` and limits edits to spec artifacts. Code-present `review fix` uses `change-reviewer` and follows the current build/open/update bounded-fix discipline, including update's hold/finalize behavior and build's no-checkbox/no-`Task:` post-completion touch-up rule. Each fix round updates the local recovery ledger, and resume guidance says to recover from the ledger plus repository state rather than hidden conversation memory. Existing mandatory risk-cadence review requirements remain unchanged. Conformance tests pin the choice-card mapping and that review-fix does not add a new delivery gate. Run `mochiflow freeze` and include the regenerated `engine/MANIFEST.json`.
  - Stop: if adding the choice-card labels would make review mandatory or would change approve-to-build / approve-PR semantics, stop and return to plan.
- [ ] T-004 [AC-07, AC-10] Update user-facing documentation and adapter-facing summaries
  - Depends on: T-001, T-002, T-003
  - Files:
    - `README.md`
    - `README.ja.md`
    - `docs/concepts.md`
    - `docs/configuration.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: User docs describe plain `review` as result-only and `review fix [1-3]` as bounded automatic fixing without introducing another public verb or severity/gate flags. Documentation keeps review framed as a quality assist, not a delivery approval gate. Conformance tests pin any engine-doc strings needed for adapter generation or public command summaries.
  - Stop: if docs would need to document unsupported numeric forms like `review 2`, keep the docs focused on supported forms only.
- [ ] T-005 [AC-11] Dogfood sync and final verification
  - Depends on: T-001, T-002, T-003, T-004
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - `.mochiflow/specs/review-fix-budget/spec.md`
    - `.mochiflow/specs/review-fix-budget/design.md`
    - `.mochiflow/specs/review-fix-budget/tasks.md`
  - Done: Run `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check`. Run `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, `cargo run --manifest-path cli/Cargo.toml -- freeze --check`, `mochiflow lint --spec review-fix-budget`, and `mochiflow doctor`. Update the AC Matrix with implementation paths, results, and evidence. Record mandatory `change-reviewer` result in `design.md ## Review Results` before build completion because the spec is `risk: elevated`.
  - Stop: if `adapter generate --check` reports drift in generated adapter files, inspect whether source templates need planned edits; do not hand-edit generated adapter outputs.
