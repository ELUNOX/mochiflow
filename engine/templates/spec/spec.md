# {title}

<!--
This template is structural. When rendering a real artifact, translate
human-facing headings and prose to the configured project language.
Preserve machine-readable IDs and enum values such as AC-01, QA-01, T-001,
NFR-01, UNVERIFIED, PASS, PENDING_HUMAN, HUMAN_CONFIRMED, N/A: <reason>, FAIL.
Remove template-only Rules blocks unless the project intentionally keeps them.
-->

## Problem / Goal

### Problem

- {Describe the user or system problem.}

### Goal

- {Describe the desired outcome.}

### Non-goals

- {Describe what this spec intentionally does not do.}

## Users / Actors

| Actor | Need | Notes |
| --- | --- | --- |
| {actor} | {need} | {constraints or assumptions} |

## User Stories

- US-01: As {actor}, I want {goal}, so that {reason}.
  - Priority: Must
  - Related AC: AC-01

## Scope

### In

- {in scope}

### Out

- {out of scope}

## Requirements / Acceptance Criteria

| AC | Type | Priority | Requirement | Verification |
| --- | --- | --- | --- | --- |
| AC-01 | functional | Must | THE SYSTEM SHALL ... | automated |
| AC-02 | edge | Must | WHEN ..., THE SYSTEM SHALL ... | QA-01 |

<!-- Authoring rules:

- Each AC must have a stable ID.
- Each AC must be independently checkable.
- Each AC must have a Verification value.
- Requirement text should be compatible with EARS-style wording when practical.
- Do not write vague AC such as "works correctly".
- Prefer observable behavior over implementation detail.
-->

Allowed Type examples:

- functional
- edge
- error
- security
- privacy
- performance
- accessibility
- compatibility
- human

Allowed Priority values:

- Must
- Should
- Could

## Business Rules

- BR-01: {rule}

## Examples / QA Scenarios

| QA | AC | Scope | Given | When | Then | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| QA-01 | AC-02 | {surface} | {precondition} | {action} | {expected result} | screenshot |

<!-- Authoring rules:

- QA scenarios must reference one or more AC IDs.
- Given / When / Then must describe concrete behavior.
- Evidence should describe what is needed, for example screenshot, screen recording, logs, test output, or user confirmation.
- Do not hard-code a platform unless the spec actually targets it.
- Use `{surface}` or `{surfaces}` placeholders where appropriate.
-->

## Non-functional Requirements

| NFR | Category | Requirement | Verification |
| --- | --- | --- | --- |
| NFR-01 | accessibility | THE SYSTEM SHALL ... | QA-01 |

<!-- Authoring rules:

- Include NFRs when risk, integration, security, privacy, performance, accessibility, compatibility, or migration concerns exist.
- For trivial specs, this section may contain "None identified" if that matches the project style.
-->

## Open Questions

Use this marker for unresolved questions:

- [NEEDS-CLARIFICATION: {question}]

<!-- Authoring rules:

- Approval to build must stop if unresolved `NEEDS-CLARIFICATION` items remain.
- Do not silently ignore unresolved questions.
-->

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | {surface} | automated | TBD | TBD | UNVERIFIED |  |  |
| AC-02 | {surface} | QA | QA-01 | TBD | UNVERIFIED |  |  |

<!-- Authoring rules:

- AC Matrix must exist at plan time.
- Each AC must have one or more rows in the matrix.
- Result must use the canonical enum:
  - UNVERIFIED
  - PASS
  - PENDING_HUMAN
  - HUMAN_CONFIRMED
  - N/A: <reason>
  - FAIL
-->
