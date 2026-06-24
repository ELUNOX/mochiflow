# Unify QA experience in ship: single-source scenarios, round-trip protocol, PR Testing section

## Background and Design Rationale

Ship's human QA flow lacks a structured round-trip protocol and splits QA
information across three locations (spec.md QA Scenarios, ephemeral
`qa-instructions.md`, AC Matrix). The root causes are:

1. No defined ask → respond → record → retry loop between the agent and the
   human during ship acceptance.
2. The QA "source of truth" is fragmented — `qa-instructions.md` holds how-to
   detail that never reaches PR reviewers (it is ephemeral/gitignored).

Design decisions:
- **Single QA source**: spec.md QA Scenarios becomes the sole authoritative
  definition of what to test and how. `qa-instructions.md` is removed entirely.
  Rationale: consolidating eliminates the split; the template's role (steps +
  expected result) is absorbed into spec.md, and the reviewer view is derived as
  PR `## Testing`.
- **Free-form human response**: human QA responses are interpreted by the agent
  (not pattern-matched against a fixed vocabulary). This aligns with
  language.md's existing two-layer model (free conversation input → canonical
  Matrix token).
- **PR Feedback Loop triggers added to router**: the loop procedure already
  exists in ship.md but is unreachable without reading the docs.

Origin: backlog seed `ship-qa-experience` (source phase: ship).

## User Story

As a developer using mochiflow, I want a clear QA round-trip during ship and
reviewer-visible testing instructions in the PR, so that QA failures have a
defined recovery path and PR reviewers know how to test the change.

## Scope

- In: ship.md QA acceptance rewrite (round-trip protocol + rework loop),
  router.md trigger additions, workflow.md acceptance-adapter and authoring
  updates, `pr-description.md` `## Testing` addition, `qa-instructions.md`
  template removal, spec.md template QA Scenarios column update,
  conformance test updates, MANIFEST regeneration.
- Out: CLI Rust library changes (beyond conformance tests), plan.md/build.md
  procedure changes, AC Matrix token changes, language.md changes.

## Edge Cases

- Human responds ambiguously (e.g. "hmm not sure") — agent re-asks for a clear
  pass/fail intent rather than guessing.
- All QA items are automated (no human-operated items) — ship skips the human
  round-trip entirely and relies on verification command results.
- Spec has no QA Scenarios section — lint should warn; ship falls back to
  verification-only acceptance.

## Acceptance Criteria (EARS)

- AC-01: WHEN ship enters acceptance, THE SYSTEM SHALL present human-operated
  and visual QA items as a numbered list derived from spec.md QA Scenarios
  (scenario name, steps, expected result).
- AC-02: WHEN a human responds with pass intent, THE SYSTEM SHALL record
  `人間確認済み` in the AC Matrix for that item.
- AC-03: WHEN a human responds with fail intent and a reason, THE SYSTEM SHALL
  record `FAIL` in the AC Matrix, pause ship (status stays `approved`), run a
  build-equivalent fix loop, then re-present only the failed items.
- AC-04: WHEN all QA items reach a done-eligible result, THE SYSTEM SHALL resume
  the ship step that sets `status: done`.
- AC-05: THE SYSTEM SHALL add router triggers `{slug} feedback` / 「修正依頼」 /
  「PR feedback」 that invoke the existing PR Feedback Loop in ship.md.
- AC-06: WHEN ship generates `pr-body.md`, THE SYSTEM SHALL include a
  `## Testing` section derived from spec.md QA Scenarios.
- AC-07: THE SYSTEM SHALL remove `engine/templates/delivery/qa-instructions.md`
  and update all references in ship.md, workflow.md, and authoring.md to point to
  spec.md QA Scenarios.
- AC-08: THE SYSTEM SHALL update `spec.standard.md` QA Scenarios table to
  include a `Type` column (Automated / Human-operated / Visual).
- AC-09: THE SYSTEM SHALL pass `cargo test`, `mochiflow lint`, `mochiflow
  freeze --check`, and `mochiflow doctor` after all changes.

## QA Scenarios

| QA | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | cli | Automated | Run `cargo test --manifest-path cli/Cargo.toml` after all engine file changes | All tests pass (conformance tests updated to no longer reference deleted template) |
| QA-02 | cli | Automated | Run `mochiflow lint --spec ship-qa-experience` | 0 fail, 0 warn |
| QA-03 | cli | Automated | Run `mochiflow freeze --check` after MANIFEST regeneration | Exit 0 |
| QA-04 | cli | Automated | Run `mochiflow doctor` | Exit 0 |
| QA-05 | human | Human-operated | Read ship.md acceptance section and confirm the round-trip protocol is clear and complete | Rework loop is documented with numbered steps |
| QA-06 | human | Human-operated | Read router.md and confirm `{slug} feedback` / 「修正依頼」 trigger is present and correctly routes | Triggers listed in ship trigger_patterns and router handles them |
| QA-07 | human | Human-operated | Read `pr-description.md` and confirm `## Testing` section exists with derivation instructions | Template contains the new section |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `人間確認済み`, or `対象外（<reason>）`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | human | QA-05 | `engine/commands/ship.md` | PENDING_HUMAN | | |
| AC-02 | cli | human | QA-05 | `engine/commands/ship.md` | PENDING_HUMAN | | |
| AC-03 | cli | human | QA-05 | `engine/commands/ship.md` | PENDING_HUMAN | | |
| AC-04 | cli | human | QA-05 | `engine/commands/ship.md` | PENDING_HUMAN | | |
| AC-05 | cli | human | QA-06 | `engine/commands/ship.md`, `engine/router.md` | PENDING_HUMAN | | |
| AC-06 | cli | human | QA-07 | `engine/templates/delivery/pr-description.md` | PENDING_HUMAN | | |
| AC-07 | cli | automated | QA-01, QA-03 | `engine/templates/delivery/`, `engine/commands/ship.md`, `engine/reference/workflow.md`, `engine/reference/authoring.md` | UNVERIFIED | | |
| AC-08 | cli | automated | QA-01 | `engine/templates/spec/spec.standard.md` | UNVERIFIED | | |
| AC-09 | cli | automated | QA-01, QA-02, QA-03, QA-04 | all | UNVERIFIED | | |
