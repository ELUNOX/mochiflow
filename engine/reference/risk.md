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
| `elevated` | independent-reviewer once, after all tasks | optional |
| `critical` | independent-reviewer after **each** task | required, appended per task |

Build commit cadence is task-based and owned by `commands/build.md` plus
`reference/git.md`, not by this risk table. When `tasks.md` exists, normal build
commits complete one task at a time regardless of risk; taskless / micro specs
produce one logical-unit build commit.

Reviewer = `agents/independent-reviewer.md`, read-only. A recorded reviewer
verdict (`pass` / `pass-with-comments`) is required when `risk ≥ elevated`; this
is a build-completion gate and one of the acceptance conditions open's accept
close-out checks before setting `accepted` (`workflow.md ## AC Matrix`). Verified commit
units may be committed before the mandatory reviewer run; reviewer findings are
fixed, verified, and committed as follow-up work before build completes. Record
mandatory reviewer runs in `design.md ## Review Results`, using `Reviewer mode:
delegated | inline` and `Verdict: pass | pass-with-comments | fail`. For
`critical`, append one entry per required review run; for `elevated`, append the
single post-task review entry.

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
how strong the evidence must be. `plan.md` (authoring) and
`agents/independent-reviewer.md` (Stage 1) reference this mapping instead of
restating thresholds, per `reference/authoring.md` SSOT discipline.

Seven adversarial personas frame the "do not trust that it works" pass:

- P1 new user: intuitive operation, misclicks, empty submit, repeated clicks.
- P2 power user: fast keyboard operation, large input, Tab order, IME Enter.
- P3 malicious user: boundaries, invalid values, unauthorized actions, duplicate submit.
- P4 data integrity: inspect backing tables / state, not only the screen.
- P5 migration: old data, missing fields, format / encoding differences, volume.
- P6 regression: nearby existing behavior still works.
- P7 spec skeptic: compare the primary specification to observed behavior.

Personas are recorded as `QA-XX` rows in `spec.md ## QA Scenarios` (with a
`Persona` column). A persona that does not apply is a row with a reasoned
`N/A: <reason>`, never an omission. `## QA Scenarios` is the "what to test"
source and carries no result columns; an attack whose outcome must be recorded is
referenced from the relevant AC's AC Matrix `Planned test/QA` / `Evidence` column
(the results ledger). Attacks are never promoted to formal ACs and never get a
separate attack-id scheme.

Required coverage and evidence strength scale with `risk`:

| risk | required personas | evidence strength |
| --- | --- | --- |
| `standard` | at least P1, P3, P6, P7 exercised; others reasoned `N/A: <reason>` | automated / AI-observed evidence or a reasoned `N/A` |
| `elevated` | all relevant personas exercised (especially P3, P4, P5 where applicable) | concrete evidence for each exercised persona; `N/A` needs a specific reason |
| `critical` | all applicable personas exercised | strong evidence (test output, logs, human confirmation); casual `N/A` is not accepted |

Micro specs (no `## QA Scenarios` table) keep persona coverage optional. Specs
authored before this convention are not retrofitted.

## Micro escalation

Micro is a depth, not a stored risk value. It is available only when the metadata
and file set stay standard-risk, single-surface, and `integration: none`, with no
condition that would require `design.md` below. If direct micro planning or later
work discovers durable rationale, an active pitfall, integration, elevated or
critical risk, public contract impact, human QA, or an ADR fold need, escalate
the same spec in place before approval or delivery.

## Review transport

This section defines the independent-reviewer transport — the selection
discipline "prefer a delegated subagent when the adapter/runtime exposes one,
else run inline reviewer role". It applies only to the read-only
`agents/independent-reviewer.md`. Build implementation itself is inline and does
not use this transport.

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

For review, run `agents/independent-reviewer.md` read-only. Inline review must
read `agents/independent-reviewer.md`, use the same Stage 1 / Stage 2 / verdict
format, and record `Reviewer mode: inline`. While in reviewer role, the agent is
read-only: do not edit files, update status, stage, commit, or create PR
metadata. Review inputs are spec artifacts, full diff / changed files,
integration log, and verification results — **never conversation history**. A
**code-less spec** (no implementation yet — `plan.md`'s pre-approval review for
`risk >= elevated`, or ad-hoc review on a spec with no code) uses the reviewer's
**plan-quality mode**: Stage 1 conformance + spec-artifact quality only, with
**no diff / changed-files / integration-log input** required (Stage 2 is `N/A`
until code exists). The mandatory risk-cadence review reconstructs the full diff
from git (`git diff origin/{base}...HEAD` for the completion-gate review, or a
task's own commit for a per-task `critical` review) and reads the changed code
from scratch.
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
run `agents/independent-reviewer.md` via `## Review transport` regardless of
risk level. Ad-hoc review is report-only and read-only.

- Target: the active spec's latest artifacts (spec.md, design.md, tasks.md as applicable).
- A code-less spec (no implementation yet) uses the reviewer's plan-quality mode
  (Stage 1 conformance + spec-artifact quality, no diff/changed-files input) per
  `## Review transport`; once code exists, ad-hoc review uses the
  post-implementation mode.
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
