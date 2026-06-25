---
name: spec-build
phase: build
description: |
  mochiflow's build phase. Implement an approved spec inline (read, write,
  verify, self-review), maintain the integration log per risk, run only
  read-only review through independent-reviewer transport, commit, and produce the AC
  Verification Matrix. A trivial standard-risk spec may run with spec.md only
  and a no-PR fast path branch choice. Activate on the explicit
  command `mochiflow-build`, or natural phrasing like "実装して" / "進めて" /
  "ビルドして". Does not create PRs, set `done`, or archive (that is ship).
triggers:
  - mochiflow-build
  - 実装して
  - 進めて
  - ビルドして
trigger_patterns:
  - "{slug} build"
artifacts:
  - "{specs_dir}/{slug}/pitch.md"
  - "{specs_dir}/{slug}/spec.md (AC Verification Matrix)"
  - "{specs_dir}/{slug}/tasks.md (when present)"
prerequisites:
  - "{specs_dir}/{slug}/spec.yaml exists with status approved (verify with `mochiflow ready {slug}`)"
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

Implement the approved spec and produce verification and the AC Verification Matrix. Do not create the PR or archive.

## Procedure

1. Confirm build eligibility with `mochiflow ready {slug}`: it runs `lint`, requires `status: approved`, and checks every surface's `default` verification command is runnable (not a `TODO:` placeholder). `default` is the canonical build-completion profile: it should be the reliable local command whose success is sufficient to say the surface is ready for PR / merge, except for checks explicitly documented as human-operated or CI-only. A non-zero exit is a stop condition — resolve it before implementing. Then read `spec.yaml` (risk / type / surfaces), `pitch.md`, `spec.md` (plus `design.md` / `tasks.md` if present), the constitution (`[constitution].project` / `[constitution].local`), the foundational context (`[context].product` / `[context].structure` / `[context].tech`) for orientation, and `[adr].pitfalls`. If `mochiflow ready` is unavailable, fall back to reading `spec.yaml` and confirming `status: approved` and runnable verification manually.
2. Define this build's **commit unit**: when `tasks.md` exists, the unit is one currently open task; when `tasks.md` is absent (taskless / micro specs), the unit is the whole logical change. Prepare the branch per `reference/git.md ## Branch`: verify the branch `{prefix}/{slug}` exists locally or on `origin`, switch to it, and error-stop if it does not exist. Verify the worktree has no changes other than this spec's own `{specs_dir}/{slug}/**` (else stop). Exception: when build resumes from `ship.md ## PR Feedback Loop`, the restore from `_done` is related, so `{specs_dir}/{slug}/**` and `{specs_dir}/_done/{slug}/**` are the only allowed dirty paths; any other dirt still stops.
3. **Task loop**: repeat 3a–3f for each open task.
   - 3a. Read surrounding source before editing; for logic changes use TDD (RED→GREEN→REFACTOR), match existing style, and keep changes minimal. Per `reference/engineering-standards.md`, for any dependency / tool / framework-idiom change or any deviation, confirm the upstream-recommended approach from primary sources before implementing and record its source.
   - 3b. Append seam decisions / ownership / dead-code handling to `design.md ## Integration Log` only when `design.md` exists and the integration-log column in `reference/risk.md` calls for it. For `standard`, do not create or require `design.md ## Integration Log`.
   - 3c. Run the canonical `default` command from `reference/workflow.md ## Verification profiles` for build-completion evidence. Optional profiles such as `quick` / `targeted` may be used for intermediate feedback, but they do not replace `default`. Fix any FAIL and re-run to PASS.
   - 3d. Treat approved `tasks.md` structure as a plan contract. During build, `tasks.md` may be changed only to mark completed task checkboxes (`- [ ]` → `- [x]`) and to record AC Matrix result / evidence fields when the matrix lives in `tasks.md`. If implementation needs task additions, deletions, splits, renumbering, AC / NFR / chore reference changes, dependency changes, `Files:` changes, or meaningful `Done:` / `Stop:` changes, stop and route back to `plan` for re-approval instead of editing the task structure in build.
   - 3e. When the task's implementation and verification PASS, first mark that task as checked in `tasks.md` (`- [ ]` → `- [x]`) when `tasks.md` exists. Do not stage or commit while the completed task remains unchecked. Then commit per `reference/git.md ## Auto-commit`, using one `Task:` trailer for that task. Normal build commits do not combine multiple task completions; taskless / micro specs create one logical-unit build commit with no `Task:` trailer. Stage files explicitly.
   - 3f. Follow the reviewer cadence in `reference/risk.md`; when required, run `agents/independent-reviewer.md` read-only via `reference/risk.md ## Review transport` (prefer delegated subagent when available; use inline reviewer role only when subagents are unavailable or dispatch fails for a runtime/tooling reason) and append the reviewer mode + verdict to `design.md ## Review Results`. For `critical`, this happens after each task.
4. After all tasks complete, run final verification once more. Fix any FAIL and re-run to PASS.
5. For `elevated`, run the required independent-reviewer once after all tasks using the same review transport. Record `Reviewer mode: delegated | inline` with the verdict in `design.md ## Review Results`.
6. Record the AC Verification Matrix (at the end of tasks.md if present, else end of spec.md). Settle automated AC as `PASS` / `FAIL` / `N/A: <reason>`, mark an automated AC row not yet verified as the provisional `UNVERIFIED`, and record AC needing human/visual checking as `PENDING_HUMAN` without requesting that QA here (the request is made once, in ship). Provisional tokens (`UNVERIFIED`, `PENDING_HUMAN`) are build-time placeholders only and are not done-eligible (`reference/workflow.md ## AC Matrix`).
7. Include the build-time AC Verification Matrix update in the final build commit for this phase, then stop. `ship` only commits Matrix rows or evidence changed by final verification / human QA, as part of the close-out commit.

## Presentation

- In user-facing summaries, call the AC Verification Matrix the acceptance
  checks or verification items in the artifact language. Keep the exact heading
  only when pointing to the document.
- Report reviewer output as the review result. Include `delegated` / `inline`
  only when it explains how the review ran or when the user asks.
- Summarize implementation as what changed, what was checked, and what remains
  for wrap-up; do not lead with `risk`, `status`, or reviewer mode.
- On build completion, always include: (1) the verification result (all items
  passed, or human confirmation items remain), and (2) explicit next-step
  guidance directing the user to `mochiflow-ship`.

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
4. If they disagree (a checked task lacks a matching `Task:` trailer in any
   commit, or a `Task:` trailer exists for an unchecked task), **stop and
   reconcile** before editing source files — read the relevant commits and
   `tasks.md` to determine the true state, fix `tasks.md` checkboxes to match
   reality, then resume.

## Stop conditions

- Do not implement when `status` is not `approved` or `spec.yaml` is missing (a non-zero `mochiflow ready {slug}` exit signals this).
- Stop when an out-of-scope change or a new design decision is needed.
- Do not finish build while verification or a required reviewer verdict is FAIL.
- `build` never sets `status: done`. Setting `done` is ship's responsibility, on the acceptance conditions in `reference/workflow.md ## AC Verification Matrix`. At build's end the status stays `approved`.
- Do not create the PR / move to `_done/` / request human checking (those are ship's responsibility).
