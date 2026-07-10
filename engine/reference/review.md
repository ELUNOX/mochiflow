# Review Reference

Reviewer cadence, transport, bounded fixes, verdict freshness, and ad-hoc review
for the main workflow agent. Risk classification, the `design.md` required
condition, and QA attack coverage live in `reference/risk.md`; the AC Matrix
format and delivery approval gates live in `reference/verification.md` and
`reference/lifecycle.md`. The reviewer's own read-only audit method and output
contract live in `agents/reviewer-core.md` and the two profile files.

## Reviewer cadence (single source of truth)

`risk` (`standard < elevated < critical`, owned by `reference/risk.md`) decides
reviewer cadence and the integration-log requirement:

| risk | reviewer cadence | integration log |
| --- | --- | --- |
| `standard` | none (AC Matrix only) | not written |
| `elevated` | change-reviewer once, after all tasks | optional |
| `critical` | change-reviewer after **each** task | required, appended per task |

Build commit cadence is task-based and owned by `commands/build.md` plus
`reference/git.md`, not by this table. When `tasks.md` exists, normal build
commits complete one task at a time regardless of risk; taskless / micro specs
produce one logical-unit build commit.

Mandatory implementation reviewer = `agents/change-reviewer.md`, read-only. A
recorded reviewer verdict (`pass` / `pass-with-comments`) is required when
`risk ≥ elevated`; this is a build-completion gate and one of the acceptance
conditions open's accept close-out checks before setting `accepted`
(`reference/verification.md ## AC Matrix`). Verified commit units may be
committed before the mandatory reviewer run; reviewer findings are fixed,
verified, and committed as follow-up work before build completes. Record
mandatory reviewer runs in `design.md ## Review Results`, using
`Review profile: change-reviewer`, `Reviewer mode: delegated | inline`, and
`Verdict: pass | pass-with-comments | fail`, followed by `Reviewed through:
<sha>` on its own line directly below `Verdict:`. For `critical`, append one
entry per required review run; for `elevated`, append the single post-task
review entry.

## Shared bounded-fix judgment

An in-scope code change has no task-structure change, no new AC, and no new
design decision. An out-of-scope change routes to `plan` before `open`, or to a
new spec after `open`. `commands/build.md`, `commands/open.md`, and
`commands/update.md` reference this shared judgment rather than redefining it.

## Verdict freshness

A recorded reviewer verdict is valid only for the code diff it actually reviewed
through the recorded `Reviewed through: <sha>`. For `risk ≥ elevated`, an
in-scope code change after the recorded reviewer verdict is applied and
committed locally, then held until the next push/accept boundary: a `git push`
that updates an open PR, or `mochiflow accept`. At that boundary, when any
code-changing commit exists beyond the recorded `Reviewed through` sha, a fresh
reviewer run (same transport, on the new full diff from git) is required at most
once for the accumulated commits before the change is pushed or accepted, and
the fresh verdict plus updated `Reviewed through: <sha>` are recorded in
`design.md ## Review Results`. A non-code commit such as `docs(context)` or
PR-body-only metadata does not by itself make the verdict stale. A stale pass
verdict must not be reused to clear the gate for an unreviewed diff. Review
batching changes trigger frequency, not review scope: reviewer input remains the
full diff from git.

## Review transport

This section defines reviewer transport — the selection discipline "prefer a
delegated subagent when the adapter/runtime exposes one, else run inline
reviewer role". It applies only to the read-only reviewer contracts:
`agents/plan-auditor.md` and `agents/change-reviewer.md`. Build implementation
itself is inline and does not use this transport.

Both canonical reviewers preserve S0 repository grounding and S2 whole-tree
impact / regression search. The profile split changes the review target, not the
grounding standard.

Delegated reviewer transport is preferred whenever the adapter/runtime exposes a
subagent mechanism. A user request that triggers ad-hoc review, or a
user-approved build flow that reaches mandatory risk-cadence review, is also an
explicit request to use delegated reviewer transport when available. Do not fall
back to inline merely because the host runtime says subagents require an explicit
delegation request; this rule and the active trigger provide that request.
Select the first available mode:

1. `delegated`: dispatch a subagent when the adapter/runtime supports it.
2. `inline`: only when subagents are unavailable or dispatch fails for a
   runtime/tooling reason, the main agent temporarily switches to the read-only
   reviewer role and executes the same procedure inline.

For review, select the reviewer profile by target:

- `plan-auditor`: code-less spec review before implementation, including
  `plan.md`'s pre-approval review for `risk >= elevated` and ad-hoc review on a
  spec with no implementation. It runs S0 Grounding, S1 Internal Coherence, S2
  Impact & Regression, S4 Knowledge Confrontation, and Falsification with
  `S3 Code Quality` reported `N/A (no implementation yet)`; **no diff /
  changed-files / integration-log input** is required.
