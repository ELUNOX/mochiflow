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
  - 修正依頼
  - PR feedback
trigger_patterns:
  - "{slug} ship"
  - "{slug} merged"
  - "{slug} マージ済み"
  - "{slug} 完了"
  - "{slug} feedback"
  - "{slug} 修正依頼"
  - "{slug} PR feedback"
artifacts:
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
  - templates/delivery/pr-description.md
---

# mochiflow-ship

## Purpose

Complete traceability verification, human QA when needed, PR preparation, the
living-spec fold, and archive.

## Procedure

### Acceptance

1. Run the final verification command for each surface and record the result in
   the AC Verification Matrix. Settle automated AC rows as `PASS` or `FAIL`.
2. Identify QA items that need human operation or visual checking from `spec.md`
   QA Scenarios (rows where `Type` is `Human-operated` or `Visual`). If no such
   items exist, skip to step 4.
3. **QA round-trip protocol** — present and collect human QA results:
   - 3a. Present the human-required QA items as a numbered list in the
     conversation language, derived from spec.md QA Scenarios (scenario name,
     steps, expected result). Number items sequentially.
   - 3b. The human responds in conversation language (free-form intent). Accepted
     forms include per-item responses ("1: OK, 2: NG reason") or batch
     confirmation ("all OK"). These are examples, not a fixed vocabulary — the
     agent interprets intent, not pattern-matches tokens.
   - 3c. For each item, map the human's intent to the canonical AC Matrix token:
     pass intent → `人間確認済み`, fail intent → `FAIL`, not-applicable with
     reason → `対象外（<reason>）`. Record the token and evidence pointer in the
     AC Verification Matrix.
   - 3d. If any item is ambiguous (cannot determine pass or fail intent), re-ask
     for that specific item with a clear pass/fail question. Do not guess.
   - 3e. **Rework loop**: if any item is `FAIL`, pause ship (status stays
     `approved`). Run a build-equivalent fix loop (modify → verify → commit on
     the feature branch). After the fix, re-present: (1) the failed items, plus
     (2) any previously-passed items whose implementation files were modified by
     the fix (regression check). Repeat from 3b for the re-presented items only.
   - 3f. When all human QA items reach a done-eligible result (`人間確認済み` or
     `対象外（<reason>）`), the round-trip is complete. Proceed to step 4.
4. When the acceptance conditions in `reference/workflow.md ## AC Verification Matrix` all hold (matrix complete, every result is done-eligible, and the reviewer verdict recorded when `risk ≥ elevated`), edit `spec.yaml` `status: done`, `updated`, and `completed` (the current UTC timestamp in ISO 8601, e.g. `2026-06-21T22:16:03Z`) directly (no approval word; there is no CLI transition command), then run `mochiflow lint --spec {slug}` to confirm the transition is valid. `completed` is the immutable completion time that orders the Done view in `INDEX.md`; set it (or overwrite it on a re-ship) each time status becomes `done`. This is not a gate; `ship` is the only path that sets `done`.

### Close-out

5. Fold, archive, and close out — on the feature branch, before `mochiflow pr`.
   - Fold per `reference/git.md ## Living-spec fold`: append the *why* that code cannot reproduce (decision rationale, rejected options) to `[adr].decisions` with a date, and operational pitfalls to `[adr].pitfalls` using the active guardrail format. Do not fold prose that describes current state. Skip when there is no new rationale or pitfall.
   - **Foundational context refresh check (not a fold)**: if the change introduced a coarse structural shift (new module / surface / moved entry point / technology or verification responsibility) that makes `[context].product` / `[context].structure` / `[context].tech` stale, record/report a post-ship `refresh-context` follow-up after PR creation or after merge. Do **not** run or trigger `refresh-context` before the close-out commit or `mochiflow pr`; it writes context files and does not auto-commit, which would dirty the tree before PR pre-flight. The context layer is refreshed from code under human confirmation, never folded.
   - Archive: move `{specs_dir}/{slug}/` → `{specs_dir}/_done/{slug}/` with `git mv` (so the rename stages as a paired delete + add; nothing to stage when specs are gitignored) and regenerate `{index}` (`mochiflow index`).
   - Make the **single close-out commit** per `reference/git.md ## Auto-commit and staging ### Ship close-out commit`: stage exactly `status: done` + the AC Verification Matrix + the fold (`[adr]`) + the `_done/{slug}/` move + `{index}`, with an external-reviewer message (no spec slug, no AC IDs, no mochiflow vocabulary). Nothing is pushed to the base branch here.

