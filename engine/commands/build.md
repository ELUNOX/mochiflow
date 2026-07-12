---
name: spec-build
phase: build
description: |
  mochiflow's build phase. Implement an approved spec inline: execute task units
  on the main agent, verify, commit, maintain the integration log per risk, run
  read-only review through the change-reviewer transport, and produce the
  AC Verification Matrix. A micro spec may run with spec.yaml + spec.md only.
  Does not create PRs, set a terminal state, or move the spec (that is open).
artifacts:
  - "{specs_dir}/{slug}/pitch.md (when present)"
  - "{specs_dir}/{slug}/spec.md (AC Verification Matrix)"
  - "{specs_dir}/{slug}/tasks.md (when present)"
prerequisites:
  - "{specs_dir}/{slug}/spec.yaml exists with status approved (verify with `mochiflow ready {slug}`)"
execution: inline
delegate_to:
  - agents/change-reviewer.md
load:
  required:
    - reference/lifecycle.md
    - reference/verification.md
    - reference/risk.md
    - reference/git.md
    - reference/engineering-standards.md
  conditional:
    - when: risk >= elevated needs mandatory review, an ad-hoc review/fix runs, or any bounded-fix judgment is needed
      files:
        - reference/review.md
---

# mochiflow-build

After route selection, consult the `build` action from `mochiflow inspect <slug> --json`; `ready` remains the compatibility entry gate and this procedure owns execution.

## Purpose

Implement the approved spec and produce verification and the AC Verification Matrix. Do not create the PR or move the spec.

## Procedure

1. Confirm build eligibility with `mochiflow ready {slug}`: it runs `lint`, requires `status: approved`, and checks every surface's `default` verification command is runnable (not a `TODO:` placeholder). `default` is the canonical build-completion profile: it should be the reliable local command whose success is sufficient to say the surface is ready for PR / merge, except for checks explicitly documented as human-operated or CI-only. A non-zero exit is a stop condition — resolve it before implementing. Then read `spec.yaml` (risk / type / surfaces), `spec.md`, `pitch.md` when present, `design.md` / `tasks.md` when present, the constitution (`[constitution].project` / `[constitution].local`), the foundational context (`[context].product` / `[context].structure` / `[context].tech`) for orientation, and `[adr].pitfalls`. If `mochiflow ready` is unavailable, fall back to reading `spec.yaml` and confirming `status: approved` and runnable verification manually. This eligibility gate (`mochiflow ready` / `status: approved`) is the entry condition for **starting build as a phase**.
2. Define this build's **commit unit**: when `tasks.md` exists, the unit is one currently open task; when `tasks.md` is absent (taskless / micro specs), the unit is the whole logical change. Prepare the branch per `reference/git.md ## Branch`: verify the branch `{prefix}/{slug}` exists locally or on `origin`, switch to it, and error-stop if it does not exist. Verify the worktree has no changes other than this spec's own `{specs_dir}/{slug}/**` (else stop).
3. Run the task loop inline. When `tasks.md` exists, execute open tasks **one at a time** in dependency order on the main agent (no `[P]` parallelism unless the human explicitly approves a separate concurrency plan; one working tree). When `tasks.md` is absent, execute the whole logical change as a single unit. For each unit:
   - 3a. Read surrounding source before editing; for logic changes use TDD (RED→GREEN→REFACTOR), match existing style, and keep changes minimal. Per `reference/engineering-standards.md`, for any dependency / tool / framework-idiom change or any deviation, confirm the upstream-recommended approach from primary sources before implementing and record its source.
   - 3b. Append seam decisions / ownership / dead-code handling to `design.md ## Integration Log` only when `design.md` exists and the integration-log column in `reference/review.md ## Reviewer cadence` calls for it. For `standard`, do not create or require `design.md ## Integration Log`.
   - 3c. Run the canonical `default` command from `reference/verification.md ## Verification profiles` for build-completion evidence. Optional profiles such as `quick` / `targeted` may be used for intermediate feedback, but they do not replace `default`. Fix any FAIL and re-run to PASS.
   - 3d. Treat approved `tasks.md` structure as a plan contract. During build, `tasks.md` may be changed only to mark completed task checkboxes (`- [ ]` → `- [x]`) and to record AC Matrix result / evidence fields only in the legacy case where the matrix still lives in `tasks.md` (the canonical location is `spec.md ## Verification Plan / AC Matrix` per `reference/verification.md ## AC Matrix`). If implementation needs task additions, deletions, splits, renumbering, AC / NFR / chore reference changes, dependency changes, `Files:` changes, or meaningful `Done:` / `Stop:` changes, stop and route back to `plan` for re-approval instead of editing the task structure in build.
   - 3e. When the task's implementation and verification PASS, first mark that task as checked in `tasks.md` (`- [ ]` → `- [x]`) when `tasks.md` exists. Do not stage or commit while the completed task remains unchecked. Then commit per `reference/git.md ## Auto-commit`, using one `Task:` trailer for that task. Normal build commits do not combine multiple task completions; taskless / micro specs create one logical-unit build commit with no `Task:` trailer. Stage files explicitly.
   - 3f. Follow the reviewer cadence in `reference/review.md`; when required, run `agents/change-reviewer.md` read-only via `reference/review.md ## Review transport` (prefer delegated subagent when available; use inline reviewer role only when subagents are unavailable or dispatch fails for a runtime/tooling reason) and append the review profile, reviewer mode, verdict, and `Reviewed through: <sha>` to `design.md ## Review Results`. For `critical`, this happens after each task and reviews that task's own diff from git before the next task starts.
