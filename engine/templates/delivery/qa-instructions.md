# QA Instructions: {spec-title}

**spec**: `{slug}`
**branch**: `{type}/{slug}`
**module**: {module}
**created**: {date}

---

## Legend

- Automated — command-reproducible checks such as unit, integration, snapshot, or UI tests.
- Human-operated — checks requiring a physical device, OS permission dialog, external integration, or complex business-data state.
- Visual confirmation — visual checks that are not represented in an accessibility tree, such as animation behavior or chart shape.

---

## Preparation

AI runs:

- Run the final verification command (build / test) and record the result.

Human completes when needed:

- [ ] {Test-data prerequisites, if any}
- [ ] Tell the AI that QA is ready to start.

---

## Scenario List

| #     | Scenario           | Scope | Type | Priority | Result |
| ----- | ------------------ | ----- | ---- | ------ | ---- |
| QA-01 | {scenario-01-name} | ios / api / web / cross-surface / human | Automated | Required | Not run |
| QA-02 | {scenario-02-name} | ios / api / web / cross-surface / human | Human-operated | Required | Not run |
| QA-03 | {scenario-03-name} | ios / api / web / cross-surface / human | Visual confirmation | Recommended | Not run |

---

## Detailed Steps

### QA-01: {scenario-01-name}

**Precondition**: {precondition}
**Scope**: ios / api / web / cross-surface / human
**Runner**: Automated / Human-operated / Visual confirmation
**Verification method**: automated test / AI records the result after human operation / human visual confirmation
**Result**: Not run / PASS / FAIL / Human confirmed / Not applicable (reason)
**Evidence**: {screenshot path, log path, or human confirmation comment}

Human steps:

1. {operation step}
2. Tell the AI to record the QA-01 result.

Automated check:

```
Command: {verification command}
Expected: {expected result}
```

Visual confirmation when needed:

- {visual check}

**Failure example**: {failure-example}

---

### QA-02: {scenario-02-name}

...

---

## Report Formats

When the AI observes a failure:

```
Scenario: QA-XX (check X)
Expected: {expected}
Actual: {actual}
Evidence: {screenshot path or log path}
```

When a human finds an issue during visual confirmation:

```
Scenario: QA-XX
Step: X
Actual behavior:
Expected behavior:
Reproducibility: always / intermittent
Evidence: {screenshot path or human confirmation comment}
```

When human confirmation finds no issue:

```
Scenario: QA-XX
Confirmed by: {name or "human"}
Result: Human confirmed
Comment:
```
