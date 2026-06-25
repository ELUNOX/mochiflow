# Clarify doctor/freeze coherence and context freshness

## Background and Design Rationale

MochiFlow has two adjacent integrity checks that protect different scopes.
`doctor engine` validates the installed or vendored engine copy against its
MANIFEST. `freeze --check` validates source-repo derived files generated from
the workspace version and frozen contract inputs. A previous Version SSOT
decision intentionally separated these responsibilities, so this change keeps
that boundary and makes it clearer in CLI guidance and docs.

The context layer is code-derived orientation for agents. Existing `doctor
config` checks that context files exist and are not unfilled stubs, but it does
not detect when filled context references a removed public command. The chosen
freshness check is intentionally narrow: warn only on stale-looking
`mochiflow <command>` references to commands that the current CLI no longer
supports. This catches the observed failure mode without treating prose
freshness as a deterministic build artifact.

`freeze` currently resolves the source repo by walking upward from the current
working directory. Adding an explicit `--root` option gives CI and scripts a
stable way to name the source repo while preserving the existing cwd behavior.
`--config` is deliberately not reused because config belongs to installed
project state, while `freeze` is a source-repo command.

This work originated from the `doctor-freeze-coherence` backlog seed.

## User Story

As a MochiFlow contributor, I want the CLI and docs to tell me which health or
integrity check applies, so that I can run the right verification without stale
context misleading my agent.

## Scope

- In: source-repo guidance for `doctor` versus `freeze --check`.
- In: a warning-only context freshness check for removed public CLI commands.
- In: `mochiflow freeze --root <source-repo> [--check]`.
- In: automated tests for the new CLI behavior.
- In: README and docs updates explaining the intended usage.
- Out: changing the frozen contract hash inputs.
- Out: making `doctor` execute `freeze --check`.
- Out: automatic context regeneration or semantic context diffing.
- Out: changing `doctor engine` MANIFEST drift semantics.
- Out: deriving `freeze` source root from `--config`.

## Edge Cases

- A normal installed project that is not a MochiFlow source repo must not fail
  because `freeze --check` is unavailable.
- Full `mochiflow doctor` and `mochiflow doctor config` should report context
  freshness warnings consistently.
- Existing valid context that does not mention public command references should
  remain quiet.
- `mochiflow freeze --root` should work from outside the source repo and should
  fail clearly when the supplied path is not a MochiFlow source repo.
- `mochiflow freeze --check` without `--root` should retain current cwd-upward
  behavior.

## Acceptance Criteria (EARS)

- AC-01: WHEN a user runs `mochiflow doctor` or `mochiflow doctor config` in a
  MochiFlow source repo, THE SYSTEM SHALL emit warning-level guidance that
  source-repo derived-file coherence is checked with `mochiflow freeze --check`
  rather than by `doctor engine`.
- AC-02: WHEN any configured context file contains a `mochiflow <command>`
  reference whose command is not a current public MochiFlow command reference,
  THE SYSTEM SHALL report a WARN that names the context file and prompts a
  context refresh. Public command references include terminal CLI subcommands
  (`config`, `index`, `lint`, `doctor`, `adapter`, `upgrade`, `ready`,
  `backlog`, `init`, `join`, `detach`, `guide`, `completions`, `pr`, `freeze`)
  and agent workflow vocabulary (`discuss`, `plan`, `build`, `ship`, `patch`,
  `review`, `refresh-context`, `onboard`).
- AC-03: WHEN configured context files contain only current public CLI command
  references, current agent workflow command references, or no command
  references, THE SYSTEM SHALL NOT report the stale context command warning.
- AC-04: THE SYSTEM SHALL support `mochiflow freeze --root <source-repo>
  [--check]` and use that root instead of the current working directory for
  source-repo resolution.
- AC-05: WHEN `mochiflow freeze --root <path>` receives a path that is not a
  MochiFlow source repo, THE SYSTEM SHALL fail with a clear source-repo error
  and SHALL NOT write derived files.
- AC-06: THE SYSTEM SHALL keep `mochiflow freeze [--check]` without `--root`
  backward compatible with current cwd-upward source-root resolution.
- AC-07: THE SYSTEM SHALL document the distinction between installed-project
  health checks (`mochiflow doctor`) and source-repo derived-file checks
  (`mochiflow freeze --check`) in user-facing CLI documentation.

## QA Scenarios

| QA | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | cli | Automated | Run targeted CLI tests for doctor source guidance, context command freshness, and allowlist drift against real CLI subcommands. | Tests pass and cover WARN/no-WARN behavior plus allowlist coverage. |
| QA-02 | cli | Automated | Run targeted CLI tests for `freeze --root` success, invalid-root failure, and cwd fallback. | Tests pass and prove the new option does not break existing behavior. |
| QA-03 | cli | Automated | Run `mochiflow lint --spec doctor-freeze-coherence` after artifact updates. | Consistency check passes with 0 fail. |
| QA-04 | cli | Automated | Run the configured `cli.default` verification before completion. | Full CLI verification passes. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | CLI integration test for source-repo doctor guidance | `cli/crates/mochiflow-core/src/doctor.rs`, docs | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check` | Covered by `doctor_config_guides_source_repo_users_to_freeze_check` and final `cli.default`. |
| AC-02 | cli | automated | CLI integration test with stale `mochiflow <command>` context reference plus allowlist drift test against real CLI subcommands | `cli/crates/mochiflow-core/src/doctor.rs`, `cli/crates/mochiflow-cli/src/main.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` | Covered by `doctor_config_warns_on_stale_context_command_reference` and `doctor_terminal_command_allowlist_matches_clap_subcommands`. |
| AC-03 | cli | automated | CLI integration test with current CLI command, current workflow command, and no context command references | `cli/crates/mochiflow-core/src/doctor.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` | Covered by `doctor_config_allows_current_cli_and_workflow_command_references`. |
| AC-04 | cli | automated | CLI integration test for `freeze --root <source-repo> --check` from another cwd | `cli/crates/mochiflow-cli/src/main.rs`, `cli/crates/mochiflow-core/src/freeze.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check` | Covered by `freeze_root_check_uses_explicit_source_repo_from_other_cwd` and final `cli.default`. |
| AC-05 | cli | automated | CLI integration test for invalid `freeze --root` with no writes | `cli/crates/mochiflow-cli/src/main.rs`, `cli/crates/mochiflow-core/src/freeze.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` | Covered by `freeze_root_invalid_path_fails_before_writing` and `validate_repo_root_does_not_walk_to_parent`. |
| AC-06 | cli | automated | Existing and/or targeted freeze cwd-resolution test | `cli/crates/mochiflow-cli/src/main.rs`, `cli/crates/mochiflow-core/src/freeze.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check` | Covered by `freeze_without_root_keeps_cwd_upward_resolution` and existing freeze root tests. |
| AC-07 | cli | automated | Documentation content assertion tests plus default verification | `README.md`, `docs/versioning.md`, `contracts/VERSIONING.md`, `cli/crates/mochiflow-cli/tests/cli.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo fmt --manifest-path cli/Cargo.toml --all -- --check` | Covered by `docs_explain_doctor_freeze_boundaries_and_root_usage` and final `cli.default`. |
