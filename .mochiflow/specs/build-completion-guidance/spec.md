# Build phase should announce next step on completion

## Background and Design Rationale

- After build completes (AC Matrix verified, final commit made), the agent stops silently. Users must already know the verb sequence or read engine docs to discover that `mochiflow-ship` is the next action. This breaks the lifecycle guidance chain that is mochiflow's core value.
- The fix is a documentation-only addition to `engine/commands/build.md` Presentation section. No CLI code change is needed because the build-completion report is agent conversational output.
- Origin: backlog seed `build-completion-guidance` (source: conversation, ship phase observation).

## Problem / Goal

- Problem: build completes without telling the user what to do next, creating a dead end for first-time users.
- Goal: the agent always reports verification results and guides the user to `mochiflow-ship` upon build completion.
- Non-goal: explaining ship's full procedure at build end (that's ship's job).

## Scope

- In: add a presentation rule to `engine/commands/build.md` requiring next-step guidance on build completion.
- Out: CLI binary changes, new commands, changes to other engine files.

## Acceptance Criteria (EARS)

- AC-01: WHEN build completes (step 7), THE SYSTEM SHALL include in its completion report: (1) implementation/verification summary, (2) verification result status (all PASS or human confirmation pending), and (3) next-step guidance directing the user to `mochiflow-ship`.

## QA Scenarios

| QA | Scope | Steps | Expected result |
| --- | --- | --- | --- |
| QA-01 | cli | Read `engine/commands/build.md` Presentation section after the change | The section contains a rule requiring: summary, verification result, and `mochiflow-ship` guidance |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `人間確認済み`, or `対象外（<reason>）`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `cargo test --manifest-path cli/Cargo.toml` + manual doc review | `engine/commands/build.md` | PASS | 72 passed, 0 failed; rule added to Presentation section | |
