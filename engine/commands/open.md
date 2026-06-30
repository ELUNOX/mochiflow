---
name: spec-open
phase: open
description: |
  mochiflow's open action. Run final acceptance (the human QA round-trip),
  finalize the living-spec fold, set `accepted` and create the single
  feature-branch close-out commit, generate the PR title/body, take the approve-PR
  gate, then push and create the PR. The spec stays flat at
  `{specs_dir}/{slug}/` — there is no `_done/` move and no `done` write. Activate
  on the explicit command `mochiflow-open`, or natural phrasing like "PR出して" /
  "PRを作って".
triggers:
  - mochiflow-open
  - PR出して
  - PRを作って
trigger_patterns:
  - "{slug} open"
artifacts:
  - "{install_dir}/state/{slug}/pr-body.md"
  - "{install_dir}/state/{slug}/pr-request.json (pr_driver backend only)"
  - "{specs_dir}/{slug}/ (flat; never moved)"
prerequisites:
  - "Implementation and verification complete (AC Matrix exists); `status: approved`"
execution: inline
references:
  - reference/workflow.md
  - reference/risk.md
  - reference/git.md
  - templates/delivery/pr-description.md
---

# mochiflow-open

## Purpose

Take a built spec from `approved` to an open PR: final acceptance, the
living-spec fold, the `accepted` close-out commit, and PR creation. The spec is
never archived and never marked `done` — delivery state (`in_review`, `merged`)
is derived from VCS/provider, never written.

## Procedure

`open` performs these steps in order (a–g): acceptance → fold + context-check →
optional `docs(context)` commit → accept close-out → PR title/body → approve-PR →
`mochiflow pr`. The PR is **never created before the approve-PR gate (f)**.

### (a) Acceptance

1. Run the final verification command for each surface and record the result in
   the AC Verification Matrix. Settle automated AC rows as `PASS` or `FAIL`.
2. Identify QA items that need human operation or visual checking from `spec.md`
   QA Scenarios (rows where `Type` is `Human-operated` or `Visual`). If no such
   items exist, skip to (b).
3. **QA round-trip protocol** — present and collect human QA results:
   - 3a. Present the human-required QA items as a numbered list in the
     conversation language, derived from spec.md QA Scenarios (scenario name,
     steps, expected result). Number items sequentially.
   - 3b. The human responds in conversation language (free-form intent). Accepted
     forms include per-item responses ("1: OK, 2: NG reason") or batch
     confirmation ("all OK"). These are examples, not a fixed vocabulary — the
     agent interprets intent, not pattern-matches tokens.
   - 3c. For each item, map the human's intent to the canonical AC Matrix token:
     pass intent → `CONFIRMED`, fail intent → `FAIL`, not-applicable with
     reason → `N/A: <reason>`. Record the token and evidence pointer in the
     AC Verification Matrix.
   - 3d. If any item is ambiguous (cannot determine pass or fail intent), re-ask
     for that specific item with a clear pass/fail question. Do not guess.
   - 3e. **Rework loop**: if any item is `FAIL`, pause open (status stays
     `approved`). Apply a **bounded inline code fix** on the feature branch using
     the build discipline (read, modify, verify, commit), but do not re-run
     build's phase-entry gate (`mochiflow ready`) and do not revert the asserted
     state. The fix is not an open `tasks.md` task (build is already complete):
     there is no checkbox to tick and no `Task:` trailer. Commit per this verb's
     rework-commit convention (`reference/git.md`). Acceptance judgment, the
     fold, PR-body synthesis, and the approve-PR gate stay inline on the main
     agent. If the spec is `risk ≥ elevated`, a QA-`FAIL` rework that changes
     code makes any prior reviewer verdict **stale**: re-run
     `agents/independent-reviewer.md` on the new diff and record the fresh
     verdict before accept, per `reference/risk.md ## Consequences` (verdict
     freshness). After the fix, re-present: (1) the failed items, plus
     (2) any previously-passed items whose implementation files were modified by
     the fix (regression check). Repeat from 3b for the re-presented items only.
   - 3f. When all human QA items reach a done-eligible result (`CONFIRMED` or
     `N/A: <reason>`), the round-trip is complete.

