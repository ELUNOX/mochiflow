# Prevent build-phase spec mutation — Tasks

Implementation Summary: clarify the approved-task boundary, formalize compound
AC task references, and verify lint supports that contract.
risk: elevated
Critical Stop Conditions:
- Stop if implementation requires snapshot/hash drift detection.
- Stop if the change would require a public CLI/schema contract change.
- Stop if compound AC references conflict with existing lint behavior in a way
  that cannot be fixed locally.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02] Add the build-phase task-structure stop condition
  - Depends on: none
  - Files: `engine/commands/build.md`
  - Done: `mochiflow-build` guidance distinguishes allowed progress/evidence
    edits from structural `tasks.md` edits that must return to plan.
  - Stop: Stop if the rule requires mechanical drift detection to be credible.
- [x] T-002 [AC-03] Document compound AC references
  - Depends on: T-001
  - Files: `engine/reference/authoring.md`, `engine/templates/spec/tasks.md`
  - Done: Authoring guidance and task template examples show that one task may
    cover multiple ACs with `[AC-01, AC-02]`.
  - Stop: Stop if the resulting syntax conflicts with the existing task-line
    parser.
- [x] T-003 [AC-04] Lock compound AC coverage in lint tests
  - Depends on: T-002
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`, `cli/crates/mochiflow-core/src/lint.rs`
  - Done: A focused test proves compound task references cover all mentioned ACs;
    `lint.rs` is changed only if the test exposes a real parsing gap.
  - Stop: Stop if test setup requires changing the lint contract beyond AC
    extraction.
- [x] T-004 [AC-05] Sync engine artifacts and verify
  - Depends on: T-001, T-002, T-003
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`, generated adapter check output
  - Done: Required freeze, dogfood upgrade, adapter check, spec lint, and default
    CLI verification pass; changed generated artifacts are staged explicitly.
  - Stop: Stop if generated adapter output would require hand editing.
