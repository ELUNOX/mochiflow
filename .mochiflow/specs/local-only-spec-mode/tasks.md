# Support local-only MochiFlow specs when .mochiflow is gitignored — Tasks

Implementation Summary: Add explicit tracked/local spec persistence handling for accept and PR handoff, with docs and regression fixtures.
risk: elevated
Critical Stop Conditions:
- Do not weaken tracked-mode close-out commit or `Spec:` trailer preflight.
- Do not suggest force-adding ignored `.mochiflow/` artifacts in local mode.
- Stop and return to plan if mode detection cannot be based on Git ignore state.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01] Add shared spec persistence mode detection
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/spec_mode.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
    - `cli/crates/mochiflow-core/src/accept.rs`
    - `cli/crates/mochiflow-core/src/pr.rs`
  - Done: A shared helper classifies tracked versus local spec persistence from Git ignore behavior for the concrete spec path, exposes reason text for CLI output, and both `accept` and `pr` call the helper instead of implementing separate checks.
  - Stop: Detector behavior would require a config override or cannot distinguish ignored spec artifacts from ordinary runtime state.
- [ ] T-002 [AC-02, AC-03, AC-07] Split `accept` close-out behavior by persistence mode
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/accept.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Local mode runs final verification, readiness checks, spec mutation, and lint, then exits successfully without staging or committing ignored spec/ADR artifacts while printing the local-mode skip reason; tracked mode keeps the existing staged close-out commit and trailer behavior. Shared structures touched in `accept.rs` remain mode-explicit so later `pr` changes do not infer behavior from side effects.
  - Stop: Local mode can pass while unrelated tracked files are dirty, or tracked mode no longer creates the close-out commit.
- [ ] T-003 [AC-04, AC-05, AC-06, AC-07] Split `pr` preflight by persistence mode
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-core/src/accept.rs`
    - `cli/crates/mochiflow-cli/tests/pr.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Local mode no longer requires committed accepted spec/trailer and instead validates clean tracked tree, source branch, base/head inequality, head ahead of base, local accepted state, complete evidence, and required review result; `--dry-run` behavior is explicit and tested; tracked mode still requires the committed accepted spec and trailer. Shared validation between `accept.rs` and `pr.rs` is factored or called directly so evidence rules cannot drift.
  - Stop: `pr` would push or dispatch before local-mode evidence validation passes.
- [ ] T-004 [AC-10] Preserve local-mode delivery derivation after manual merge
  - Depends on: T-001, T-003
  - Files:
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-core/src/status.rs`
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/router.md`
    - `engine/commands/close.md`
    - `engine/reference/git.md`
  - Done: Provider-backed local mode can still use provider merge state, and provider-none/manual local mode derives delivered/local-cleanup-pending when the source branch tip is reachable from `origin/{base_branch}` even though no `Spec:` trailer exists; tracked-mode provider/trailer derivation remains unchanged. Router, status, index, and close guidance describe the branch-tip limitation if the branch is deleted before cleanup, and conformance pins the router wording.
  - Stop: Delivery derivation would mark a local-mode spec Done without provider merge state, trailer reachability, or branch-tip reachability.
- [ ] T-005 [AC-08, AC-09] Update engine guidance, PR body template, and user docs
  - Depends on: T-002, T-003
  - Files:
    - `engine/commands/open.md`
    - `engine/reference/git.md`
    - `engine/reference/workflow.md`
    - `engine/templates/delivery/pr-description.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `docs/configuration.md`
    - `docs/concepts.md`
  - Done: Open guidance describes tracked and local mode without force-add advice; PR body guidance requires verification evidence, review result, and durable decision summary for local mode; docs explain mode detection, constraints, recommended usage, and local-to-tracked migration; conformance pins short stable phrases for local-mode PR body evidence, review result, durable decision summary, and no force-add guidance. Engine/doc wording stays consistent with the CLI behavior introduced in T-002 and T-003.
  - Stop: Documentation implies local mode is less strict about acceptance quality or tracked mode no longer needs a close-out commit.
- [ ] T-006 [AC-11] Regenerate engine artifacts and run full verification
  - Depends on: T-005
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - `AGENTS.md`
    - `CLAUDE.md`
    - `.github/copilot-instructions.md`
    - `.kiro/steering/mochiflow.md`
    - `.kiro/agents/spec-plan-auditor.json`
    - `.kiro/agents/spec-change-reviewer.json`
  - Done: Run `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate`, `mochiflow adapter generate --check`, the configured default verification, `mochiflow doctor`, and `mochiflow lint --spec local-only-spec-mode`; record final AC Matrix evidence and the mandatory elevated-risk reviewer result before acceptance.
  - Stop: Generated adapter or vendored engine changes disagree with source `engine/` changes.
