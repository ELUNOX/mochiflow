# Verify profile should cover CI lint checks

## Background and Design Rationale

- The `cli` surface currently exposes `default = "cargo test --manifest-path cli/Cargo.toml"` in `.mochiflow/config.toml`, while `.github/workflows/ci.yml` also runs formatting, clippy, engine freeze, and cargo-deny checks before a pull request can pass.
- The build procedure and `mochiflow ready` both use the surface `default` profile as the canonical local verification signal. Leaving `default` narrower than CI lets agents complete local build work without running checks that CI will later reject.
- The agreed approach is to make `default` cover the locally toolchain-backed CI checks: `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings`, and `freeze --check`. `cargo-deny` remains CI-only because it is supplied by a GitHub Action and is not guaranteed by this repository's checked-in Rust toolchain configuration.
- A `quick` profile may keep the old fast test-only command, but build completion must depend on `default`.
- Primary source basis: The Cargo Book documents `cargo test` and `--manifest-path` for package selection; Cargo's `cargo fmt` page states that `cargo fmt` is an external Rust toolchain component; Clippy's official usage guide documents `cargo clippy` and using `-D warnings` in CI. The repository's `rust-toolchain.toml` pins Rust `1.96.0` with `rustfmt` and `clippy` components.
- Origin: backlog seed `verify-ci-parity` (source: conversation, ship phase observation).

## User Story

As a developer using MochiFlow's build flow, I want the configured local
verification command to catch the checks that CI will enforce, so that PRs do
not bounce back for avoidable formatting, lint, or freeze failures.

## Scope

- In: update `.mochiflow/config.toml` verification profiles for the `cli` surface.
- In: update engine workflow/build guidance so `default` is treated as the reliable merge-equivalent verification profile and `quick` is optional.
- In: verify the new command shape with local commands that are available in this repository.
- In: explicitly allow this spec's build work to edit `.mochiflow/config.toml`, because the config file is the target of AC-01 and AC-02 even though it is outside the configured `[write].allow` globs.
- Out: changing GitHub Actions, removing CI checks, or weakening PR requirements.
- Out: installing or vendoring `cargo-deny`, changing dependency audit policy, or making MochiFlow a CI runner.
- Out: editing `.mochiflow/context/tech.md` directly during build; record a post-ship `refresh-context` follow-up instead.
- Out: implementation changes unrelated to verification profile selection.

## Edge Cases

- The configured `default` command must fail if any subcheck fails; command chaining must not hide a failed test, format, clippy, or freeze step.
- The `quick` profile, if added, must not become the command that `mochiflow ready` or normal build completion depends on.
- Documentation must not imply `cargo-deny` has been run locally by default when it remains CI-only.
- Engine source edits require dogfood synchronization: `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check`.
- The `.mochiflow/config.toml` write exception is limited to this spec's verification profile change; it does not permit unrelated config, context, ADR, or runtime-state edits.
- Current-state context updates remain a separate `refresh-context` follow-up after ship.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL configure the `cli` surface `default` verification profile to run `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, and `cargo run --manifest-path cli/Cargo.toml -- freeze --check` as one failure-propagating local command.
- AC-02: THE SYSTEM SHALL provide a fast `cli` verification profile for test-only iteration without changing the meaning of `default` as the build-completion profile.
- AC-03: WHEN an agent reads workflow/build guidance, THE SYSTEM SHALL describe `default` as the reliable merge-equivalent verification profile and `quick` as optional fast feedback.
- AC-04: THE SYSTEM SHALL keep `cargo-deny` documented as CI-only unless a locally guaranteed standard tool path is added.
- AC-05: WHEN engine guidance changes, THE SYSTEM SHALL refresh the frozen and vendored engine artifacts required by the project constitution.
- AC-06: WHEN the plan is built, THE SYSTEM SHALL verify the changed configuration and guidance with the configured local checks that prove the new command path is runnable.

## QA Scenarios

| QA | Scope | Steps | Expected result |
| --- | --- | --- | --- |
| QA-01 | cli | Run `mochiflow config show` after the change | `cli.default` shows the full local verification command and `cli.quick` shows the test-only command |
| QA-02 | cli | Run the new `cli.default` command from `.mochiflow/config.toml` | test, fmt, clippy, and freeze checks all complete successfully or fail the command at the failing step |
| QA-03 | cli | Read `engine/reference/workflow.md` and `engine/commands/build.md` after the change | `default` is described as merge-equivalent/canonical build verification, and `quick` is described only as a faster optional profile |
| QA-04 | cli | Run `mochiflow adapter generate --check` after engine sync | Generated adapter outputs are in sync |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
