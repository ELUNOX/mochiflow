# Risk Reference

`risk` is the single dimension that scales reviewer cadence (owned by
`reference/review.md`), the integration-log requirement, QA attack coverage, and
whether `design.md` is required. It is an **ordered enum**, not a boolean:

```
standard < elevated < critical
```

`risk` is required on every spec. Discovering a higher risk mid-flow escalates
the spec in place (raise `risk`, re-evaluate). Reviewer cadence and the
integration-log requirement are tabulated in
`reference/review.md ## Reviewer cadence`.

## Judgment axes

Reversibility · boundary contract · data integrity · state/lifecycle ·
responsibility relocation · failure blast radius · integration.

For a multi-surface spec, evaluate each surface and adopt the highest risk.

## Levels

| risk | meaning |
| --- | --- |
| `standard` | ordinary change; failure recovers by revert |
| `elevated` | multiple surfaces/modules, persisted lifecycle, contract addition, manual workflow, wide non-breaking API impact |
| `critical` | migration / schema breaking, external contract breaking, auth/security, user-data loss, failure mode cannot be kept small |

## QA attack coverage

This section is the single owner of how much adversarial QA a spec must carry and
how strong the evidence must be. `plan.md` (authoring) and reviewer audit
contracts reference this mapping instead of restating thresholds, per
`reference/specs.md` SSOT discipline.

Seven QA attack dimensions frame the "do not trust that it works" pass:

- `QA-FUNC`: functional correctness and requirements fit.
- `QA-UX`: interaction, usability, error handling, and accessibility.
- `QA-ABUSE`: abuse cases, invalid input, authorization, and security.
- `QA-DATA`: data integrity, state, persistence, and migration.
- `QA-COMPAT`: integration, compatibility, generated artifacts, and contracts.
- `QA-RESIL`: reliability, performance, capacity, and recovery.
- `QA-REG`: regression, maintainability, and testability.

Attack scenarios are recorded as `QA-XX` rows in `spec.md ## QA Scenarios` (with
a `Dimension` column). A dimension that does not apply is represented by a row
with a reasoned `N/A: <reason>`, never by silent omission. `## QA Scenarios` is
the "what to test" source and carries no result columns; an attack whose outcome
must be recorded is referenced from the relevant AC's AC Matrix
`Planned test/QA` / `Evidence` column (the results ledger). Attacks are never
promoted to formal ACs and never get a separate attack-id scheme.

Required coverage and evidence strength scale with `risk`:

| risk | required dimensions | evidence strength |
| --- | --- | --- |
| `standard` | at least `QA-FUNC`, `QA-ABUSE`, and `QA-REG` exercised; others reasoned `N/A: <reason>` | automated / AI-observed evidence or a reasoned `N/A` |
| `elevated` | all relevant dimensions exercised (especially `QA-ABUSE`, `QA-DATA`, `QA-COMPAT`, and `QA-REG` where applicable) | concrete evidence for each exercised dimension; `N/A` needs a specific reason |
| `critical` | all applicable dimensions exercised | strong evidence (test output, logs, human confirmation); casual `N/A` is not accepted |

Micro specs (no `## QA Scenarios` table) keep dimension coverage optional. Specs
authored before this convention are not retrofitted.

## Micro escalation

Micro is a depth, not a stored risk value. It is available only when the metadata
and file set stay standard-risk, single-surface, and `integration: none`, with no
condition that would require `design.md` below. If direct micro planning or later
work discovers durable rationale, an active pitfall, integration, elevated or
critical risk, public contract impact, human QA, or an ADR fold need, escalate
the same spec in place before approval or delivery.

## design.md required condition

`design.md` MUST exist when **any** of:

- `risk ∈ {elevated, critical}`
- `integration ≠ none`
- `len(surfaces) > 1`
- migration or data loss risk exists
- external contract, API, or schema changes exist
- security, privacy, performance, or accessibility impact exists
- an independent review is required

Otherwise `design.md` is optional and the spec may be `spec.md` only.

## Examples

`standard`: in-screen display tweak · small bugfix · test addition · reversible refactor.

`elevated`: SwiftData optional attribute addition · persisted state-transition change ·
cross-module responsibility move · device/BLE/camera manual acceptance · non-breaking
API/endpoint/field addition · background job addition.

`critical`: schema change breaking existing data · migration / primary key / delete logic ·
auth / permission / security · external contract breaking change · data deletion or archival logic.
