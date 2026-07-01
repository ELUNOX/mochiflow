# {title}

## Background and Design Rationale

- Problem to solve / why now / why this approach.
- Key design decisions and rationale (chosen option and rejected alternatives).
- Origin, if this came from a backlog seed (slug / source).

## User Story

As {user}, I want {capability}, so that {reason}.

## Scope

- In:
- Out:

## Edge Cases

- 

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL ...
- AC-02: WHEN ..., THE SYSTEM SHALL ...

## QA Scenarios

> Cover the QA attack dimensions from `reference/risk.md ## QA attack coverage`
> (`QA-FUNC`, `QA-UX`, `QA-ABUSE`, `QA-DATA`, `QA-COMPAT`, `QA-RESIL`,
> `QA-REG`). Required coverage and evidence strength per `risk` are owned by
> that mapping; a dimension that does not apply is a row with a reasoned
> `N/A: <reason>`. Reference an attack from the AC Matrix via its `QA-XX` id; do
> not promote attacks to ACs or mint a separate attack id.

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC, QA-REG | ios | Automated / Human-operated / Visual | ... | ... |
| QA-02 | QA-DATA | ios | Automated | N/A check: no persisted data/state is touched. | N/A: docs-only change, no data integrity surface. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

<!-- Create the ## Verification Plan / AC Matrix section during plan in this
spec.md (one row per AC) so it is present at approval; record verification
results during build.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | {surface} | automated | `command ...` | `path/File.ext` | UNVERIFIED | | |

-->
