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

| # | Scenario | Scope | Kind | Priority | Result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | {scenario-01-name} | {surface} / cross-surface / human | Automated | Required | Not run |
| QA-02 | {scenario-02-name} | {surface} / cross-surface / human | Human operation | Required | Not run |
| QA-03 | {scenario-03-name} | {surface} / cross-surface / human | Human visual | Recommended | Not run |

---

## Steps

### QA-01: {scenario-01-name}

**Precondition**: {precondition}
**Scope**: {surface} / cross-surface / human
**Runner**: Automated / Human operation / Human visual
**Check method**: automated test / AI records result after human operation / human visual confirmation
**Result**: Not run / PASS / FAIL / 人間確認待ち / 人間確認済み / 対象外（理由）
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
Result: 人間確認済み
Comment:
```
