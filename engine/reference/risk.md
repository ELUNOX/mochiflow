# Risk Reference

`risk` is the single dimension that decides reviewer cadence and integration-log
requirement. It is an **ordered enum**, not a boolean:

```
standard < elevated < critical
```

`risk` is required on every spec. Discovering a higher risk mid-flow escalates
the spec in place (raise `risk`, re-evaluate this table). This file is the only
place these consequences are tabulated.

## Judgment axes

Reversibility · boundary contract · data integrity · state/lifecycle ·
responsibility relocation · failure blast radius · integration.

For a multi-surface spec, evaluate each surface and adopt the highest risk.

## Levels

| risk | meaning |
| --- | --- |
| `standard` | ordinary change; failure recovers by revert |
| `elevated` | multiple surfaces/modules, persisted lifecycle, contract addition, manual workflow, wide non-breaking API impact |
| `critical` | migration / schema breaking, external contract breaking, auth/security, user-data loss, failure mode cannot be kept small |

## Consequences (single source of truth)

| risk | reviewer cadence | integration log |
| --- | --- | --- |
| `standard` | none (AC Matrix only) | not written |
| `elevated` | change-reviewer once, after all tasks | optional |
| `critical` | change-reviewer after **each** task | required, appended per task |

Build commit cadence is task-based and owned by `commands/build.md` plus
`reference/git.md`, not by this risk table. When `tasks.md` exists, normal build
commits complete one task at a time regardless of risk; taskless / micro specs
produce one logical-unit build commit.

Mandatory implementation reviewer = `agents/change-reviewer.md`, read-only. A
recorded reviewer verdict (`pass` / `pass-with-comments`) is required when
`risk ≥ elevated`; this is a build-completion gate and one of the acceptance
conditions open's accept close-out checks before setting `accepted`
(`workflow.md ## AC Matrix`). Verified commit units may be committed before the
mandatory reviewer run; reviewer findings are fixed, verified, and committed as
follow-up work before build completes. Record mandatory reviewer runs in
`design.md ## Review Results`, using `Review profile: change-reviewer`,
`Reviewer mode: delegated | inline`, and `Verdict: pass | pass-with-comments |
fail`. For `critical`, append one entry per required review run; for
`elevated`, append the single post-task review entry.

**Verdict freshness.** A recorded reviewer verdict is valid only for the code
diff it actually reviewed. Any later code change at `risk ≥ elevated` — including
`open`'s QA-`FAIL` rework and `update`'s PR-feedback fix — makes the recorded
verdict **stale**: a fresh reviewer run (same transport, on the new full diff
from git) is required and its verdict recorded in `design.md ## Review Results`
before that change is accepted (`mochiflow accept`) or pushed. A stale pass
verdict must not be reused to clear the gate for an unreviewed diff. Branch / PR
/ archive mechanics live in `git.md`; the AC Matrix format and delivery approval
gates live in `workflow.md`.

## QA attack coverage

This section is the single owner of how much adversarial QA a spec must carry and
how strong the evidence must be. `plan.md` (authoring) and reviewer audit
contracts reference this mapping instead of restating thresholds, per
`reference/authoring.md` SSOT discipline.

Seven QA attack dimensions frame the "do not trust that it works" pass:

- `QA-FUNC`: functional correctness and requirements fit.
- `QA-UX`: interaction, usability, error handling, and accessibility.
- `QA-ABUSE`: abuse cases, invalid input, authorization, and security.
- `QA-DATA`: data integrity, state, persistence, and migration.
- `QA-COMPAT`: integration, compatibility, generated artifacts, and contracts.
- `QA-RESIL`: reliability, performance, capacity, and recovery.
- `QA-REG`: regression, maintainability, and testability.

