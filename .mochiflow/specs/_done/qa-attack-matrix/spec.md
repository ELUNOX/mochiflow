# Add a QA attack matrix to plan and reviewer flows

## Background and Design Rationale

The independent reviewer already checks spec conformance and code quality, but
nothing in the workflow forces adversarial "do not trust that it works" thinking
*before* implementation. When break-attempts happen only at code review, defects
surface late and can force re-planning. The workflow needs both halves: plan
captures feature-specific attack attempts and required evidence, and the reviewer
verifies those attacks were exercised and that the evidence backs the claim.
Origin: backlog seed `qa-attack-matrix` (source: conversation).

Key decisions (agreed in discuss):

- Reuse the existing `## QA Scenarios` table in `spec.md` by adding a persona
  dimension (P1 new user, P2 power user, P3 malicious user, P4 data integrity,
  P5 migration, P6 regression, P7 spec skeptic), rather than a separate
  `## QA Attack Matrix` section or a metadata-generated checklist. `spec.md`
  exists for every spec (micro and up), so standard specs are not left without
  attack coverage. Rejected: generated checklist (paperwork risk, hard to make
  evidence-bearing) and a `design.md`-only section (absent for standard specs).
- Scale required persona coverage and attack-evidence strength by `risk`, with
  `reference/risk.md` as the single owner of the full risk->persona/evidence
  mapping (including the `standard` default set P1/P3/P6/P7). `plan.md` and the
  reviewer reference that mapping rather than restating thresholds, per
  `reference/authoring.md` SSOT discipline. Rejected: requiring all seven
  personas for every spec (formalizes trivial work) and a fully optional
  convention (no enforcement).
- Extend the reviewer's existing Stage 1 instead of adding a new stage or
  splitting Stage 1, keeping the reviewer output contract unchanged. Rejected: a
  separate "QA Attack Review" stage (large blast radius, overlapping
  responsibility).
- Keep attacks traceable through `QA-XX` IDs referenced from the AC Matrix
  instead of promoting every attack to a formal AC or minting a parallel
  attack-ID scheme. `## QA Scenarios` stays the "what to test" source and carries
  no result columns; an attack whose outcome must be recorded is referenced from
  the relevant AC's AC Matrix `Planned test/QA` / `Evidence` (the results
  ledger), and a purely exploratory attack that backs no AC stays scenario-only
  until it surfaces a defect worth its own AC. Rejected: AC promotion for every
  attack (AC bloat, poor EARS fit) and an `ATK-XX` scheme (duplicate trace path
  across lint/templates/reviewer).

This is a docs-and-templates-only change to the engine prose; CLI lint
enforcement is intentionally deferred to a possible follow-up.

## User Story

As a developer driving a spec through MochiFlow, I want plan to capture
risk-appropriate adversarial test coverage and the reviewer to verify it was
exercised, so that "it does not actually work" defects are caught before they
force re-planning.

## Scope

- In: persona dimension (P1-P7) in `engine/templates/spec/spec.standard.md` and
  `engine/templates/spec/spec.md`; persona-coverage authoring guidance in
  `engine/commands/plan.md`; risk-scaled attack-evidence strength in
  `engine/reference/risk.md`; Stage 1 attack-evidence verification in
  `engine/agents/independent-reviewer.md`; `QA-XX` traceability guidance into the
  AC Matrix; dogfood sync of the vendored engine copy and generated adapters.
- Out: Rust/CLI lint enforcement of persona coverage; a new reviewer stage or
  reviewer output-format change; `design.md`; mandatory persona coverage for
  micro specs; new AC Matrix columns, result tokens, or attack-ID schemes.

## Edge Cases

- A standard spec whose change is genuinely internal (e.g. a reversible refactor)
  where most personas do not apply: the guidance must accept reasoned
  `N/A: <reason>` without forcing fabricated attacks.
- Micro specs that have no `## QA Scenarios` table: persona coverage stays
  optional and must not become a lint/authoring blocker.
- Specs already archived under `_done/` before this convention: they must remain
  lint-valid; the convention is not applied retroactively.
- A persona row marked exercised but with evidence that does not actually back
  the attack: the reviewer must flag it rather than accept the row.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL define a persona dimension covering P1-P7 in the
  `## QA Scenarios` tables of `engine/templates/spec/spec.standard.md` and
  `engine/templates/spec/spec.md`, so authors record per-persona attack coverage.
- AC-02: WHEN authoring a spec, `engine/commands/plan.md` SHALL instruct the
  author to capture risk-appropriate persona attack coverage in `## QA Scenarios`,
  referencing the risk->persona/evidence mapping owned by
  `engine/reference/risk.md` rather than restating per-risk thresholds.
- AC-03: THE SYSTEM SHALL define in `engine/reference/risk.md` a single
  risk->persona/evidence mapping that owns both required persona coverage and
  evidence strength per risk level: `standard` exercises at least personas P1,
  P3, P6, and P7 and allows reasoned `N/A: <reason>`; `elevated` requires evidence
  for the relevant personas; `critical` requires strong evidence and rejects
  casual `N/A`.
- AC-04: THE SYSTEM SHALL extend Stage 1 in
  `engine/agents/independent-reviewer.md` to verify, against the
  `engine/reference/risk.md` mapping, risk-appropriate persona-row presence,
  concrete `N/A` reasons, and that exercised rows carry evidence backing the
  attack, WITHOUT adding a new reviewer stage or changing the Completion output
  format.
- AC-05: THE SYSTEM SHALL keep attacks traceable via `QA-XX` IDs referenced from
  the AC Matrix `Planned test/QA` or `Evidence` columns, WITHOUT introducing a new
  AC Matrix column, a new result token, or a parallel attack-ID scheme.
- AC-06: WHEN engine source files change, THE SYSTEM SHALL keep the vendored
  engine copy and generated adapters in sync such that `mochiflow freeze`,
  `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, and
  the `cli` surface `default` verification all pass.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | cli | Human-operated | Read the updated `plan.md`, `risk.md`, reviewer, and templates as a first-time author and a spec skeptic; check the persona convention is understandable and matches the agreed design. | Persona dimension and risk-scaled evidence rules are present, internally consistent, and free of template residue. |
| QA-02 | P3 | cli | Human-operated | Draft a sample standard spec that omits required persona rows and uses an unreasoned `N/A`; apply the reviewer Stage 1 guidance to it. | Reviewer guidance flags the missing risk-appropriate rows and the unreasoned `N/A`. |
| QA-03 | P6 | cli | Automated | Run the `cli` `default` verification, `mochiflow lint`, and the engine dogfood sync checks (`mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`) after the template/prose edits. | All pass; no existing or archived spec breaks and no conformance/manifest/adapter drift. |
| QA-04 | P4 | cli | Automated | N/A check: confirm the change touches no persisted data or runtime state. | N/A: docs/templates only, no persisted data or state is involved. |
| QA-05 | P5 | cli | Automated | N/A check: confirm no data/format migration is introduced and existing specs need no conversion. | N/A: no migration; existing-spec validity is covered by QA-03. |
| QA-06 | P2 | cli | Human-operated | N/A check: consider fast/bulk authoring stress on a prose convention. | N/A: no input-rate or volume surface exists for a documentation convention. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix (in `tasks.md`) with a
  done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded, including the dogfood sync
  checks for the engine change.
