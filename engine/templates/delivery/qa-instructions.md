# QA Guide: {spec-title}

**Spec**: `{slug}`
**Branch**: `{branch}`
**Module**: {module}
**Created**: {date}

---

## Legend

- **Automated** — unit, integration, snapshot, UI, or other command-repeatable checks
- **Human operation** — checks requiring a person, device, OS permission dialog, external service, or complex business state
- **Human visual** — visual checks that are not represented in accessible state, such as animation, chart shape, or layout polish

---

## Preparation

AI runs:

- Run the final verification command and record the result.

Human runs when needed:

- [ ] {test-data or environment prerequisites}
- [ ] After setup is ready, tell the AI to start QA.

---

## Scenarios

| QA | AC | Scenario | Scope | Kind | Priority | Result | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| QA-01 | AC-01 | {scenario-01-name} | {surface} / cross-surface / human | Automated | Required | UNVERIFIED |  |
| QA-02 | AC-02 | {scenario-02-name} | {surface} / cross-surface / human | Human operation | Required | PENDING_HUMAN |  |
| QA-03 | AC-03 | {scenario-03-name} | {surface} / cross-surface / human | Human visual | Recommended | PENDING_HUMAN |  |

---

## Steps

### QA-01: {scenario-01-name}

**Precondition**: {precondition}
**Related AC**: AC-01
**Scope**: {surface} / cross-surface / human
**Runner**: Automated / Human operation / Human visual
**Check method**: automated test / AI records result after human operation / human visual confirmation
**Result**: UNVERIFIED / PASS / FAIL / PENDING_HUMAN / HUMAN_CONFIRMED / N/A: <reason>
**Evidence**: {screenshot path, log path, or human confirmation comment}

Human operation:

1. {operation step}
2. Tell the AI that QA-01 is ready to record.

Automated check:

```text
Command: {verification-command}
Expected: {expected-result}
```

Human visual check when needed:

- {visual check}

**Failing example**: {ng-example}

---

### QA-02: {scenario-02-name}

**Related AC**: AC-02

...

---

## Report Format

When AI verification fails:

```text
Scenario: QA-XX (step X)
Expected: {expected}
Actual: {actual}
Evidence: {screenshot path or log path}
```

When a human visual check finds a problem:

```text
Scenario: QA-XX
Step: X
Actual behavior:
Expected behavior:
Repro rate: always / sometimes
Evidence: {screenshot path or human confirmation comment}
```

When human confirmation passes:

```text
Scenario: QA-XX
Reviewer: {name or "human"}
Result: HUMAN_CONFIRMED
Comment:
```
