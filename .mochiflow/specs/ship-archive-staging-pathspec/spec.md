# Make ship archive staging resilient to moved spec paths

## Background and Design Rationale

Ship close-out currently depends on an agent applying several Git and lifecycle
steps correctly after an active spec directory is moved from
`{specs_dir}/{slug}/` to `{specs_dir}/_done/{slug}/`. The fragile point is
staging: a pathspec that names the moved-from slug directory can fail after the
move, even though the intended operation is the normal close-out move.

The long-term fix is to make the terminal CLI own the automatable part of ship
instead of exposing a narrow staging helper. The public command should be
`mochiflow ship`, matching the existing workflow vocabulary. Human and agent
judgment remains outside the CLI: final QA interpretation, durable learning
content, and PR text approval still happen through the workflow artifacts and
conversation. Once those judgments are recorded, `mochiflow ship` should perform
the deterministic completion mechanics: readiness checks, final verification,
done metadata, index regeneration, archive move, safe staging, staged-diff
validation, and close-out commit.

The design follows the standard Git model: use `git add -A` constrained by
configured pathspecs to capture additions and removals under lifecycle paths,
then use machine-readable Git output to verify what is staged before committing.
The basis is Git's official documentation for `git add`, `git status`, and
`git diff`, plus clap's derive API for adding a first-class CLI command in the
same style as existing MochiFlow commands.

This work originated from the backlog seed
`ship-archive-staging-pathspec`, created after a close-out attempt failed when
staging the removed active spec path directly.

## User Story

As a MochiFlow user, I want `mochiflow ship` to complete the mechanical ship
close-out safely, so that agents spend fewer tokens on repeatable Git mechanics
and close-out failures do not interrupt PR handoff.

## Scope

- In:
  - Add a first-class `mochiflow ship [slug]` CLI command.
  - Resolve the target spec from an explicit slug or, when omitted, from the
    current feature branch.
  - Validate ship readiness before mutating lifecycle files.
  - Run configured final verification for the spec surfaces.
  - Set `status: done`, `updated`, and `completed`.
  - Move the spec to `{specs_dir}/_done/{slug}/`, regenerating `{index}`.
  - Stage only configured lifecycle paths with a stable parent pathspec.
  - Verify the staged result with machine-readable Git output before committing.
  - Create the ship close-out commit with required traceability.
  - Update `mochiflow pr` pre-flight to require the committed ship close-out
    when `--spec <slug>` is supplied.
  - Update shared ship guidance and command allowlists.
- Out:
  - Automating human QA judgment.
  - Generating or approving PR title/body.
  - Running `mochiflow pr` from inside `mochiflow ship`.
  - Post-merge branch cleanup.
  - Replacing the full agent-driven ship phase.

## Edge Cases

- The active spec is already moved to `_done/{slug}` after an interrupted run.
- The active spec and archived spec both exist for the same slug.
- The current branch does not identify a spec and no slug is provided.
- The target spec is already `done`.
- The target spec still has `PENDING_HUMAN`, `UNVERIFIED`, or `FAIL` matrix
  rows.
- Required elevated-risk reviewer results are missing.
- The final verification command is missing, a TODO placeholder, or fails.
- Unrelated working tree changes exist before ship starts.
- Unrelated staged changes exist before ship starts.
- `{specs_dir}` is configured to a non-default path.
- Specs are gitignored, so lifecycle artifact staging may be intentionally empty.
- ADR files are unchanged because there are no durable learnings to record.

## Acceptance Criteria (EARS)

