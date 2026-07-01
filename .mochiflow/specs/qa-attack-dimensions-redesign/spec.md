# Redesign QA attack coverage and independent review contracts

## Background and Design Rationale

MochiFlow currently captures adversarial QA coverage through seven personas in
`spec.md ## QA Scenarios`: P1 new user, P2 power user, P3 malicious user,
P4 data integrity, P5 migration, P6 regression, and P7 spec skeptic. That
convention improved pre-implementation testing discipline, but the persona
language makes the responsibility boundary fuzzy: it can read as if plan or the
reviewer should role-play seven people rather than author and audit a
risk-based test attack model.

The current `independent-reviewer` prompt has a similar coupling problem. It
contains both code-less plan-quality review and post-implementation review under
one role, even though those activities inspect different inputs and should use
different professional standards. The follow-up design keeps the good parts of
the grounded reviewer decision while splitting the public contract into two
clear profiles: `plan-auditor` before code exists, and `change-reviewer` after
code exists.

The QA dimensions are grounded in external references and adapted to MochiFlow's
spec-lane workflow:

- ISO/IEC 25010 product quality model:
  https://iso25000.com/index.php/en/iso-25000-standards/iso-25010
- ISTQB risk-based testing:
  https://istqb-glossary.page/risk-based-testing/
- WCAG 2.2 accessibility guidance:
  https://www.w3.org/TR/WCAG22/
- Nielsen Norman Group usability heuristics:
  https://www.nngroup.com/articles/ten-usability-heuristics/
- OWASP ASVS:
  https://owasp.org/www-project-application-security-verification-standard/
- OWASP Web Security Testing Guide:
  https://owasp.org/www-project-web-security-testing-guide/
- OWASP Code Review Guide:
  https://owasp.org/www-project-code-review-guide/
- Google Engineering Practices code review guidance:
  https://google.github.io/eng-practices/review/reviewer/looking-for.html
- Google Engineering Practices small CL / refactoring guidance:
  https://google.github.io/eng-practices/review/developer/small-cls.html
- SEI CERT Coding Standards:
  https://cmu-sei.github.io/secure-coding-standards/
- NIST SP 800-53 Rev. 5:
  https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final

The chosen approach keeps `QA-XX` as the stable scenario identifier and replaces
the `Persona` column with a `Dimension` column. This preserves traceability into
the AC Matrix without adding a parallel attack-ID scheme. Risk-scaled coverage
continues to live in `reference/risk.md`; plan authors reference that policy,
and reviewers audit it without becoming authors or test executors.

Retire `independent-reviewer` as the public and canonical reviewer name. A
legacy file or adapter alias may remain only as a compatibility bridge when a
direct rename would break existing generated targets. The canonical review
contracts become:

- `plan-auditor`: read-only audit of spec metadata, requirements, design, tasks,
  QA dimensions, ADR decisions, and active pitfalls before implementation.
- `change-reviewer`: read-only code review of the full diff, changed files,
  tests, verification evidence, integration/contract drift, regression exposure,
  security/abuse concerns, code health, and refactor safety.

## User Story

As a developer using MochiFlow, I want QA coverage and review roles expressed as
standard engineering review concepts, so that plan authoring, code review, and
verification evidence have clear responsibilities and less workflow-specific
ambiguity.

## Scope

- In: update engine docs, templates, reviewer prompt contracts, Kiro reviewer
  template, and conformance guards for QA dimensions and split review profiles.
- In: keep review read-only and preserve the existing delivery approval model.
- In: retire `independent-reviewer` as the public/canonical name while providing
  an explicit compatibility path for existing adapters if needed.
- In: preserve compatibility for existing lint expectations around
  `design.md ## Review Results` (`Reviewer mode` and `Verdict`).
- Out: retroactively migrating archived specs.
- Out: adding CLI semantic lint for dimension coverage in this change.
- Out: adding write/shell capabilities to reviewer agents.
- Out: introducing a third delivery approval gate.

## Edge Cases

- A standard-risk internal refactor may exercise `QA-FUNC` and `QA-REG` while
  marking user-facing, data, and external-contract dimensions as reasoned
  `N/A`.
