# {title} — Design

<!--
This template is structural. When rendering a real artifact, translate
human-facing headings and prose to the configured project language.
Preserve machine-readable IDs and enum values such as AC-01, QA-01, T-001,
NFR-01, UNVERIFIED, PASS, PENDING_HUMAN, HUMAN_CONFIRMED, N/A: <reason>, FAIL.
Remove template-only Rules blocks unless the project intentionally keeps them.
-->

> Create `design.md` only when the workflow requires a technical contract.
> Write decisions, contracts, risks, and verification strategy. Do not turn this
> into an implementation diary or current-state encyclopedia.

## Decision Summary

- D-01: {decision}
  - Why: {reason}
  - Alternatives considered:
    - {alternative}
  - Consequences:
    - {tradeoff or implication}

## Current State Scan

- Relevant files:
  - `path/to/file`
- Existing patterns:
  - {pattern}
- Constraints:
  - {constraint}

## Proposed Design

### Components

| Component | Responsibility | Changes |
| --- | --- | --- |
| {component} | {responsibility} | {change} |

## Integration Contract

| Contract | Input / Request | Output / Response | Errors | Compatibility |
| --- | --- | --- | --- | --- |
| {contract} | {input} | {output} | {error behavior} | {compatibility notes} |

## Failure Modes

| Failure | Expected behavior | Verification |
| --- | --- | --- |
| {failure} | {expected behavior} | {AC or test} |

## Migration / Rollout / Rollback

- Migration:
  - {steps or "None"}
- Rollout:
  - {plan or "None"}
- Rollback:
  - {rollback plan or "None"}
- Backward compatibility:
  - {compatibility notes}

## Observability

- Logs:
  - {logs}
- Metrics:
  - {metrics}
- Alerts:
  - {alerts}
- Debuggability:
  - {debug notes}

## Test Strategy

| Layer | What | Related AC/NFR |
| --- | --- | --- |
| unit | {unit test scope} | AC-01 |
| integration | {integration test scope} | AC-02 |
| QA | {manual QA scope} | QA-01 |

## Review Results

- Required reviewer:
  - {yes/no and why}
- Findings:
  - {findings or "None yet"}

## Integration Log

- {integration notes}

<!-- Authoring rules:

- `design.md` is required when risk is `elevated` or `critical`.
- `design.md` is required when integration is not `none`.
- `design.md` is required for multi-surface work.
- `design.md` is required for migration or data loss risk.
- `design.md` is required for external contracts, API changes, schema changes, security, privacy, performance, or accessibility impact.
- `design.md` is required when independent reviewer is required.
- For trivial or micro specs, `design.md` may be omitted when workflow docs allow it.
- Do not turn `design.md` into a current-state encyclopedia.
- Current state should be concise and relevant only to this spec.
-->
