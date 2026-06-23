# Add git trailers for spec/task traceability and AI log reading recipes

## Background and Design Rationale

Origin: `_backlog/commit-trailer-traceability.md` (discuss handoff, 2026-06-23).

Commit messages follow Conventional Commits but have no machine-parseable link to
the spec or task that motivated them. `git log` cannot answer "which commits
belong to spec X?" or "what was the last task completed?". The current rule
("no mochiflow vocabulary in commit messages") prevents putting slugs in the
subject, but trailers (footer metadata) are invisible to casual readers and
machine-parseable via `%(trailers:...)`.

When an AI agent resumes a build in a new session, it relies solely on tasks.md
checkboxes and file reads. Git trailers give a second, commit-level signal for
cross-checking progress.

Key decisions from discuss:
- Subject remains slug-free; body may mention slug naturally; trailers are required metadata.
- Insertion is document-based (AI instruction), not hook-based (no CLI change).
- Recipes live in git.md (reusable); resume procedure lives in build.md (build-specific).
- No dependency on tasks-checkbox-enforcement (already implemented).

Risk rationale: `standard` is appropriate because the change is purely additive
engine documentation (reversible by revert), touches no CLI code, no data model,
no contract schema, no external API, and no migration. The cross-cutting nature
of new commit rules is mitigated by the fact that agents already follow
`reference/git.md` instructions — adding new sections is the same mechanism,
not a new integration surface.

AC-06 rationale: `tasks-checkbox-enforcement` was the blocking dependency for
the resume-traceability workflow described in this spec. Now that it is
implemented, the orphaned backlog seed is dead weight and belongs to this spec's
cleanup scope (traceability infrastructure housekeeping).

## User Story

As an AI coding agent resuming a build session, I want to query git history by
spec slug and task ID so that I can cross-check tasks.md progress and identify
exactly which commits implemented each task.

## Scope

- In: engine source docs (`engine/reference/git.md`, `engine/commands/build.md`),
  `mochiflow freeze` to regenerate `engine/MANIFEST.json`, vendored engine sync
  via `mochiflow upgrade --source engine`, deletion of orphaned backlog seed.
- Out: CLI code changes. Hook scripts. Lint/doctor enforcement of trailers.

## Edge Cases

- A commit spans multiple tasks: use one `Task:` trailer line per task.
- Ship close-out commit: `Spec:` required, `Task:` optional (bundles multiple concerns).
- Patch lane commit: no trailers (no spec context exists).
- discuss/plan commit (spec scaffold only): `Spec:` required, `Task:` not applicable.

## Requirements / Acceptance Criteria

| AC | Type | Priority | Requirement | Verification |
| --- | --- | --- | --- | --- |
| AC-01 | functional | Must | THE SYSTEM SHALL document trailer format rules (`Spec:` and `Task:`) in `engine/reference/git.md` under a new `## Trailers` section | automated |
| AC-02 | functional | Must | THE SYSTEM SHALL document AI git log recipes in `engine/reference/git.md` under a new `## AI Git Log Recipes` section | automated |
| AC-03 | functional | Must | THE SYSTEM SHALL document the build resume procedure in `engine/commands/build.md` | automated |
| AC-04 | functional | Must | THE SYSTEM SHALL relax the external-reviewer commit rule to permit slug mentions in body and require trailers, updating the existing Commit section | automated |
| AC-05 | functional | Must | THE SYSTEM SHALL add a rule that body text must not begin a line with `Spec:` (reserved for trailers) | automated |
| AC-06 | functional | Should | THE SYSTEM SHALL delete `_backlog/tasks-checkbox-enforcement.md` | automated |
| AC-07 | functional | Must | THE SYSTEM SHALL regenerate `engine/MANIFEST.json` via `mochiflow freeze` after source engine edits | automated |
| AC-08 | functional | Must | THE SYSTEM SHALL sync the vendored engine via `mochiflow upgrade --source engine` and pass `mochiflow freeze --check` and `mochiflow doctor` | automated |

## QA Scenarios

| # | Scope | Operation | Expected |
| --- | --- | --- | --- |
| QA-01 | cli | Run `cargo test` | All existing tests pass (no regression from docs-only change) |
| QA-02 | cli | Run `mochiflow freeze --check` | Exit 0 (manifest matches source engine) |
| QA-03 | cli | Run `mochiflow doctor` | Exit 0 (engine integrity check passes) |

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `rg "## Trailers" engine/reference/git.md` + content check for `Spec:` and `Task:` rules | `engine/reference/git.md` | PASS | heading + rules present | |
| AC-02 | cli | automated | `rg "## AI Git Log Recipes" engine/reference/git.md` + content check for recipe commands | `engine/reference/git.md` | PASS | heading + recipes present | |
| AC-03 | cli | automated | `rg "resume" engine/commands/build.md` + content check for resume steps | `engine/commands/build.md` | PASS | resume section with stop-and-reconcile | |
| AC-04 | cli | automated | `rg "Trailers are metadata" engine/reference/git.md` or equivalent relaxed rule text | `engine/reference/git.md` | PASS | rule text present | |
| AC-05 | cli | automated | `rg "must not begin a line with" engine/reference/git.md` or equivalent body-prefix rule | `engine/reference/git.md` | PASS | rule text present | |
| AC-06 | cli | automated | file absence check: `! test -f .mochiflow/specs/_backlog/tasks-checkbox-enforcement.md` | `.mochiflow/specs/_backlog/tasks-checkbox-enforcement.md` | PASS | file deleted | |
| AC-07 | cli | automated | `mochiflow freeze --check` exit 0 | `engine/MANIFEST.json` | PASS | exit 0 | |
| AC-08 | cli | automated | `mochiflow doctor` exit 0 | `.mochiflow/engine/**` | PASS | 0 fail | |