- A documentation-only change may still need `QA-FUNC` and `QA-REG`, but can mark
  `QA-ABUSE`, `QA-DATA`, `QA-COMPAT`, and `QA-RESIL` as `N/A` with concrete
  reasons.
- A generated adapter rename can break users even when the prompt content is
  correct; the implementation must either provide a compatibility alias for the
  existing Kiro generated reviewer target or fully test a deliberate migration to
  new generated names.
- A pure refactor can pass existing functional tests while still changing
  behavior through wiring or state ownership; `change-reviewer` must explicitly
  review behavior-preservation evidence.
- A risk-elevated plan can choose to skip pre-approval review, because review is
  a quality assist, not a third gate.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL replace the P1-P7 persona QA coverage model with
  industry-grounded QA attack dimensions in the engine authoring guidance,
  templates, and reviewer audit wording.
- AC-02: THE SYSTEM SHALL keep `QA-XX` as the scenario identifier and use a
  `Dimension` column for QA attack coverage, without introducing a parallel
  attack-ID scheme.
- AC-03: THE SYSTEM SHALL define risk-scaled dimension coverage and evidence
  strength in `reference/risk.md`, including the standard-risk default and
  reasoned `N/A` behavior.
- AC-04: THE SYSTEM SHALL define `plan-auditor` as the canonical
  pre-implementation review contract for spec/design/task/QA/ADR audit,
  preserving repository grounding and whole-tree impact/regression search.
- AC-05: THE SYSTEM SHALL define `change-reviewer` as the canonical
  post-implementation code-review contract, including tests, code health,
  security/abuse, integration compatibility, whole-tree regression search, and
  refactor safety.
- AC-06: THE SYSTEM SHALL preserve read-only reviewer transport and compatibility
  for existing reviewer verdict recording (`Reviewer mode`, `Verdict`) while
  allowing a review profile to distinguish `plan-auditor` from
  `change-reviewer`, and SHALL define how legacy `plan-quality mode` /
  `post-implementation mode` terminology maps to the new names.
- AC-07: THE SYSTEM SHALL retire `independent-reviewer` as the public/canonical
  reviewer name in favor of `plan-auditor` and `change-reviewer`, allowing the
  old name only as an explicitly documented legacy compatibility alias.
- AC-08: THE SYSTEM SHALL update Kiro adapter templates and conformance guards so
  generated reviewer resources use the new canonical review names, remain
  read-only and grounded, and either migrate or alias the legacy generated target
  deliberately.
- AC-09: THE SYSTEM SHALL update engine manifest, vendored engine, generated
  adapters, and verification evidence so `mochiflow freeze --check`,
  `mochiflow adapter generate --check`, `mochiflow lint --spec
  qa-attack-dimensions-redesign`, and the `cli` surface verification pass.
- AC-10: THE SYSTEM SHALL require both canonical reviewer contracts to include
  actionable remediation guidance for every finding severity, while preserving
  the existing gate semantics where only Critical / High confirmed findings
  cause `fail` and Medium / Low findings remain non-blocking
  `pass-with-comments`.