4. After all tasks complete, run final verification once more. Fix any FAIL and re-run to PASS.
5. For `elevated`, run the required `change-reviewer` once after all tasks using the same review transport. Record `Review profile: change-reviewer`, `Reviewer mode: delegated | inline`, `Verdict: pass | pass-with-comments | fail`, and `Reviewed through: <sha>` in `design.md ## Review Results`, with `Reviewed through` on its own line directly below `Verdict:`.
6. Record the AC Verification Matrix in `spec.md ## Verification Plan / AC Matrix` (its canonical location per `reference/verification.md ## AC Matrix`; a legacy matrix living at the end of `tasks.md` is updated in place). After final verification, settle automated AC as `PASS` / `FAIL` / `N/A: <reason>` before build completes. `UNVERIFIED` is allowed only as an in-progress placeholder before the final build record; do not leave automated rows `UNVERIFIED` at build completion. Record AC needing human/visual checking as `PENDING_HUMAN` without requesting that QA here (the request is made once, in open). Provisional tokens (`UNVERIFIED`, `PENDING_HUMAN`) are build-time placeholders only and are not done-eligible (`reference/verification.md ## AC Matrix`).
7. Include the final AC Verification Matrix update in the final build record commit for this phase, then stop. When implementation has already been committed by task commits, create a record-only commit with subject `docs(spec): record build verification`, the required `Spec: {slug}` trailer, and no `Task:` trailer. For taskless / micro specs, the single logical-unit build commit may include the implementation and final Matrix together when no earlier implementation commit exists. `open` only commits human QA results, final verification evidence appended by `mochiflow accept`, and fold/context changes as part of the close-out path.

## Post-completion bounded fixes before open

After all tasks (or the single logical-unit commit for taskless/micro specs) complete and before `open` runs, an in-scope request is applied and committed locally with no `Task:` trailer.
Use the shared bounded-fix judgment in `reference/review.md`: the request must require no task-structure change, no new AC, and no new design decision.
This post-completion fix is held; do not re-run the task loop, do not tick another checkbox, and do not run the mandatory reviewer at that moment.
The fresh review, when required for `risk ≥ elevated`, runs later at the next push/accept boundary described in `reference/review.md`.
An out-of-scope request still routes back to `plan` for re-approval.

`{slug} review fix [1-3]` after implementation uses `agents/change-reviewer.md`
and this same post-completion bounded-fix discipline: the main agent applies
in-scope fixes, verifies, commits without a `Task:` trailer, updates the local
review-fix ledger under `{install_dir}/state/{slug}/`, and holds the change for
the next open / accept boundary. A result-only review remains `{slug} review`
and does not edit files.

## Presentation

- In user-facing summaries, call the AC Verification Matrix the acceptance
  checks or verification items in the artifact language. Keep the exact heading
  only when pointing to the document.
- Report reviewer output as the review result. Include `delegated` / `inline`
  only when it explains how the review ran or when the user asks.
- When a build-adjacent choice card offers review actions, distinguish
  **Review results** (`review` / `mochiflow-review`) from **Review and fix**
  (`review fix`). Both map to `commands/review.md`; neither adds a delivery
  gate.
- Summarize implementation as what changed, what was checked, and what remains
  for wrap-up; do not lead with `risk`, `status`, or reviewer mode.
- On build completion, always include: (1) the verification result (all items
  passed, or human confirmation items remain), and (2) a numbered choice card:
  **Create the PR** (`open` / `mochiflow-open`) or
  **Create a resume prompt** (`resume` / `later`). The resume prompt is generated
  inline from the active slug and spec path and tells the next session to run
  `{slug} open`. Build ends at `status: approved`; it does not set a terminal
  state, create a PR, or move the spec.

## Resume from new session

When build resumes in a new session (no prior conversation state):

1. Read `tasks.md` checkboxes to identify completed (`- [x]`) and open (`- [ ]`)
   tasks.
2. Cross-check with git history:
   ```bash
   git log --grep="Spec: {slug}" \
     --format="%s | %(trailers:key=Task,valueonly)"
   ```
3. If trailers and checkboxes agree, resume from the first unchecked task.
   **When zero tasks are unchecked** (every task is committed) the per-task loop
   is already complete: do not look for a task to resume: proceed to the
   completion path (step 4 onward) — final verification, the `elevated`
   reviewer run if not yet recorded for the latest diff, the AC Matrix
   settlement, and the final build commit — performing only the steps not yet
   done.
4. If they disagree (a checked task lacks a matching `Task:` trailer in any
   commit, or a `Task:` trailer exists for an unchecked task), **stop and
   reconcile** before editing source files — read the relevant commits and
   `tasks.md` to determine the true state, fix `tasks.md` checkboxes to match
   reality, then resume.

## Stop conditions

- Do not implement when `status` is not `approved` or `spec.yaml` is missing (a non-zero `mochiflow ready {slug}` exit signals this).
- Stop when an out-of-scope change or a new design decision is needed.
- Do not finish build while verification or a required reviewer verdict is FAIL.
- `build` never sets `status: accepted`. Setting `accepted` is open's responsibility, on the acceptance conditions in `reference/verification.md ## AC Matrix`. At build's end the status stays `approved`.
- Do not create the PR / set a terminal state / move the spec / request human checking (those are open's responsibility).
