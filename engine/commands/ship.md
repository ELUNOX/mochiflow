---
name: spec-ship
phase: ship
description: |
  mochiflow's ship phase. Drive final traceability checks, QA evidence,
  close-out commit, PR metadata, and PR handoff. Fold durable knowledge and
  archive before PR on the feature branch; post-merge cleanup is local hygiene
  only. Activate on the explicit command `mochiflow-ship`, or natural phrasing
  like "PR出して" / "リリースして"; the human merge report "{slug} merged" /
  "{slug} マージ済み" / "{slug} 完了" resumes post-merge local cleanup only.
triggers:
  - mochiflow-ship
  - PR出して
  - リリースして
trigger_patterns:
  - "{slug} ship"
  - "{slug} merged"
  - "{slug} マージ済み"
  - "{slug} 完了"
artifacts:
  - "{specs_dir}/{slug}/qa-instructions.md"
  - "{install_dir}/state/{slug}/pr-body.md"
  - "{install_dir}/state/{slug}/pr-request.json (pr_driver backend only)"
  - "{specs_dir}/_done/{slug}/"
prerequisites:
  - Implementation and verification complete (AC Matrix exists)
execution: inline
allowed_writes:
  - "{specs_dir}/**"
  - "{install_dir}/state/**"
  - "{adr.decisions}"
  - "{adr.pitfalls}"
  - "{index}"
forbidden_writes:
  - "{write.allow}"
  - .git/**
references:
  - reference/workflow.md
  - reference/git.md
  - templates/delivery/qa-instructions.md
  - templates/delivery/pr-description.md
---

# mochiflow-ship

## Purpose

Complete traceability verification, human QA when needed, PR preparation, the
living-spec fold, and archive.

## Procedure

### Acceptance

1. Confirm the AC Matrix exists in `spec.md`, and every AC has at least one row.
2. Confirm required tasks in `tasks.md` are `[x]`. Required finalization tasks
   are `[x]` or explicitly not applicable with reason.
3. Confirm `design.md` is present when required by risk / integration /
   surfaces / migration / contract / security / privacy / performance /
   accessibility / reviewer policy.
4. Confirm migration and rollback notes are present when required.
5. Build `qa-instructions.md` into `{specs_dir}/{slug}/` from the QA scenarios
   in `spec.md` (reference, do not copy). Pick the adapter via
   `reference/workflow.md ## Acceptance adapters`.
6. Run the final verification command and record the result.
7. Request QA that needs human operation / visual checking here exactly once.
   The human follows `qa-instructions.md`; record results and evidence in the
   AC Matrix.
8. Block ship if any AC Matrix row has:
   - `UNVERIFIED`
   - `PENDING_HUMAN`
   - `FAIL`
   - `N/A` without the required `N/A: <reason>` form
9. Block ship if required QA evidence is missing, or if required reviewer results
   are missing. High/Critical reviewer findings must be resolved or explicitly
   blocked by existing policy before ship.
10. When all acceptance conditions in `reference/workflow.md ## AC Matrix` hold,
    set `spec.yaml` `status: done` / `updated` mechanically. This is not a gate
    and uses no approval word; `ship` is the only path that sets `done`.

### Close-out

11. Fold, archive, and close out on the feature branch before `mochiflow pr`.
    - Fold per `reference/git.md ## Living-spec fold`: append why/history that
      code cannot reproduce to `[adr].decisions` and operational pitfalls to
      `[adr].pitfalls`. Do not fold current-state prose.
    - If the change introduced a coarse structural shift that makes foundational
      context stale, prompt the human to run `refresh-context`; do not regenerate
      or commit context automatically here.
    - Archive `{specs_dir}/{slug}/` (including `qa-instructions.md`) to
      `{specs_dir}/_done/{slug}/` and regenerate `{index}` with `mochiflow index`.
    - Make the single close-out commit per `reference/git.md ## Auto-commit and staging ### Ship close-out commit`.

### PR

12. Generate the PR title / description per `templates/delivery/pr-description.md`
    after archive, write the body to `{install_dir}/state/{slug}/pr-body.md`,
    present it, and wait for human approval (delivery approval gate 2).
13. After approval, run
    `mochiflow pr --spec {slug} --title "<title>" --body-file {install_dir}/state/{slug}/pr-body.md`
    (add `--draft` if applicable). Read its exit code:
    - `0` — PR created; capture the printed URL.
    - `10` — manual handoff; the branch is pushed and the human creates the PR.
    - `3` — pre-flight failed; fix and re-run.
    - `1`/`2` — backend / config failure; stop and diagnose.
    Do not call `az` / `gh` / `git push` directly.

## PR Feedback Loop

If PR feedback, CI failure, reviewer comments, or PR-body approval follow-up
requires code changes before merge:

1. Do not use `patch` unless the change is unrelated to the shipped spec.
2. Move `{specs_dir}/_done/{slug}/` back to `{specs_dir}/{slug}/`.
3. Set `spec.yaml` status from `done` back to `approved` and update `updated`.
4. Apply the requested changes through `build`.
5. Re-run verification and update the AC Verification Matrix.
6. Re-run `ship` close-out: set `done`, archive again, regenerate `INDEX`, and
   update the PR body when needed.

### Post-merge

14. After the human reports the merge, run
    `reference/git.md ## Post-merge local cleanup`: switch to base, pull
    fast-forward only, safe-delete the local branch, and remove
    `{install_dir}/state/{slug}/`. The fold + archive are already merged via the
    close-out commit; do not commit or push anything to the base branch here.
    Knowledge discovered at or after merge is routed to a follow-up, never
    appended to the archived spec.

## Presentation

- In user-facing speech, describe ship as wrap-up / PR preparation in the
  conversation language. Use `ship` only for the command or when the user uses it.
- Describe fold as recording durable learnings, archive as moving work to
  completed, and `status: done` as marking the work complete.
- Describe the AC Matrix as the acceptance checks or verification items, and the
  reviewer verdict as the review result. Keep exact internal terms only when
  pointing to file headings or CLI commands.

## Stop conditions

- Do not proceed to ship while implementation, required tasks, review,
  verification, or AC Matrix evidence are incomplete.
- Do not proceed if any AC Matrix result is `UNVERIFIED`, `PENDING_HUMAN`, or
  `FAIL`, or if any `N/A` result lacks a reason.
- Do not run `mochiflow pr` before human approval of the PR content.
- Do not force past a pre-flight FAIL (`mochiflow pr` exit 3); fix and re-run.
- Do not call `git push` / `gh` / `az` directly; `mochiflow pr` owns push and creation.
- Do not build the close-out commit before `status: done` holds (acceptance conditions met). Skip the fold only when there is genuinely no new rationale or pitfall; archive (the `_done` move + `INDEX`) still happens.
- Do not commit or push anything to the base branch during post-merge cleanup — the fold + archive are already in the PR; post-merge is local hygiene only.
- The no-PR fast path makes the same close-out commit (`status: done` + AC matrix + fold + archive + `INDEX`) on the current branch and creates no PR. `ship` still sets `status: done` on the acceptance conditions (step 4) — there is no path where `done` is set outside ship.
- Before merge, route PR feedback / CI fixes for this shipped spec through
  `## PR Feedback Loop`, not `patch`.