### (b) Finalize the fold

4. `open` owns authoring the fold (not `accept`). Fold per
   `reference/git.md ## Living-spec fold`: append the *why* that code cannot
   reproduce (decision rationale, rejected options) as a **new per-file record**
   under `[adr].decisions` (`{YYYY-MM-DD}-{slug}.md` with front-matter `id` /
   `date` / `area` / `spec: {slug}` / `status: active`), and operational
   pitfalls as a new record under `[adr].pitfalls` using the active guardrail
   format. `area` defaults to the spec's `surfaces`; every close-out record must
   include `spec: {slug}` so `mochiflow accept` can stage only this spec's fold.
   When a new decision overrides an earlier one, add the new record with
   `supersedes: <id>` and flip the superseded
   record to `status: superseded` with the reciprocal `superseded_by: <id>`
   (never rewrite the old record's body). Regenerate each affected store's
   gitignored `INDEX.md` (never stage it). Pitfalls captured during build are
   finalized here. Do not fold prose that describes current state. Skip only
   when there is genuinely no new rationale or pitfall.
   - **Foundational context refresh check (not a fold)**: if the change
     introduced a coarse structural shift (new module / surface / moved entry
     point / technology or verification responsibility) that makes
     `[context].product` / `[context].structure` / `[context].tech` stale, run
     `refresh-context` (`commands/refresh-context.md`) **on the feature branch
     now**, under human confirmation that the regenerated context matches current
     code. `refresh-context` regenerates the files but does **not** auto-commit;
     `open` (not `refresh-context`) owns the `git add` of the `[context]` paths
     and the separate `docs(context)` commit created in step (c) below — placed
     after this check and **before** the `mochiflow accept` close-out commit, so
     the refresh ships inside the PR and `mochiflow pr` pre-flight still sees a
     clean tree. If the human does not confirm current-state accuracy, commit
     nothing and record a post-merge follow-up instead. Staleness discovered only
     **at or after merge** is the fallback case: route it to a post-merge
     follow-up (a `fix` spec or a backlog seed per
     `reference/git.md ## Living-spec fold`), never a base-branch edit. The
     context layer is refreshed from code under human confirmation, never folded.

### (c) Context refresh commit (optional)

5. If step 4's context-refresh check ran `refresh-context` and the human
   confirmed the regenerated context, `open` stages the `[context]` paths
   (`git add` of `[context].product` / `[context].structure` / `[context].tech`)
   and creates a separate `docs(context): ...` commit on the feature branch with
   the `Spec: {slug}` trailer, per
   `reference/git.md ## Auto-commit and staging`. This commit is positioned
   **after** the fold/context-check and **before** the accept close-out commit
   (step 6), so the accept close-out stays the single final state commit and the
   working tree is clean for `mochiflow pr`. When no structural shift was
   detected, or the human did not confirm, there is no `docs(context)` commit and
   this step is skipped.

### (d) Accept close-out commit

6. When the acceptance conditions in
   `reference/workflow.md ## AC Verification Matrix` all hold (matrix complete,
   every result done-eligible, and the reviewer verdict recorded when
   `risk ≥ elevated`), run `mochiflow accept {slug}`. The command re-runs final
   verification, appends final verification evidence to already-`PASS`
   automated AC Matrix rows, sets `spec.yaml` `status: accepted` and `updated`,
   runs `lint`, stages the target spec (`{specs_dir}/{slug}/**`) and
   already-written ADR record files linked to this slug, and creates the single
   feature-branch close-out commit. `mochiflow accept` does not convert
   `UNVERIFIED` to `PASS`; resolve provisional rows before this step. The spec stays flat: there is **no
   `_done/` move, no `done` write, and no committed `INDEX.md`** (the board is
   gitignored and refreshed by the shared post-command step). Use
   `mochiflow accept --dry-run` first to inspect blockers and planned paths.
   - Manual fallback only when the CLI command is unavailable: after setting
     `status: accepted` (no `completed`, no `_done` move, no `INDEX` write),
     stage with `git add {specs_dir}/{slug} {adr_record_paths...}` and validate
     with `git diff --cached --name-status -z` before committing. Never stage
     `INDEX.md`.
   - The result is the **single close-out commit** per
     `reference/git.md ## Auto-commit and staging ### Accept close-out commit`,
     with an external-reviewer message (no spec slug, no AC IDs, no mochiflow
     vocabulary). Nothing is pushed to the base branch here.

### (e) Generate PR title/body

7. On the normal PR path, generate the PR title / description per
   `templates/delivery/pr-description.md` (the spec lives flat under
   `{specs_dir}/{slug}/`), write the body to
   `{install_dir}/state/{slug}/pr-body.md` (ephemeral, gitignored — **never** the
   spec dir).

### (f) Approve-PR gate

8. Present the PR title/body and wait for human approval (gate 2). Present the
   approval action as **Create the PR** (`create pr` / `approved`) in a numbered
   choice card. If the user gives PR text corrections instead, revise the
   title/body and re-present the approval card. This is the only human gate in
   `open` — there is no second gate beyond approve-PR.

### (g) Push and create the PR

9. After the **Create the PR** approval action, run
   `mochiflow pr --spec {slug} --title "<title>" --body-file {install_dir}/state/{slug}/pr-body.md`
   (add `--draft` if applicable). `open` is the sole producer of the body file;
   `mochiflow pr` only reads it (and writes `pr-request.json` under
   `state/{slug}/` for the `pr_driver` backend only). The working tree is clean
   because the close-out commit (step 6) captured every tracked change (and any
   context refresh was already committed in step 5). The CLI runs pre-flight (clean tree / branch / base≠head and the `accepted`+`Spec:`
   trailer spec check), pushes the branch, and resolves the backend per
   `reference/git.md ## PR`. Read its exit code:
   - `0` — PR created; capture the printed URL. The spec is now derived
     `in_review`. Present the PR URL and the conversational post-merge next
     action (below).
   - `10` — manual handoff: the branch is pushed; create the PR with the
     presented content via your provider UI/CLI, then report the URL. Present
     the same conversational post-merge next action (below).
   - `3` — pre-flight failed; fix and re-run.
   - `1`/`2` — backend / config failure; stop and diagnose.

   **PR-created conversational handoff.** On a successful handoff (exit `0` or
   `10`), the agent's final PR-created response MUST tell the user, in
   conversation-language plain wording, to merge the PR in the provider UI, then
   return to chat and report that it merged so post-merge local cleanup can run.
   Include the PR URL when one is available (exit `0`); on a URL-less manual
   handoff (exit `10`) or any backend that returned no URL, describe the pushed
   branch and handoff instead of a URL and still include the same
   merge-then-report next action. This next action is local workflow guidance and
   is never written into the PR body (which stays artifact-language,
   external-reviewer facing).

## Presentation

- In user-facing speech, describe open as creating the PR / preparing delivery
  in the conversation language. Use `open` only for the command or when the user
  uses it.
- Describe the fold as recording durable learnings and `status: accepted` as the
  work being accepted (quality-complete), not done — "done" is observed from the
  merge, never written.
- Describe the AC Matrix as the acceptance checks or verification items, and the
  reviewer verdict as the review result.
- After PR creation, state the next human action conversationally: merge the PR
  in the provider, then return to chat and report the merge so local cleanup can
  run. Keep this next action out of the PR body.

## Stop conditions

- Do not proceed to open while implementation and verification are incomplete.
- Do not run `mochiflow pr` before human approval of the PR content (gate 2).
- Do not proceed if AC Matrix evidence is incomplete, any required task/review is
  missing, any result is `UNVERIFIED`, `PENDING_HUMAN`, or `FAIL`, or any
  not-applicable result lacks a reason.
- Never write `status: done`, never move the spec into `_done/`, and never stage
  or commit `INDEX.md`.
- On a pre-flight FAIL (`mochiflow pr` exit 3), fix the reported issue and re-run.
- After the PR is open, route feedback / CI fixes through `commands/update.md`,
  not a fresh open and not `patch`.
