---
name: spec-update
phase: update
description: |
  mochiflow's update action. Handle review feedback, CI failures, and PR-body
  corrections while the work is in review. Code changes are delegated through the
  same `build` loop (not reimplemented); update re-verifies, pushes, updates PR
  metadata, and revises the fold when feedback changes a decision. The spec is
  never moved and never resurrected — it has stayed flat the whole time. Activate
  on the explicit command `mochiflow-update`, or natural phrasing like "修正依頼" /
  "PR feedback" / "PRを直して".
triggers:
  - mochiflow-update
  - 修正依頼
  - PR feedback
  - PRを直して
trigger_patterns:
  - "{slug} update"
  - "{slug} feedback"
  - "{slug} 修正依頼"
  - "{slug} PR feedback"
artifacts:
  - "{install_dir}/state/{slug}/pr-body.md"
  - "{specs_dir}/{slug}/ (flat; never moved)"
prerequisites:
  - "A PR is open for the spec (derived `in_review`); spec is flat at `{specs_dir}/{slug}/`"
execution: inline
references:
  - reference/workflow.md
  - reference/git.md
  - templates/delivery/pr-description.md
---

# mochiflow-update

## Purpose

Address PR feedback, CI failures, and reviewer comments on an open PR without
moving or resurrecting the spec. The spec lives flat at `{specs_dir}/{slug}/`
for its whole life, so there is nothing to restore.

## Procedure

1. Confirm the spec is the in-review spec for the open PR (resolve by slug or by
   the current `{prefix}/{slug}` branch). The spec is already flat at
   `{specs_dir}/{slug}/`; **do not move it, and do not revert its asserted
   state** (it stays `accepted`).
2. **Delegate code changes to the `build` loop** — do not reimplement build
   logic here. Apply the requested changes **through the build worker mechanism**
   (`commands/build.md` — delegated worker when available, else inline; read,
   edit, TDD where applicable); update defines no separate delegation path. The
   reused worker's unit is the **bounded PR-feedback fix**, not an open
   `tasks.md` task (build is already complete): there is no checkbox to tick and
   no `Task:` trailer, and the worker commits per this verb's feedback-commit
   convention (step 4 / `reference/git.md`). build's eligibility gate
   (`mochiflow ready` / `status: approved`) is **not** re-run — the spec is
   already `accepted` and in review, and that state is reused as-is (never
   reverted to `approved`). Re-verify with the surface's `default` command, and
   update the AC Verification Matrix rows touched by the change. Feedback
   interpretation and PR-metadata updates stay inline on the main agent.
3. When feedback changes a decision or surfaces a new pitfall, revise the fold
   (`[adr].decisions` / `[adr].pitfalls`) so the durable record keeps matching
   the final design.
4. Commit the changes on the feature branch (one task-sized commit per
   `reference/git.md`), then push the branch so the open PR updates.
5. Update PR metadata when needed: regenerate the PR title/body into
   `{install_dir}/state/{slug}/pr-body.md` and update the PR via the provider
   (or re-run `mochiflow pr` for the configured backend). PR-body-only
   corrections skip the code loop.
6. Each spec-lane commit step regenerates the board via `mochiflow index` so the
   gitignored `INDEX.md` stays fresh; `INDEX.md` is never staged or committed.

## Presentation

- Describe update as addressing PR feedback / revising the open PR in the
  conversation language. Use `update` only for the command or when the user uses
  it.
- Report what changed, what was re-verified, and what was pushed.

## Stop conditions

- Do not move the spec directory and do not revert the `accepted` asserted state.
- Do not reimplement build logic inside update — delegate to `build`.
- Do not write `status: done`, never archive to `_done/`, and never stage or
  commit `INDEX.md`.
- If the feedback is unrelated to this spec, route it as its own spec or `patch`
  instead.