Attack scenarios are recorded as `QA-XX` rows in `spec.md ## QA Scenarios` (with
a `Dimension` column). A dimension that does not apply is represented by a row
with a reasoned `N/A: <reason>`, never by silent omission. `## QA Scenarios` is
the "what to test" source and carries no result columns; an attack whose outcome
must be recorded is referenced from the relevant AC's AC Matrix
`Planned test/QA` / `Evidence` column (the results ledger). Attacks are never
promoted to formal ACs and never get a separate attack-id scheme.

Required coverage and evidence strength scale with `risk`:

| risk | required dimensions | evidence strength |
| --- | --- | --- |
| `standard` | at least `QA-FUNC`, `QA-ABUSE`, and `QA-REG` exercised; others reasoned `N/A: <reason>` | automated / AI-observed evidence or a reasoned `N/A` |
| `elevated` | all relevant dimensions exercised (especially `QA-ABUSE`, `QA-DATA`, `QA-COMPAT`, and `QA-REG` where applicable) | concrete evidence for each exercised dimension; `N/A` needs a specific reason |
| `critical` | all applicable dimensions exercised | strong evidence (test output, logs, human confirmation); casual `N/A` is not accepted |

Micro specs (no `## QA Scenarios` table) keep dimension coverage optional. Specs
authored before this convention are not retrofitted.

## Micro escalation

Micro is a depth, not a stored risk value. It is available only when the metadata
and file set stay standard-risk, single-surface, and `integration: none`, with no
condition that would require `design.md` below. If direct micro planning or later
work discovers durable rationale, an active pitfall, integration, elevated or
critical risk, public contract impact, human QA, or an ADR fold need, escalate
the same spec in place before approval or delivery.

## Review transport

This section defines reviewer transport — the selection discipline "prefer a
delegated subagent when the adapter/runtime exposes one, else run inline
reviewer role". It applies only to the read-only reviewer contracts:
`agents/plan-auditor.md` and `agents/change-reviewer.md`. Build implementation
itself is inline and does not use this transport. The legacy
`agents/independent-reviewer.md` file is only a compatibility wrapper that maps
old invocations to one of these canonical contracts.

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
For mandatory risk-cadence review during `build`, after the verdict is produced, return to builder role before fixing findings or resuming the flow.
For ad-hoc review, do not fix findings inline; report them and ask whether to enter the appropriate build/fix flow.

## design.md required condition

`design.md` MUST exist when **any** of:

- `risk ∈ {elevated, critical}`
- `integration ≠ none`
- `len(surfaces) > 1`
- migration or data loss risk exists
- external contract, API, or schema changes exist
- security, privacy, performance, or accessibility impact exists
- an independent reviewer is required

Otherwise `design.md` is optional and the spec may be `spec.md` only.

## Ad-hoc review

When the user explicitly requests review (`レビューして` / `mochiflow-review`),
run the appropriate canonical reviewer via `## Review transport` regardless of
risk level. Ad-hoc review is report-only and read-only.

- Target: the active spec's latest artifacts (spec.md, design.md, tasks.md as applicable).
- A code-less spec (no implementation yet) uses `plan-auditor` per
  `## Review transport`; once code exists, ad-hoc review uses
  `change-reviewer`.
- On High or Critical findings: report findings only, then ask whether to enter
  the appropriate build/fix flow. Do not edit files as part of ad-hoc review.
- On PASS / pass-with-comments: report the result and resume the interrupted flow.
- Does not edit files, change `status`, create commits, or block approval by itself.

This is independent of the risk-cadence table above. Risk-cadence review is
automatic and mandatory; ad-hoc review is user-triggered and optional.

## Examples

`standard`: in-screen display tweak · small bugfix · test addition · reversible refactor.

`elevated`: SwiftData optional attribute addition · persisted state-transition change ·
cross-module responsibility move · device/BLE/camera manual acceptance · non-breaking
API/endpoint/field addition · background job addition.

`critical`: schema change breaking existing data · migration / primary key / delete logic ·
auth / permission / security · external contract breaking change · data deletion or archival logic.