- `change-reviewer`: post-implementation review, including mandatory
  risk-cadence review during `build`, stale-verdict re-review during `open` /
  `update`, and ad-hoc review once code exists. It runs S0 Grounding, S1 Spec
  And Evidence Coherence, S2 Impact & Regression, S3 Code Quality, S4 Knowledge
  Confrontation, and Falsification.

Inline review must read the selected canonical agent file, use that file's
S0-S4 / Falsification / verdict format, and record `Reviewer mode: inline`.
While in reviewer role, the agent is read-only: do not edit files, update
status, stage, commit, or create PR metadata. Review inputs are spec artifacts,
full diff / changed files when code exists, integration log, and verification
results — **never conversation history**. The mandatory risk-cadence review
reconstructs the full diff from git (`git diff origin/{base}...HEAD` for the
completion-gate review, or a task's own commit for a per-task `critical` review)
and reads the changed code from scratch.
For mandatory risk-cadence review during `build`, after the verdict is
produced, return to builder role before fixing findings or resuming the flow.
For result-only ad-hoc review, do not fix findings inline; report them and ask
whether to enter the appropriate build/fix flow. For `review fix [1-3]`, the
reviewer remains read-only and the main agent applies only bounded fixes under
`## Review-fix loop`.

## Review-fix loop

`commands/review.md` owns the public grammar. This section owns the shared
boundaries for `review fix [1-3]`.

Reviewers remain read-only in every review-fix cycle. The main agent owns
fixes, verification, stop decisions, staging, commits, push boundaries, status
changes, and PR metadata. Do not create or invoke a write-capable reviewer or
worker role for review fixes.

Each fix round is one reviewer pass followed by at most one bounded main-agent
fix pass. The number after `fix` is the maximum fix-round budget. The loop stops
after the final requested fix round and does not require a clean post-fix review.

Automatic fixes are allowed only when they satisfy the shared bounded-fix
judgment above: no task-structure change, no new AC, no new design decision, no
spec split, and no unrelated work. A finding that requires human judgment,
planning, a new contract decision, or a repeated unresolved issue after a prior
fix stops the loop instead of spending more budget. Verification failure after a
fix also stops the loop until the failure is resolved.

Later review cycles must be fresh independent reviews. Reviewer input is the
current artifacts or current full diff, plus cycle-local changed files or diff
as focus input when useful. Do not pass previous findings, previous verdicts,
previous reviewer summaries, review-fix ledger contents, or conversation
history to the reviewer. The main agent may retain prior findings for applying
fixes and deciding whether a finding repeated after a prior fix, but that
memory is not reviewer input.

Review-fix recovery state is local and gitignored under
`{install_dir}/state/{slug}/`, for example
`{install_dir}/state/{slug}/review-fix.json`. The ledger is for main-agent
recovery only and is not durable spec evidence. Record at least:

- requested fix rounds;
- completed fix rounds;
- current phase and reviewer profile;
- touched files;
- verification evidence;
- stop reason;
- updated_at.

On resume, recover from repository files plus the local review-fix ledger, not
from hidden conversation memory. If the ledger is missing or unreadable, do not
invent prior loop state; restart review or ask for explicit direction.

## Ad-hoc review

When the user explicitly requests review (`レビューして` / `mochiflow-review`),
run the appropriate canonical reviewer via `## Review transport` regardless of
risk level. Plain `{slug} review` is result-only and read-only. `{slug} review
fix [1-3]` is still user-triggered ad-hoc review, but only the reviewer is
read-only; the main agent may apply bounded fixes under `## Review-fix loop`.

- Target: the active spec's latest artifacts (spec.md, design.md, tasks.md as applicable).
- A code-less spec (no implementation yet) uses `plan-auditor` per
  `## Review transport`; once code exists, ad-hoc review uses
  `change-reviewer`.
- On High or Critical findings in result-only mode: report findings only, then
  ask whether to enter the appropriate build/fix flow. Do not edit files as
  part of result-only review.
- In `review fix [1-3]` mode: keep the reviewer read-only, then let the main
  agent apply at most one bounded fix pass for that round under
  `## Review-fix loop`.
- On PASS / pass-with-comments: report the result and resume the interrupted flow.
- Review by itself does not change `status`, create PR metadata, or block
  approval. Result-only review also does not edit files or create commits; fix
  mode follows the active lifecycle context for edits, staging, and commits.

This is independent of the risk-cadence table above. Risk-cadence review is
automatic and mandatory; ad-hoc review is user-triggered and optional.
