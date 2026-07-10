---
name: spec-update
phase: update
description: |
  mochiflow's update action. Handle review feedback, CI failures, and PR-body
  corrections while the work is in review. Code changes are applied as bounded
  inline fixes using build discipline (not a separate phase restart). Update is
  hold-by-default for bare feedback: it re-verifies, commits locally, and holds;
  an explicit finalize signal reviews-if-stale once, pushes, updates PR
  metadata, and revises the fold when feedback changes a decision. The spec is
  never moved and never resurrected — it has stayed flat the whole time.
artifacts:
  - "{install_dir}/state/{slug}/pr-body.md"
  - "{specs_dir}/{slug}/ (flat; never moved)"
prerequisites:
  - "A PR is open for the spec (derived `in_review`); spec is flat at `{specs_dir}/{slug}/`"
execution: inline
load:
  required:
    - reference/delivery.md
    - reference/verification.md
    - reference/git.md
  conditional:
    - when: finalize on risk >= elevated re-reviews a stale verdict, a review/fix runs, or any bounded-fix judgment is needed
      files:
        - reference/review.md
    - when: PR metadata changes and the title/body is regenerated
      files:
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
2. Classify the input as hold-only or finalize:
   - Hold-only feedback signals are the existing bare natural-language triggers
     with no slug qualifier: `修正依頼`, `PR feedback`, `PRを直して`, and
     equivalent in-scope requests. Use the shared bounded-fix judgment in `reference/review.md` for whether a request is in scope.
     For hold-only input, apply the requested code
     changes as a **bounded inline PR-feedback fix** using build discipline
     (read, edit, TDD where applicable, verify, commit). This is not an open
     `tasks.md` task (build is already complete): there is no checkbox to tick
     and no `Task:` trailer. Build's eligibility gate (`mochiflow ready` /
     `status: approved`) is **not** re-run — the spec is already `accepted` and
     in review, and that state is reused as-is (never
   reverted to `approved`).
     For hold-only input: apply, commit locally, re-verify, update the touched AC Matrix rows, then stop.
     No push, no `mochiflow accept`, and no mandatory reviewer run happen on a hold-only message.
   - Finalize signals are an explicit finalize invocation, the slug-qualified
     update event selected by `router.md`, or an unambiguous completion statement
     such as "that's everything / push it / update the PR". A finalize input
     runs the review-if-stale, push, and PR-metadata
     steps below over every held commit since the last recorded
     `Reviewed through` sha, not just the latest commit.
   Feedback interpretation and PR-metadata updates stay inline on the main
   agent.
3. When feedback changes a decision or surfaces a new pitfall, revise the fold
   (`[adr].decisions` / `[adr].pitfalls`) so the durable record keeps matching
   the final design.
4. On finalize, if there are no held commits and no PR metadata changes to
   publish, report that the PR is already up to date and stop without review or
   push.
5. On finalize for `risk ≥ elevated`, when no code-changing commit exists beyond
   the last recorded `Reviewed through` sha, do not re-run the reviewer (for
   example, a PR-body-only correction does not require a new review). When one
   or more code-changing commits exist beyond that sha, re-run
   `agents/change-reviewer.md` exactly once on the full diff from git and record
   the fresh verdict plus updated `Reviewed through: <sha>` in
   `design.md ## Review Results` before pushing, per
   `reference/review.md ## Verdict freshness`.
6. On finalize, push the branch so the open PR updates.
7. Update PR metadata when needed: regenerate the PR title/body into
   `{install_dir}/state/{slug}/pr-body.md` and update the PR via the provider
   (or re-run `mochiflow pr` for the configured backend). PR-body-only
   corrections skip the code loop.
8. Each spec-lane commit step regenerates the board via `mochiflow index` so the
   gitignored `INDEX.md` stays fresh; `INDEX.md` is never staged or committed.

`{slug} review fix [1-3]` while the spec is in review uses
`agents/change-reviewer.md` and update's bounded inline PR-feedback fix
discipline. The main agent applies in-scope fixes, verifies, commits locally,
updates the local review-fix ledger under `{install_dir}/state/{slug}/`, and
holds by default. It does not push, update PR metadata, or run
`mochiflow accept` unless the active update flow later receives an explicit
finalize signal. Result-only `{slug} review` reports findings without editing.

## Presentation

- Describe update as addressing PR feedback / revising the open PR in the
  conversation language. Use `update` only for the command or when the user uses
  it.
- For hold-only turns, report what changed, what was re-verified, and that the
  commit is held locally and not yet pushed or reviewed.
- For finalize turns, report what was reviewed if stale, what was pushed, and
  what PR metadata changed.
- When an update-adjacent choice card offers review actions, distinguish
  **Review results** (`review` / `mochiflow-review`) from **Review and fix**
  (`review fix`). Both map to `commands/review.md`; neither adds a delivery
  gate, and review-and-fix keeps update hold-by-default until finalize.

## Stop conditions

- Do not move the spec directory and do not revert the `accepted` asserted state.
- Do not restart the build phase inside update; apply only the bounded inline
  fix needed for the PR feedback.
- Do not push automatically on a hold-only message, and do not guess finalize
  from an ambiguous reply — ask one clarifying question instead.
- Do not write `status: done`, never archive to `_done/`, and never stage or
  commit `INDEX.md`.
- If the feedback is unrelated to this spec, route it as its own spec instead.