- AC-01: WHEN a user runs `mochiflow ship [slug]` for an approved, fully verified spec, THE SYSTEM SHALL run the configured final verification for every declared surface before changing the spec to `done`.
- AC-02: WHEN all ship readiness checks and final verification pass, THE SYSTEM SHALL set `spec.yaml` to `status: done` with current `updated` and `completed` values.
- AC-03: WHEN the target spec is active under `{specs_dir}/{slug}/`, THE SYSTEM SHALL move it to `{specs_dir}/_done/{slug}/` and regenerate the configured index.
- AC-04: WHEN staging ship close-out changes, THE SYSTEM SHALL stage configured lifecycle paths using a stable parent pathspec that captures the archive deletion and addition without using the moved-from slug path as a required pathspec.
- AC-05: IF unrelated working tree or staged changes exist before ship starts, THEN THE SYSTEM SHALL stop before mutating lifecycle files and report the unrelated paths.
- AC-06: IF the staged result includes paths outside the allowed ship close-out set, THEN THE SYSTEM SHALL stop before committing and report the unexpected paths.
- AC-07: WHEN the staged result is valid, THE SYSTEM SHALL create one close-out commit with a Conventional Commit subject and a `Spec: <slug>` trailer.
- AC-08: IF `mochiflow pr --spec <slug>` is run before the requested spec's ship close-out is committed, THEN THE SYSTEM SHALL fail pre-flight before pushing.
- AC-09: WHEN `mochiflow ship` is re-run after an interrupted close-out, THE SYSTEM SHALL either resume safely from the detected lifecycle state or stop with a precise, non-destructive recovery message.
- AC-10: WHEN engine ship guidance describes manual fallback staging, THE SYSTEM SHALL recommend `git add -A` scoped to configured lifecycle parents instead of a pathspec that requires `{specs_dir}/{slug}` after the archive move.

## QA Scenarios

| QA | Persona | Scope | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | P1 new user | cli | Run `mochiflow ship` on a branch whose name maps to exactly one approved spec. | The command resolves the spec, prints the verification and close-out actions, and either completes or reports the first missing readiness item. |
| QA-02 | P2 power user | cli | Run `mochiflow ship <slug>` for a configured non-default `specs_dir`. | The command uses configured paths and does not assume `.mochiflow/specs`. |
| QA-03 | P3 malicious user | cli | Create an unrelated staged file before running `mochiflow ship <slug>`. | The command stops before lifecycle mutation and reports the pre-existing staged path. |
| QA-04 | P4 data integrity | cli | Ship a fixture with an active spec move, index update, and ADR change. | The resulting commit contains only allowed lifecycle paths and the `Spec:` trailer. |
| QA-05 | P5 migration | cli | Use a legacy done spec without `completed`, and a current active approved spec. | The command only modifies the target active spec and does not retrofit unrelated archived specs. |
| QA-06 | P6 regression | cli | Run existing `mochiflow pr` tests and then run `mochiflow pr --spec <slug>` before ship. | Existing PR behavior remains intact, while the new slug-aware pre-flight blocks an unshipped spec. |
| QA-07 | P7 spec skeptic | cli | Compare this specification's allowed path rules with `git diff --cached --name-status` after ship. | The staged or committed paths match the specified lifecycle set and do not include unrelated files. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Required elevated-risk review result is recorded in `design.md ## Review Results`.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | CLI integration test with a passing surface verify command; QA-01 | `cli/crates/mochiflow-core/src/ship.rs`, `cli/crates/mochiflow-cli/src/main.rs` | UNVERIFIED |  |  |
| AC-02 | cli | automated | CLI integration test inspects archived `spec.yaml`; QA-04 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-03 | cli | automated | CLI integration test inspects `_done/{slug}` and regenerated `INDEX.md`; QA-04 | `cli/crates/mochiflow-core/src/ship.rs`, `cli/crates/mochiflow-core/src/index.rs` | UNVERIFIED |  |  |
| AC-04 | cli | automated | Git fixture test verifies active deletion and archived addition are captured; QA-02, QA-07 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-05 | cli | automated | CLI integration tests for dirty working tree and pre-staged unrelated file; QA-03 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-06 | cli | automated | Unit or integration test injects unexpected staged path before validation; QA-03, QA-07 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-07 | cli | automated | CLI integration test inspects latest commit subject and trailers; QA-04 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-08 | cli | automated | PR integration test verifies `mochiflow pr --spec <slug>` fails before ship and passes pre-flight after ship on dry-run; QA-06 | `cli/crates/mochiflow-core/src/pr.rs` | UNVERIFIED |  |  |
| AC-09 | cli | automated | Integration tests for already-moved and partially staged lifecycle states; QA-01, QA-04 | `cli/crates/mochiflow-core/src/ship.rs` | UNVERIFIED |  |  |
| AC-10 | cli | automated | Conformance test checks engine guidance text and adapter generated output; QA-07 | `engine/commands/ship.md`, `engine/reference/git.md`, `.mochiflow/engine/**` | UNVERIFIED |  |  |
