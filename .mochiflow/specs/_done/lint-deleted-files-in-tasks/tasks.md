# Suppress unchecked-task warnings for planned file deletions - Tasks

Implementation Summary: Add explicit planned-deletion support to lint task file checks.
risk: elevated
Critical Stop Conditions:
- Stop if supporting `deleted: ` requires a broader task schema redesign.
- Stop if git status cannot distinguish deleted paths without weakening existing dirty-file warnings.
- Stop if engine template updates reveal generated adapter drift that cannot be resolved by the standard freeze/upgrade flow.
- Stop if the mandatory elevated-risk review fails after implementation.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03, AC-04] Add deletion-aware lint parsing
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/lint.rs`
  - Done:
    - Lint can preserve git dirty status kind for task file warnings.
    - `deleted: ` entries suppress the warning only when git reports deletion.
    - Normal dirty entries and non-deleted `deleted: ` entries keep warning while unchecked.
  - Stop:
    - Stop if deletion handling requires changing task completion semantics instead of only warning selection.

- [x] T-002 [AC-01, AC-02, AC-03, AC-04] Add conformance coverage for deletion-marked task files
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done:
    - A deleted tracked file listed as `deleted: ` does not emit the unchecked-task dirty-file warning.
    - A normal dirty file still emits the existing warning while unchecked.
    - A non-deleted dirty path listed as `deleted: ` still emits the warning while unchecked.
    - Checked-task behavior remains covered.
  - Stop:
    - Stop if the fixture cannot reliably create deleted, modified, and untracked git status cases.

- [x] T-003 [AC-05] Document planned-deletion task file notation
  - Depends on: T-001
  - Files:
    - `engine/reference/authoring.md`
    - `engine/templates/spec/tasks.md`
    - `.mochiflow/engine/reference/authoring.md`
    - `.mochiflow/engine/templates/spec/tasks.md`
    - `engine/MANIFEST.json`
  - Done:
    - Authoring guidance and task template describe `deleted: ` inside `Files:` blocks.
    - Dogfood engine sync is completed with `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check`.
  - Stop:
    - Stop if documenting the marker requires changing adapter output wording beyond generated MochiFlow instructions.

- [x] T-004 [chore: elevated-risk review] Run mandatory independent review
  - Depends on: T-001, T-002, T-003
  - Files:
    - `.mochiflow/specs/lint-deleted-files-in-tasks/design.md`
  - Done:
    - Independent reviewer verdict is recorded in `design.md ## Review Results`.
    - Verdict is `pass` or `pass-with-comments`.
  - Stop:
    - Stop if the reviewer returns `fail`; fix findings before build completion.
