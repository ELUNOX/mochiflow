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

> Cover the adversarial personas P1-P7 (P1 new user, P2 power user, P3 malicious
> user, P4 data integrity, P5 migration, P6 regression, P7 spec skeptic). Required
> coverage and evidence strength per `risk` are owned by
> `reference/risk.md ## QA attack coverage`; a persona that does not apply is a
> row with a reasoned `N/A: <reason>`. Reference an attack from the AC Matrix via
> its `QA-XX` id; do not promote attacks to ACs or mint a separate attack id.

| QA | Persona | Scope | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | ios | ... | ... |

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
