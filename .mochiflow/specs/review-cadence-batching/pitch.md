# Batch bounded in-scope fixes: hold locally, review once before push/accept

## Problem

`reference/risk.md`'s "verdict freshness" rule is correct in principle — for
`risk >= elevated`, any code change after the recorded reviewer verdict makes it
stale, and a fresh `change-reviewer` run is required before the next push
(`update.md` step 4) or `mochiflow accept` (`open.md` step 3e QA-FAIL rework).
But it is triggered at "every code change" granularity, not at the granularity
of the boundary it is meant to protect.

`commands/update.md` runs apply-fix → commit → review (if stale) → push → PR
metadata as one atomic procedure per invocation. Natural-language triggers
(`修正依頼`, `PRを直して`) re-run that whole procedure on every user
utterance, so N separate small PR-feedback items in one sitting become N full
reviewer runs and N pushes to the same open PR. Nothing in the current
procedures distinguishes "apply this and hold" from "this is the last item,
ship it."

The same gap exists, undocumented, in the window after `build` finishes its
task loop but before `open` runs: there is no defined lightweight path for "one
more small in-scope touch-up" there — an agent either silently reopens the task
loop (risking the `prevent-build-phase-spec-mutation` invariant) or forces a
full `plan` detour for something that never left the approved scope.

## Appetite

Small-to-medium. This is engine workflow-contract editing across
`commands/update.md`, `commands/open.md`, `commands/build.md`, and
`reference/risk.md`, plus the conformance tests that pin engine doc wording
(the existing `read_repo_file("engine/...")` assertion pattern in
`cli/crates/mochiflow-cli/tests/conformance.rs`). No Rust *behavior* code
changes to `accept.rs` / the reviewer transport are required. Bounded to those
files plus the freeze/upgrade/adapter-generate cycle the constitution requires
after any `engine/` edit.

## Solution

1. **Generalize the existing "bounded inline fix" judgment, don't invent one.**
   `build.md` already stops build for "an out-of-scope change or a new design
   decision" and routes to `plan`; `update.md` already routes unrelated feedback
   to a new spec. State this judgment once (in `reference/risk.md` or
   `reference/workflow.md`) and have `build.md` / `open.md` / `update.md`
   reference it: **in-scope** requests (no task-structure change, no new AC, no
   new design decision) get the new handling below, wherever they occur (before
   or after `open`). **Out-of-scope** requests keep today's routing exactly as
   is: before `open`, back to `plan` for re-approval; after `open` (the spec is
   `accepted` and never reverts), a new spec.
2. **New handling for in-scope requests: apply, commit, hold.** Apply the fix
   with build discipline (read, TDD where applicable, re-verify with the
   surface's default/appropriate profile), commit it locally, and stop there.
   Do not push, do not run `mochiflow accept`, do not re-run the mandatory
   reviewer yet. Report a short status line noting the change is applied and
   held (not yet pushed/accepted or reviewed).
3. **Record `Reviewed through: <sha>` next to `Verdict:`** in
   `design.md ## Review Results` every time the mandatory reviewer runs
   (including build's own first elevated/critical run), so it is always
   possible to tell how many held commits exist since the last recorded review.
4. **One explicit finalize signal, separate from the soft hold-only hints.**
   The soft natural-language triggers (`修正依頼`, `PRを直して`, "ここも直し
   て") mean hold-only under this design. A distinct, explicit signal —
   `mochiflow-update` as an explicit command, or an unambiguous "that's
   everything / push it / update the PR" statement — means finalize: re-run
   `change-reviewer` once on the full diff from git (fidelity unchanged — still
   never conversation history or compact reports), record the fresh verdict and
   new `Reviewed through: <sha>`, then push (`update.md`) or proceed to accept
   (the post-build/pre-`open` window) for however many held commits
   accumulated.
5. **Move `open.md`'s freshness trigger to the accept gate (step 6), not just
   3e.** For `risk >= elevated`, add an unconditional precondition immediately
   before `mochiflow accept` runs: when any commit that changes code exists
   beyond the recorded `Reviewed through` sha — whether from 3e's QA-`FAIL`
   rework or from a held post-build bounded fix — re-run `change-reviewer`
   once and record the fresh verdict before proceeding. This fires whether or
   not the QA round-trip (step 3) even ran, which matters because step 3 is
   skipped entirely for a spec with no human-operated/visual QA item. 3e keeps
   its existing round-based batching but no longer carries a second, separate
   re-review instruction — it just feeds the same step-6 gate.
6. **Document the post-build/pre-`open` window in `build.md`**: after all tasks
   (or the single logical-unit commit for taskless/micro specs) complete and
   before `open` runs, further in-scope requests use the same bounded-fix +
   hold handling instead of silently reopening the task loop or forcing a plan
   detour. Out-of-scope requests keep routing to `plan` exactly as today.

## Rabbit Holes

- Do not weaken "a fresh reviewer verdict is required before push/accept" —
  only the trigger frequency changes, never the invariant.
- Do not turn "in-scope vs out-of-scope" into a new formal field/token; it stays
  the same judgment call already in `build.md`/`update.md`, now stated once and
  shared.
- Do not attempt the general trigger/routing overhaul tracked separately in the
  `trigger-routing-redesign` backlog seed — this spec adds exactly one new
  explicit finalize signal to `update`'s trigger set, nothing more.
- Do not narrow the reviewer's diff scope (an incremental, since-last-`sha`
  review) in this spec — the mandatory reviewer keeps reading the full diff
  from git; only how often it runs changes.
- Do not add a persisted "held/pending" state to `spec.yaml` — it stays fully
  derivable from git (commits after the last recorded `Reviewed through:
  <sha>`), consistent with the existing "derive, don't persist" pattern for
  delivery state.

## No-gos

- No change to `build.md`'s task-structure mutation rule
  (`prevent-build-phase-spec-mutation` stays intact) — task
  additions/removals/reordering still require returning to `plan`.
- No change to the two delivery approval gates, and review stays a quality
  assist, never a third gate.
- No CLI behavior changes to `accept.rs` or the reviewer transport (a possible
  follow-up, not this spec).
- No changes to `agents/change-reviewer.md`'s review contract (S0-S4 / diff
  scope).
- No redesign of the general trigger/routing architecture (see Rabbit Holes).

## Alternatives Considered

- A choice card after every applied fix asking "続けますか？pushします
  か？" — rejected: reproduces the same "something happens after every fix"
  friction this spec exists to remove.
- Fully automatic time- or count-based batching (push every N minutes/commits)
  — rejected: decouples push timing from the user's actual intent.
- Weakening verdict freshness to "review once per PR regardless of later
  pushes" — rejected: reopens exactly the stale-verdict risk
  `2026-06-28-build-orchestrator-disposable-workers`'s verdict-freshness
  sub-decision closed.
- Also relaxing `build.md`'s out-of-scope→`plan` rule in this pass — rejected:
  keeps appetite small and avoids touching the `prevent-build-phase-spec-mutation`
  invariant.
- Narrowing the reviewer's diff scope now (incremental S3 review) — rejected
  for this pass: a separate, riskier change to the reviewer contract; revisit
  once the frequency fix proves out.

## Open Questions

None — ready for plan.
