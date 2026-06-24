# Prevent build-phase spec mutation — Design

## Design Decisions

- Treat approved `tasks.md` structure as a build contract rather than mutable
  implementation scratch space. This preserves the approve-to-build gate while
  still allowing build to record progress and evidence.
- Use a procedural stop condition instead of structural drift detection. Hashing
  or snapshotting approved task structure is intentionally out of scope for this
  change.
- Formalize compound AC task references instead of requiring one task per AC.
  A single task may naturally satisfy multiple related criteria, and lint should
  count every AC ID in the reference.
- Keep generated adapter outputs as generated artifacts. Source edits happen in
  `engine/`, then project-local `.mochiflow/engine/` is updated through the
  required dogfood sync.

## Architecture

- Build guidance lives in `engine/commands/build.md`, then syncs into the
  vendored `.mochiflow/engine/commands/build.md`.
- Task authoring rules live in `engine/reference/authoring.md`.
- Task examples live in `engine/templates/spec/tasks.md`.
- Lint behavior for AC extraction lives in `cli/crates/mochiflow-core/src/lint.rs`;
  tests live in `cli/crates/mochiflow-cli/tests/conformance.rs`.
- Engine-source edits require `mochiflow freeze`, `mochiflow upgrade --source
  engine`, and `mochiflow adapter generate --check`.

## Data Model / Interfaces

- No schema, CLI flag, or public command interface changes are planned.
- The task-line interface remains Markdown: `- [ ] T-### [AC-01] title`, with
  compound AC references documented as `- [ ] T-### [AC-01, AC-02] title`.
- AC extraction remains ID-based. Any implementation change should preserve
  existing `Covers AC:` support.

## Error Handling

- If targeted tests show compound references are already supported, leave
  `lint.rs` unchanged and rely on tests to lock the behavior.
- If tests expose a parsing gap, fix only the AC extraction path needed for task
  coverage and unknown-reference checks.
- If engine sync or adapter check reports generated drift, resolve the drift
  before build completion.

## Test Strategy

- Add a focused conformance test for a completed spec where one task references
  `[AC-01, AC-02]` and both ACs must be considered covered.
- Keep or add an unknown-reference assertion if compound references expose a
  regression path.
- Run the project default verification for the `cli` surface:
  `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`.

## Review Results

Reviewer mode: delegated
Verdict: pass
Reviewed after implementation and verification. No findings.
