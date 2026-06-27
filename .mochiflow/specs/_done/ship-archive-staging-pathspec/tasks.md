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

- [x] T-001 [AC-01, AC-02, AC-03, AC-09, AC-11] Add ship target resolution and lifecycle mutation
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `mochiflow ship [slug]` resolves an approved active spec, supports dry-run without mutation, runs final verification, stops before mutation for failing/missing/TODO verification commands, updates eligible automated AC Matrix rows to `PASS` without overwriting AC-specific evidence, writes done metadata with `completed` formatted as `YYYY-MM-DDTHH:MM:SSZ`, moves the spec to `_done`, regenerates the index while leaving ignored runtime state unstaged, and handles active-only, archived-only uncommitted, both active and archived, neither present, already done, and partially staged states in tests.
  - Stop: stop if target resolution needs a branch convention that conflicts with existing `reference/git.md`.
- [x] T-002 [AC-04, AC-05, AC-06, AC-07] Implement safe staging validation and close-out commit
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: ship stages only configured lifecycle paths, uses the ship lifecycle allowlist rather than branch-switch dirt rules, rejects unrelated dirt using `git status --porcelain=v1 -z`, validates `git diff --cached --name-status -z`, handles paths with spaces/special characters, and creates one close-out commit with `Spec: <slug>` trailer.
  - Stop: stop if Git path handling cannot be expressed with configured paths without repository-wide staging.
- [x] T-003 [AC-08] Add slug-aware PR pre-flight guard
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: non-dry-run `mochiflow pr --spec <slug>` fails before push when ship close-out is not committed, accepts a committed done spec on the manual-handoff path without network access, and preserves existing path-like request-dir behavior without applying the slug guard.
  - Stop: stop if `--spec` ambiguity between slug and explicit request directory cannot remain backward-compatible.
- [x] T-004 [AC-10] Update shared guidance, adapters, and frozen artifacts
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
- [x] T-005 [AC-02, AC-04, AC-07, AC-09, AC-10] Record evidence and dogfood close-out constraints
  - Depends on: T-001, T-002, T-004
  - Files:
    - `.mochiflow/specs/ship-archive-staging-pathspec/spec.md`
    - `.mochiflow/specs/ship-archive-staging-pathspec/design.md`
    - `.mochiflow/specs/ship-archive-staging-pathspec/tasks.md`
  - Done: AC Matrix evidence names concrete test functions or command outputs for each automated behavior, QA-04 records committed path/trailer evidence, QA-05 records legacy done-spec non-modification evidence, and this spec's own close-out plan explicitly uses the AC-10 manual fallback instead of the new command.
  - Stop: stop if evidence would rely only on a generic final verification PASS token.
