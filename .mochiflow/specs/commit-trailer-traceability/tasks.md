# Add git trailers for spec/task traceability — Tasks

Implementation Summary: Add trailer rules, relaxed body rule, AI recipes, and build resume procedure to engine docs; freeze manifest and sync vendored engine.
risk: standard
Critical Stop Conditions:
- Do not modify CLI source code
- Do not change contract schemas

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-04] Relax external-reviewer commit rule in engine/reference/git.md
  - Depends on: none
  - Files: `engine/reference/git.md`
  - Done: Commit section permits slug in body, forbids slug/AC/vocabulary in subject, and references trailers for metadata
  - Stop: changing meaning of existing non-trailer rules

- [x] T-002 [AC-01] Add Trailers section to engine/reference/git.md
  - Depends on: T-001
  - Files: `engine/reference/git.md`
  - Done: New `## Trailers` section documents `Spec:` and `Task:` format, required/optional rules per phase, and multi-task format
  - Stop: introducing hook or CLI requirements

- [x] T-003 [AC-05] Add body restriction rule for `Spec:` line prefix
  - Depends on: T-002
  - Files: `engine/reference/git.md`
  - Done: Rule states body must not start a line with `Spec:` (reserved for trailer parsing safety)
  - Stop: over-restricting body content

- [x] T-004 [AC-02] Add AI Git Log Recipes section to engine/reference/git.md
  - Depends on: T-002
  - Files: `engine/reference/git.md`
  - Done: New `## AI Git Log Recipes` section with recipes for: all commits for a spec, last completed task, recent file changes with spec context
  - Stop: recipes requiring custom tooling beyond git builtins

- [x] T-005 [AC-03] Add build resume procedure to engine/commands/build.md
  - Depends on: T-004
  - Files: `engine/commands/build.md`
  - Done: Procedure section for new-session resume: read tasks.md → git log cross-check → resume from first unchecked task. Specifies stop-and-reconcile when trailers and checkboxes disagree (checked task lacks matching trailer, or trailer exists for unchecked task).
  - Stop: changing existing build procedure steps

- [x] T-006 [AC-06] Delete orphaned tasks-checkbox-enforcement backlog seed
  - Depends on: none
  - Files: `.mochiflow/specs/_backlog/tasks-checkbox-enforcement.md`
  - Done: File no longer exists
  - Stop: deleting other backlog files

- [x] T-007 [AC-07] Regenerate engine manifest and sync vendored engine
  - Depends on: T-001, T-002, T-003, T-004
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/reference/git.md`, `.mochiflow/engine/commands/build.md`
  - Done: `mochiflow freeze` succeeds, `mochiflow upgrade --source engine` succeeds, `mochiflow freeze --check` exit 0, `mochiflow doctor` exit 0
  - Stop: freeze or doctor failures unrelated to this spec's changes
