# Risk Reference

`risk` is the single dimension that decides reviewer cadence, integration-log
requirement, and commit granularity. It is an **ordered enum**, not a boolean:

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

| risk | reviewer cadence | integration log | commit granularity |
| --- | --- | --- | --- |
| `standard` | none (AC Verification Matrix only) | not written | 1 commit |
| `elevated` | independent-reviewer once, after all tasks | optional | per logical step |
| `critical` | independent-reviewer after **each** task | required, appended per task | per task |

Reviewer = `agents/independent-reviewer.md`, read-only. A recorded reviewer
verdict (`pass` / `pass-with-comments`) is required when `risk ≥ elevated`; this
is one of the acceptance conditions ship checks before setting `done`
(`workflow.md ## AC Verification Matrix`). Record mandatory reviewer runs in
`design.md ## Review Results`, using `Reviewer mode: delegated | inline` and
`Verdict: pass | pass-with-comments | fail`. For `critical`, append one entry per
required review run; for `elevated`, append the single post-task review entry.
Branch / PR / archive mechanics live in `git.md`; the AC Verification Matrix
format and human gates live in `workflow.md`.

## Review transport

Reviewer cadence names the reviewer procedure, not a mandatory transport. Run
`agents/independent-reviewer.md` read-only using the first available mode:

1. `delegated`: dispatch a subagent when the adapter/runtime supports it.
2. `inline`: when subagents are unavailable, the main agent temporarily switches
   to the independent-reviewer role and executes the same reviewer procedure
   inline.

Inline review must read `agents/independent-reviewer.md`, use the same Stage 1 /
Stage 2 / verdict format, and record `Reviewer mode: inline`. While in reviewer
role, the agent is read-only: do not edit files, update status, stage, commit,
or create PR metadata. Review inputs are spec artifacts, full diff / changed
files, integration log, and verification results — never conversation history as
evidence. After the verdict is produced, return to builder role before fixing
findings or resuming the flow.

## design.md required condition

`design.md` MUST exist when **any** of:

- `risk ∈ {elevated, critical}`
- `integration ≠ none`
- `len(surfaces) > 1`

Otherwise `design.md` is optional and the spec may be `spec.md` only.

## Ad-hoc review

When the user explicitly requests review (`レビューして` / `mochiflow-review`),
run `agents/independent-reviewer.md` via `## Review transport` regardless of
risk level.

- Target: the active spec's latest artifacts (spec.md, design.md, tasks.md as applicable).
- On HIGH findings: fix inline and re-run lint before resuming.
- On PASS / pass-with-comments: report the result and resume the interrupted flow.
- Does not change `status`, create commits, or block approval by itself.

This is independent of the risk-cadence table above. Risk-cadence review is
automatic and mandatory; ad-hoc review is user-triggered and optional.

## Examples

`standard`: in-screen display tweak · small bugfix · test addition · reversible refactor.

`elevated`: SwiftData optional attribute addition · persisted state-transition change ·
cross-module responsibility move · device/BLE/camera manual acceptance · non-breaking
API/endpoint/field addition · background job addition.

`critical`: schema change breaking existing data · migration / primary key / delete logic ·
auth / permission / security · external contract breaking change · data deletion or archival logic.
