# Redesign QA attack coverage and independent review contracts

## Problem

MochiFlow's current QA attack coverage uses seven personas (P1 new user, P2
power user, P3 malicious user, P4 data integrity, P5 migration, P6 regression,
P7 spec skeptic). The coverage intent is useful, but the persona framing makes
the responsibility boundary fuzzy: it can sound like plan or reviewer should
role-play seven people instead of authoring and auditing a risk-based test attack
model.

The current `independent-reviewer` contract has a related problem. It combines
plan-quality review and post-implementation review inside one role, even though
those jobs inspect different evidence and should use different standards. Before
code exists, the right activity is a spec/plan audit. After code exists, the
right activity is a real change/code review, including refactor safety.

## Appetite

Medium-large. This is an engine contract redesign across review prompts,
risk/authoring guidance, spec templates, adapter generation, and conformance
guards. It should remain documentation/template/prompt level unless plan proves
that CLI lint support is needed. No schema, `contracts.lock`, or
`engine/VERSION` change is expected unless plan discovers a hard contract
boundary.

## Solution

Replace the persona-framed QA coverage model with industry-grounded QA attack
dimensions. The working dimension set is:

- `QA-FUNC`: functional correctness and requirements fit.
- `QA-UX`: interaction, usability, error handling, and accessibility.
- `QA-ABUSE`: abuse cases, invalid input, authorization, and security.
- `QA-DATA`: data integrity, state, persistence, and migration.
- `QA-COMPAT`: integration, compatibility, generated artifacts, and contracts.
- `QA-RESIL`: reliability, performance, capacity, and recovery.
- `QA-REG`: regression, maintainability, and testability.

Keep the responsibility split explicit:

- plan defines the relevant QA attack dimensions and any reasoned `N/A`;
- the pre-implementation reviewer audits dimension coverage against risk and
  planned evidence strength;
- build/open executes verification and records evidence in the AC Matrix.

Split the current `independent-reviewer` concept into two named review contracts:

- `plan-auditor`: a read-only plan-quality audit of `spec.yaml`, `spec.md`,
  `design.md`, `tasks.md`, risk, QA attack dimensions, and ADR alignment before
  implementation.
- `change-reviewer`: a read-only post-implementation code review of the full
  diff, changed files, tests, verification evidence, integration/contract drift,
  regression risk, security/abuse exposure, and refactor safety.

`independent-reviewer` can remain as an internal transport umbrella if that
preserves adapter compatibility, but the user-facing and contract-level names
should move toward the industry terms above: plan/spec audit before code exists,
and change/code review after code exists.

Post-implementation review should be grounded in standard code-review practice:
design fit, behavioral correctness, test quality, code health, naming,
complexity, maintainability, security, documentation/context consistency, and
every changed line. Refactors should be first-class review subjects. Pure
refactors must preserve behavior and rely on existing or added characterization
tests; semantic refactors are ordinary implementation changes; large refactors
should normally be separated from feature or bug-fix work.

Use primary-source references during plan, including ISO/IEC 25010, ISTQB
risk-based testing, OWASP ASVS/WSTG and Code Review Guide, WCAG 2.2, Nielsen's
usability heuristics, Google Engineering Practices for code review and
refactoring CL shape, SEI CERT Coding Standards for security-sensitive code, and
NIST SP 800-53 where critical security/privacy controls are relevant.

## Rabbit Holes

- Do not make reviewers responsible for authoring QA scenarios, executing tests,
  fixing findings, updating status, staging, committing, or creating PR metadata.
- Do not force all dimensions as heavy evidence for every standard-risk spec;
  keep risk-scaled coverage.
- Do not turn post-implementation review into another spec audit with a diff
  attached; it should be a real change/code review.
- Do not make pure refactor claims without behavior-preservation evidence.
- Do not add semantic lint enforcement just because the prompt vocabulary
  changes; structural lint can be considered, but judgment-heavy checks likely
  belong to the reviewer.

## No-gos

- No write/shell capability for any reviewer contract.
- No conversation history as reviewer input.
- No third delivery approval gate; review remains a quality assist and
  acceptance condition where risk requires it.
- No retroactive migration requirement for archived specs.
- No new parallel attack-ID scheme if `QA-XX` scenario IDs can remain the stable
  trace path with a `Dimension` column.
- No adapter-breaking rename unless a compatibility path is planned.

## Alternatives Considered

- Keep P1-P7 personas and clarify wording. Rejected because it keeps the
  role-play ambiguity and does not align well with standard quality/security
  vocabulary.
- Put QA attack coverage entirely in the reviewer. Rejected because that makes
  the reviewer a planner and surfaces gaps too late.
- Keep one `independent-reviewer` file with two modes only. Rejected as the
  ideal end state because plan audit and code review have different inputs and
  evaluation standards, though a compatibility wrapper may still be useful.
- Require all dimensions for every spec. Rejected because it over-formalizes
  small reversible work.
- Treat refactors as ordinary low-risk cleanup. Rejected because refactors can
  silently change behavior and should carry explicit preservation evidence.

## Open Questions

- None -- ready for plan.
