# Verify profile should cover CI lint checks — Tasks

Implementation Summary: make `cli.default` local verification match practical CI checks and update workflow guidance.
risk: elevated
Critical Stop Conditions:
- New `default` command cannot be run reliably from a normal checkout
- Engine source and vendored engine drift after guidance edits
- Plan discovers `cargo-deny` must be locally mandatory

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-04] Update project verification profiles
  - Depends on: none
  - Files: `.mochiflow/config.toml`
  - Done: `cli.default` runs test, fmt, clippy, and freeze checks; `cli.quick` runs the test-only command; `cargo-deny` remains outside local `default`
  - Stop: if the chained default command is too brittle to run from the repository root

- [x] T-002 [AC-03, AC-04] Update workflow guidance for default versus quick profiles
  - Depends on: T-001
  - Files: `engine/reference/workflow.md`, `engine/commands/build.md`
  - Done: workflow/build docs identify `default` as canonical build/merge-equivalent verification and `quick` as optional fast feedback; `cargo-deny` is not implied to be locally covered
  - Stop: if changing build guidance requires a CLI behavior change to stay coherent

- [x] T-003 [AC-05] Refresh engine-generated artifacts
  - Depends on: T-002
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`, `AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`, `.kiro/**`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check` complete; generated outputs are either unchanged or explicitly committed if regenerated
  - Stop: if adapter generation changes unrelated files beyond the engine guidance update

- [x] T-004 [AC-01, AC-02, AC-03, AC-04, AC-06] Verify and record acceptance evidence
  - Depends on: T-003
  - Files: `.mochiflow/specs/verify-ci-parity/spec.md`, `.mochiflow/specs/verify-ci-parity/design.md`, `.mochiflow/specs/verify-ci-parity/tasks.md`
  - Done: new `cli.default` command passes; `mochiflow config show` confirms profiles; `mochiflow lint --spec verify-ci-parity` passes; AC Matrix rows are updated with evidence; elevated-risk review result is recorded in `design.md ## Review Results`; post-ship `refresh-context` follow-up is reported
  - Stop: if verification fails for a reason outside this spec's scope

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | Run the resolved `cli.default` command | `.mochiflow/config.toml` | PASS | `cargo test && cargo fmt --check && cargo clippy -D warnings && cargo run -- freeze --check` passed; `cargo test` ran 266 tests | |
| AC-02 | cli | automated | `mochiflow config show` | `.mochiflow/config.toml` | PASS | `mochiflow config show` lists `cli.default` full command and `cli.quick` test-only command | |
| AC-03 | cli | automated | Read docs and run `mochiflow lint --spec verify-ci-parity` | `engine/reference/workflow.md`, `engine/commands/build.md` | PASS | workflow/build docs updated; post-review `mochiflow lint --spec verify-ci-parity` passed 0 fail, 0 warn | |
| AC-04 | cli | automated | Read config/docs and compare to `.github/workflows/ci.yml` | `.mochiflow/config.toml`, `engine/reference/workflow.md`, `engine/commands/build.md` | PASS | local `default` covers test/fmt/clippy/freeze; docs keep human/CI-only checks excluded from `default` | |
| AC-05 | cli | automated | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check` | `engine/MANIFEST.json`, `.mochiflow/engine/**`, generated adapters if changed | PASS | `mochiflow freeze` wrote manifest; `mochiflow upgrade --source engine` completed; `mochiflow adapter generate --check` passed 0 drift, 0 failed | |
| AC-06 | cli | automated | New `cli.default` command plus `cargo test --manifest-path cli/Cargo.toml` | Changed config, docs, engine artifacts, and spec files | PASS | final `cli.default` command passed; post-review `mochiflow lint --spec verify-ci-parity` passed 0 fail, 0 warn | |