### PR

6. On the normal PR path, generate the PR title / description per `templates/delivery/pr-description.md` (the spec now lives under `_done/{slug}/`), write the body to `{install_dir}/state/{slug}/pr-body.md` (ephemeral, gitignored — **never** the spec dir), present it, and wait for human approval (gate 2). The PR title/body are always produced on the PR path — the automatable, provider-independent part. On the explicit no-PR fast path, skip this PR section after the close-out commit.
7. After approval on the PR path, run `mochiflow pr --spec {slug} --title "<title>" --body-file {install_dir}/state/{slug}/pr-body.md` (add `--draft` if applicable). ship is the sole producer of the body file; `mochiflow pr` only reads it (and writes `pr-request.json` under `state/{slug}/` for the `pr_driver` backend only). The working tree is clean because the close-out commit (step 5) already captured every tracked change. The CLI owns pre-flight (clean tree / branch / base≠head), the single `git push`, and backend resolution per `reference/git.md ## PR` (`pr_driver` > `provider` built-in > legacy `pr_command` > manual). Read its exit code:
   - `0` — PR created; capture the printed URL.
   - `10` — manual handoff: the branch is pushed; create the PR with the presented content via your provider UI/CLI, then report the URL / merge.
   - `3` — pre-flight failed; fix and re-run. Do not force past it.
   - `1`/`2` — backend / config failure; stop and diagnose.
   Do not call `az` / `gh` / `git push` directly — `mochiflow pr` is the only path.

## PR Feedback Loop

If PR feedback, CI failure, reviewer comments, or PR-body approval follow-up
requires code changes before merge:

1. Do not use `patch` unless the change is unrelated to the shipped spec.
2. Move `{specs_dir}/_done/{slug}/` back to `{specs_dir}/{slug}/`.
3. Set `spec.yaml` status from `done` back to `approved` and update `updated`.
4. Treat this restore as a related lifecycle change for the same shipped spec:
   only `{specs_dir}/{slug}/**` and `{specs_dir}/_done/{slug}/**` may be dirty
   when build resumes from this PR Feedback Loop. Any other dirt still stops.
5. Apply the requested changes through `build`.
6. Re-run verification and update the AC Verification Matrix.
7. Re-run `ship` close-out: set `done` (re-stamping `completed` with the new
   completion time), archive again, regenerate `INDEX`, and update the PR body
   when needed.

### Post-merge

8. After the human reports the merge, run
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

- Do not proceed to ship while implementation and verification are incomplete.
- On the PR path, do not run `mochiflow pr` before human approval of the PR content (gate 2). On the explicit no-PR fast path, do not run `mochiflow pr`.
- Do not proceed if AC Matrix evidence is incomplete, any required task/review is missing, any result is `PENDING_HUMAN` or `FAIL`, or any not-applicable result lacks a reason.
- Do not force past a pre-flight FAIL (`mochiflow pr` exit 3); fix and re-run.
- Do not call `git push` / `gh` / `az` directly; `mochiflow pr` owns push and creation.
- Do not build the close-out commit before `status: done` holds (acceptance conditions met). Skip the fold only when there is genuinely no new rationale or pitfall; archive (the `_done` move + `INDEX`) still happens.
- Do not commit or push anything to the base branch during post-merge cleanup — the fold + archive are already in the PR; post-merge is local hygiene only.
- The no-PR fast path makes the same close-out commit (`status: done` + AC matrix + fold + archive + `INDEX`) on the current branch and creates no PR. `ship` still sets `status: done` on the acceptance conditions (step 4) — there is no path where `done` is set outside ship.
- Before merge, route PR feedback / CI fixes for this shipped spec through
  `## PR Feedback Loop`, not `patch`.
