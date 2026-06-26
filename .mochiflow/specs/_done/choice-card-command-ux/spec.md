# Clarify choice-card commands and numbered replies

## Background and Design Rationale

MochiFlow currently asks users to move between phases with internal terms such as
`approved`, `build`, `ship`, and `later`. That is precise for engine authors, but
it is a poor primary UX. A plan confirmation can read as "implementation starts
now" even when the real operation is only `draft -> approved`, a consistency
check, and a plan commit. `later` is also opaque because it produces a resume
prompt rather than merely stopping.

The chosen approach is to make the visible choices describe user intent in the
conversation language: create the plan, confirm the plan, review, start
implementation, start PR preparation, create the PR, and create a resume prompt.
Internal keywords remain accepted compatibility inputs, but they are not the
first thing users see.

Numbered replies are intentionally ephemeral. A reply such as `1` is safe only
when it maps to the most recent choice card in the current conversation. It must
not become a durable command because the same number can mean different actions
at different lifecycle points.

For the plan confirmation card, the displayed choice itself carries the approval
semantics. Selecting the localized "confirm the plan" action by label or
displayed number is the approve-to-build gate input.

The implementation should update the engine source, not the vendored engine copy
directly. Because engine files change, the build work must also refresh
`engine/MANIFEST.json`, sync `.mochiflow/engine/` from source, and verify adapter
drift per the project constitution.

## User Story

As a developer using MochiFlow in Japanese conversation, I want phase-completion
choices to use plain action labels and short numbered replies, so that I can
continue safely without memorizing internal workflow terms or wondering whether
implementation or PR creation will start immediately.

## Scope

- In:
  - Define user-facing choice labels and accepted trigger words for discuss
    completion, plan draft confirmation, plan-confirmed choices, ad-hoc review
    completion, build completion, and PR title/body approval.
  - Define numbered replies as ephemeral aliases for the most recent choice card.
  - Treat visible choice selection as the action dispatch rule: selecting the
    localized plan-confirmation action by label or number satisfies the
    approve-to-build gate.
  - Replace visible `later` language with a localized resume-prompt action at
    only the agreed high-value handoff points: discuss completion, plan
    confirmation, review completion, and build completion.
  - Clarify that confirming the plan commits the plan but does not start
    implementation.
  - Keep PR body edits as ordinary feedback before the PR creation gate.
  - Sync and verify generated engine artifacts after source-engine edits.
- Out:
  - CLI runtime state for choice-card numbering.
  - Persistent numbered-command state across sessions.
  - A dedicated `PR本文を修正する` command.
  - Changing PR creation to run without an explicit PR approval action.
  - Making ad-hoc review mandatory before implementation.

## Edge Cases

- A bare number arrives after no active choice card exists.
- A bare number arrives after a later message has made the previous card stale.
- A user replies with `OK`, `承認`, or `LGTM` at the plan gate after the new
  primary label is introduced.
- A user replies with `1` when the most recent card maps `1` to the localized
  plan-confirmation action.
- A user writes free-form PR text feedback instead of choosing the localized PR
  creation action.
- A review result contains findings and the user asks to continue or resume.
- An older English workflow user replies with `review`, `build`, `ship`, `later`,
  `approved`, or `create pr`.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL present phase-completion choices with plain
  user-facing action labels while keeping stable internal commands and keywords
  available as compatibility triggers.
- AC-02: WHEN a choice card is displayed, THE SYSTEM SHALL allow numbered
  replies as aliases for that card's choices and SHALL treat those numbers as
  valid only for the most recent unambiguous choice card in the current
  conversation.
- AC-03: WHEN discuss completes, THE SYSTEM SHALL present localized visible
  actions for creating the plan and creating a resume prompt, with `plan` /
  `mochiflow-plan` and `resume` / `later` accepted as compatibility triggers.
- AC-04: WHEN a draft plan is ready for confirmation, THE SYSTEM SHALL present
  a localized visible approval action for confirming the plan and SHALL explain
  that this updates `spec.yaml` to `status: approved`, re-runs consistency
  checks, and commits the plan artifacts without starting implementation.
- AC-05: WHEN the most recent unambiguous choice card maps an option to
  the plan-confirmation action, THE SYSTEM SHALL treat selecting that option by
  label or number as the approve-to-build gate input.
- AC-06: WHEN a plan has been confirmed and committed, THE SYSTEM SHALL present
  localized visible actions for review, starting implementation, and creating a
  resume prompt, ordered by risk as defined by the plan procedure.
- AC-07: WHEN ad-hoc review completes from a plan-confirmed flow or another
  `status: approved` context, THE SYSTEM SHALL present localized visible actions
  for starting implementation and creating a resume prompt, while preserving
  report-only behavior for review findings.
- AC-08: WHEN build completes, THE SYSTEM SHALL present localized visible actions
  for starting PR preparation and creating a resume prompt as the follow-up
  actions.
- AC-09: WHEN PR title/body content is presented before PR creation, THE SYSTEM
  SHALL present a localized visible action for creating the PR as the explicit PR
  approval action and SHALL handle PR text edits as ordinary feedback, not as a
  dedicated command.
- AC-10: THE SYSTEM SHALL show the resume-prompt action only at discuss
  completion, plan confirmation, review completion, and build completion unless
  the user explicitly asks for a resume prompt elsewhere.
- AC-11: WHEN source engine files are updated for this change, THE SYSTEM SHALL
  refresh frozen and vendored engine artifacts and verify that generated
  adapters remain in sync.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1 | cli | AI-observed | Read the updated phase-completion instructions as a first-time Japanese user. | The visible actions describe what will happen without requiring knowledge of `build`, `ship`, `approved`, or `later`. |
| QA-02 | P2 | cli | AI-observed | Check that each choice card supports a short numbered reply and a concise compatibility trigger. | A power user can answer with a number or a stable keyword instead of typing the full Japanese label. |
| QA-03 | P3 | cli | AI-observed | Inspect numbered-reply rules for stale, out-of-range, contextless numeric input, and the plan-confirmation card. | Numbers are limited to the most recent unambiguous choice card and do not become global commands; selecting the localized plan-confirmation action by number dispatches the same action as selecting it by label. |
| QA-04 | P4 | cli | AI-observed | Inspect whether the design introduces a stored choice-card state file or persistent numbered-command state. | No persistent state is introduced; lifecycle state remains in spec artifacts. |
| QA-05 | P5 | cli | AI-observed | Inspect whether the change requires migration, schema changes, or data conversion. | N/A: documentation-only engine behavior contract; no persisted data schema is changed. |
| QA-06 | P6 | cli | Automated / AI-observed | Run the configured verification and inspect existing compatibility triggers. | Existing command tokens such as `mochiflow-plan`, `review`, `build`, `ship`, `later`, `approved`, and `create pr` remain usable where specified. |
| QA-07 | P7 | cli | AI-observed | Compare the final engine text against this spec's action matrix and no-go decisions. | The implemented wording matches the approved choice labels, trigger sets, review-context condition, plan-confirmation numbering rule, resume-prompt placement, and PR-body feedback decision. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- The required reviewer result for elevated risk is recorded before completion.
