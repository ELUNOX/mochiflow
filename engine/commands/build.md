---
name: spec-build
phase: build
description: |
  mochiflow's build phase. Implement an approved spec inline by executing
  tasks.md when present, maintaining the AC Matrix, verifying, committing, and
  running risk-based review through independent-reviewer transport. Activate on
  the explicit command `mochiflow-build`, or natural phrasing like "実装して" /
  "進めて" / "ビルドして". Does not create PRs, set `done`, or archive.
triggers:
  - mochiflow-build
  - 実装して
  - 進めて
  - ビルドして
trigger_patterns:
  - "{slug} build"
artifacts:
  - "{specs_dir}/{slug}/spec.md (AC Matrix)"
  - "{specs_dir}/{slug}/tasks.md (when present)"
prerequisites:
  - "{specs_dir}/{slug}/spec.yaml exists with status approved"
execution: both
delegate_to:
  - agents/independent-reviewer.md
references:
  - reference/workflow.md
  - reference/risk.md
  - reference/git.md
  - reference/engineering-standards.md
allowed_writes:
  - "{specs_dir}/**"
  - "{write.allow}"
forbidden_writes:
  - "{write.deny}"
  - .git/**
---

# mochiflow-build

## Purpose

Implement the approved spec and produce verified evidence in the AC Matrix. Do
not create the PR, set `done`, fold, or archive.

## Procedure

1. Read `spec.yaml`; confirm `status: approved` and risk / type / surfaces.
   Read `spec.md` plus `design.md` / `tasks.md` if present, the constitution,
   the foundational context, and `[adr].pitfalls`.
2. Check commit granularity in `reference/risk.md ## Consequences` and decide
   this build's commit unit. Prepare the branch per `reference/git.md ## Branch`.
3. Load `tasks.md` if it exists. If it does not exist, treat the approved
   `spec.md` as a micro task list and implement only the scoped ACs.
4. When `tasks.md` exists, execute incomplete top-level checkbox tasks:
   - Find tasks whose line starts with `- [ ] T-###`.
   - Respect wave order.
   - Respect `Depends on`; execute only tasks whose dependencies are complete.
   - Treat `[P]` as "parallelizable", not as a requirement to run in parallel.
   - Do not silently skip a task.
5. For each runnable task:
   - Review the listed `Files` and surrounding source before editing.
   - Implement the change, matching existing style and keeping the diff scoped.
   - Update or add tests.
   - Run the canonical command from `reference/workflow.md ## Verification profiles`.
   - Update the AC Matrix row(s): Implementation, Result, Evidence, and Notes.
   - Record `PASS`, `FAIL`, `N/A: <reason>`, or `PENDING_HUMAN` as appropriate.
   - Mark the task `[x]` after implementation, verification, and Matrix updates
     are ready for the current commit unit.
   - Create a commit when the commit policy says to do so. Include the
     implementation, tests, AC Matrix updates, and task checkbox update in the
     same commit unit whenever practical.
6. Append seam decisions / ownership / dead-code handling to
   `design.md ## Integration Log` only when `design.md` exists and the
   integration-log column in `reference/risk.md` calls for it. For `standard`,
   do not create or require an Integration Log.
7. Follow the reviewer cadence in `reference/risk.md`. For `critical`, run
   required review after each task and pass before moving to the next task. For
   `elevated`, run required review once after all tasks. Record reviewer mode
   and verdict in `design.md ## Review Results`.
8. If reviewer reports High/Critical findings, fix them in follow-up work, run
   verification again, update Review Results and AC Matrix, and commit the fix.
9. After all required tasks complete, run final verification once more. Fix any
   FAIL and re-run to PASS.
10. Commit any remaining build-time artifact updates in the final build commit
    for this phase, then stop. `ship` only commits Matrix rows or evidence
    changed by final verification / human QA, as part of close-out.

## AC Matrix Result Rules During Build

- Automated verification that passed becomes `PASS`.
- Automated verification that failed becomes `FAIL` and blocks completion.
- Human QA needed but not completed becomes `PENDING_HUMAN`; this is allowed
  during build but not at ship.
- Not-applicable rows must be written as `N/A: <reason>` with a concrete reason.
- Do not use localized result values as canonical table values.

## Presentation

- In user-facing summaries, call the AC Verification Matrix the acceptance
  checks or verification items in the artifact language. Keep the exact heading
  only when pointing to the document.
- Report reviewer output as the review result. Include `delegated` / `inline`
  only when it explains how the review ran or when the user asks.
- Summarize implementation as what changed, what was checked, and what remains
  for wrap-up; do not lead with `risk`, `status`, or reviewer mode.

## Stop conditions

- Do not implement when `status` is not `approved` or `spec.yaml` is missing.
- Stop when an out-of-scope change, AC change, or a new design decision is
  needed; return to the proper spec update flow.
- Stop on repeated verification failure, missing migration / rollback decision,
  or required human input.
- Never mark a task `[x]` before verification passes and the AC Matrix is updated.
- Do not change ACs during build without going back to the proper spec update flow.
- Do not commit while verification is FAIL. Do not complete build while a
  mandatory reviewer verdict is `fail`.
- If a commit fails after a task is marked `[x]`, stop and report the dirty
  worktree state; do not continue to the next task.
- `build` never sets `status: done`. Setting `done` is ship's responsibility,
  on the acceptance conditions in `reference/workflow.md ## AC Matrix`.
- Do not create the PR / move to `_done/` / request human checking outside the
  Matrix bookkeeping; final QA request is ship's responsibility.
