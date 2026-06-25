# Lint: detect template residue and multi-line EARS ACs - Tasks

Implementation Summary: Harden `mochiflow lint` so unfinished template residue fails and multi-line EARS ACs are recognized correctly.
risk: standard
Critical Stop Conditions:
- Stop if residue detection needs a full Markdown parser instead of small scoped helpers.
- Stop if a candidate residue pattern cannot distinguish unfinished template text from common authored examples.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-04] Add template residue lint checks
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/lint.rs`
  - Done: Expanded spec documents fail on unfinished residue classes while fenced code, inline code, and valid AC Matrix provisional tokens remain accepted.
  - Stop: A residue class has no reliable signal without broad Markdown parsing or high false-positive risk.

- [ ] T-002 [AC-03] Scan EARS keywords across AC blocks
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/lint.rs`
  - Done: EARS warning detection treats each AC declaration and its continuation lines as one block ending at the next AC declaration or section heading.
  - Stop: Existing AC ID collection would need a schema-level rewrite instead of a local lint helper.

- [ ] T-003 [AC-01, AC-02, AC-03, AC-04] Add conformance coverage
  - Depends on: T-001, T-002
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Tests prove residue failures, code-example false-positive avoidance, valid matrix placeholders, and multi-line EARS acceptance.
  - Stop: Tests require fixture setup that duplicates large portions of existing lint harness code.

- [ ] T-004 [AC-05] Align guidance and generated engine artifacts
  - Depends on: T-001, T-002, T-003
  - Files: `engine/commands/plan.md`, `engine/reference/authoring.md`, `engine/MANIFEST.json`, `.mochiflow/engine/commands/plan.md`, `.mochiflow/engine/reference/authoring.md`, `.mochiflow/engine/MANIFEST.json`, `AGENTS.md`, `CLAUDE.md`, `.github/copilot-instructions.md`, `.kiro/steering/mochiflow.md`
  - Done: Guidance reflects that lint enforces residue checks, and required freeze / upgrade / adapter checks leave generated artifacts consistent.
  - Stop: Guidance changes expand into unrelated workflow policy changes.

- [ ] T-005 [AC-01, AC-02, AC-03, AC-04, AC-05] Run final verification and update results
  - Depends on: T-004
  - Files: `.mochiflow/specs/lint-residue-and-multiline-ears/spec.md`, `.mochiflow/specs/lint-residue-and-multiline-ears/tasks.md`
  - Done: Default CLI verification, spec lint, and AC Matrix evidence are recorded with passing results.
  - Stop: Any required verification remains failing after the implementation tasks are complete.
