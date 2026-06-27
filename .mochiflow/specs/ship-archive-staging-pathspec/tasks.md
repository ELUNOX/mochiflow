# Make ship archive staging resilient to moved spec paths — Tasks

Implementation Summary: Add `mochiflow ship` as the CLI owner for safe ship close-out mechanics.
risk: elevated
Critical Stop Conditions:
- Stop if the command would need to automate human QA or PR approval.
- Stop if the staged allowlist cannot distinguish target lifecycle files from unrelated specs.
- Stop if a new public config/schema contract becomes necessary.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01, AC-02, AC-03, AC-09] Add ship target resolution and lifecycle mutation
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `mochiflow ship [slug]` resolves an approved active spec, runs final verification, writes done metadata, moves the spec to `_done`, regenerates the index, and handles documented retry states in tests.
  - Stop: stop if target resolution needs a branch convention that conflicts with existing `reference/git.md`.
- [ ] T-002 [AC-04, AC-05, AC-06, AC-07] Implement safe staging validation and close-out commit
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: ship stages only configured lifecycle paths, rejects unrelated dirt, validates `git diff --cached --name-status`, and creates one close-out commit with `Spec: <slug>` trailer.
  - Stop: stop if Git path handling cannot be expressed with configured paths without repository-wide staging.
- [ ] T-003 [AC-08] Add slug-aware PR pre-flight guard
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `mochiflow pr --spec <slug> --dry-run` fails before push when ship close-out is not committed and preserves existing PR behavior for request-dir use.
  - Stop: stop if `--spec` ambiguity between slug and explicit request directory cannot remain backward-compatible.
- [ ] T-004 [AC-10] Update shared guidance, adapters, and frozen artifacts
  - Depends on: T-001, T-002, T-003
  - Files:
    - `engine/commands/ship.md`
    - `engine/reference/git.md`
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/commands/ship.md`
    - `.mochiflow/engine/reference/git.md`
    - `.mochiflow/engine/MANIFEST.json`
    - `AGENTS.md`
    - `.github/copilot-instructions.md`
    - `.kiro/steering/mochiflow.md`
    - `CLAUDE.md`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: engine guidance calls `mochiflow ship`, manual fallback uses configured lifecycle parent pathspecs, command allowlists include terminal `ship`, and dogfood generated artifacts are in sync.
  - Stop: stop if generated adapter outputs contain unrelated user-authored changes outside managed blocks.