## QA Scenarios

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-UX, QA-REG | cli | Human-operated | Read the updated templates and guidance as a first-time author and a spec skeptic; check that dimensions are understandable, not role-play personas, and still trace through `QA-XX` to the AC Matrix. | The authoring path is understandable and internally consistent. |
| QA-02 | QA-ABUSE, QA-COMPAT | cli | Human-operated | Review the updated reviewer contracts for privilege creep, conversation-history dependence, and reviewer-as-author wording. | Reviewers remain read-only auditors and do not author QA, fix findings, stage, commit, or create PR metadata. |
| QA-03 | QA-DATA | cli | Automated | N/A check: confirm the change touches no persisted product data or runtime state. | N/A: engine docs/templates/tests only; no data persistence path exists. |
| QA-04 | QA-COMPAT, QA-REG | cli | Automated | N/A check: confirm archived specs are not migrated and existing legacy wording remains acceptable where already archived. | N/A: no archival migration is required; compatibility is preserved. |
| QA-05 | QA-COMPAT, QA-RESIL, QA-REG | cli | Automated | Run conformance and surface verification after engine/template/adapter updates. | Existing workflow contracts still pass, including reviewer transport and adapter generation checks. |
| QA-06 | QA-REG | cli | Human-operated | Check the post-implementation review contract against a large-refactor scenario and a mechanical-rename scenario. | The contract distinguishes pure/mechanical refactor from semantic refactor and requires behavior-preservation evidence where needed. |
| QA-07 | QA-FUNC | cli | Automated | Check the workflow-contract behavior through conformance and CLI tests: dimension coverage, review profile selection, Kiro target migration, and legacy wrapper behavior. | Functional workflow behavior matches the redesigned contracts without changing lint/accept semantics. |
| QA-08 | QA-UX, QA-REG | cli | Automated | Check reviewer completion output for a concrete remediation block on all findings, including Medium and Low findings. | Review output remains read-only and gate-compatible, but gives the main workflow agent enough fix guidance to avoid rediscovering context. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-01, QA-07 | T-001 replaced P1-P7 guidance with QA dimensions in risk, authoring, and templates; T-002 updated reviewer audit wording. | PENDING_HUMAN | Automated: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` PASS. Human QA-01 remains for open. |  |
| AC-02 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-01, QA-07 | T-001 documented the fixed QA dimension catalog and required coverage in `engine/reference/risk.md` and templates. | PENDING_HUMAN | Automated: surface default verification PASS. Human QA-01 remains for open. |  |
| AC-03 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-01, QA-07 | T-001 kept QA rows as `QA-XX` scenarios with AC Matrix evidence instead of formal AC promotion. | PENDING_HUMAN | Automated: surface default verification PASS. Human QA-01 remains for open. |  |
| AC-04 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-02 | T-002 added canonical `plan-auditor` and `change-reviewer` contracts with separate responsibilities. | PENDING_HUMAN | Automated: conformance `canonical_reviewers_grounded_adversary_contract_is_pinned` PASS inside surface default verification. Human QA-02 remains for open. |  |
| AC-05 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-02, QA-06 | T-002 made `change-reviewer` the post-implementation code review contract and added refactor safety / behavior-preservation duties. | PENDING_HUMAN | Automated: surface default verification PASS. Human QA-02 and QA-06 remain for open. |  |
| AC-06 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-02 | T-002 preserved `Reviewer mode` / `Verdict` and added optional `Review profile`; legacy mode terms map to new profiles only as aliases. | PENDING_HUMAN | Automated: lint and surface default verification PASS. Human QA-02 remains for open. |  |
| AC-07 | cli | automated + human | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-02 | T-002 retired `independent-reviewer` as public/canonical and left it as a legacy wrapper; T-004 pins that behavior. | PENDING_HUMAN | Automated: conformance PASS inside surface default verification. Human QA-02 remains for open. |  |
| AC-08 | cli | automated + human | `mochiflow adapter generate --check`; conformance adapter tests; QA-02 | T-003 generated `.kiro/agents/spec-plan-auditor.json` and `.kiro/agents/spec-change-reviewer.json`, removed old generated reviewer target, and updated adapter tests. | PENDING_HUMAN | Automated: `cargo run --manifest-path cli/Cargo.toml -- adapter generate --check` PASS; surface default verification PASS. Human QA-02 remains for open. |  |
| AC-09 | cli | automated | `mochiflow freeze --check`; `mochiflow adapter generate --check`; `mochiflow lint --spec qa-attack-dimensions-redesign`; surface `default` verification | T-005 regenerated adapter outputs, confirmed no `contracts/contracts.lock` diff, and ran the configured verification. | PASS | `mochiflow freeze` no-op; `mochiflow upgrade --source engine` PASS; `mochiflow adapter generate --check` PASS; `cargo run --manifest-path cli/Cargo.toml -- adapter generate --check` PASS; `mochiflow lint --spec qa-attack-dimensions-redesign` PASS after T-005 check; surface default verification PASS. |  |
| AC-10 | cli | automated | `cargo test --manifest-path cli/Cargo.toml --test conformance`; QA-08 | T-006 added remediation guidance requirements and output fields to `plan-auditor` and `change-reviewer` without changing verdict rules. | PASS | `cargo test --manifest-path cli/Cargo.toml --test conformance` PASS; surface default verification PASS. |  |
