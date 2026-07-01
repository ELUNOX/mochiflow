# Redesign QA attack coverage and independent review contracts - Design

## Design Decisions

- Replace personas with dimensions, not with a new checklist artifact. `QA-XX`
  remains the scenario ID and the table gains a `Dimension` column. This keeps
  the existing AC Matrix trace path and avoids a second attack-ID scheme.
- Define seven dimensions as the canonical vocabulary: `QA-FUNC`, `QA-UX`,
  `QA-ABUSE`, `QA-DATA`, `QA-COMPAT`, `QA-RESIL`, and `QA-REG`.
- Keep `reference/risk.md` as the single owner of risk-to-coverage and evidence
  strength. `plan.md`, templates, and reviewer prompts reference it rather than
  restating the thresholds.
- Split review contracts by evidence type. `plan-auditor` handles code-less
  spec/design/task/QA/ADR audit. `change-reviewer` handles post-implementation
  code review from full diff, changed files, verification results, and ADR
  context.
- Preserve the grounded-reviewer core in both new contracts. `plan-auditor` and
  `change-reviewer` must both verify material claims against repository
  evidence and perform a whole-tree impact/regression search for renamed,
  retired, relocated, or newly introduced responsibilities. Splitting the
  reviewer must not regress the S0 grounding or S2 impact coverage established by
  `2026-07-01-grounded-independent-reviewer`.
- Retire `independent-reviewer` as the public and canonical name. During
  migration, the old name may exist only as a legacy compatibility alias or
  wrapper. Existing `Reviewer mode: delegated | inline` and `Verdict:
  pass | pass-with-comments | fail` recording stays valid so lint and accept do
  not need a schema-level change. A `Review profile:
  plan-auditor | change-reviewer` line can be documented as optional metadata.
- Retire `plan-quality mode` and `post-implementation mode` as public mode names.
  During migration, treat them as legacy aliases only: `plan-quality mode` maps
  to `plan-auditor`, and `post-implementation mode` maps to `change-reviewer`.
  Update `risk.md`, `review.md`, `plan.md`, reviewer prompts, and conformance
  pins consistently so both old terms do not remain canonical.
- Keep all reviewer contracts read-only. Delegation remains allowed only for
  review; implementation, judgment, gates, integration, and fold stay on the
  main agent.
- Do not add semantic CLI lint for dimension adequacy in this change.
  Conformance pins the prompt/template contract; the reviewer performs the
  judgment-heavy audit.

## Architecture

The implementation should update the engine source, then regenerate derived
engine artifacts through the dogfood sync path:

- `engine/reference/risk.md` owns dimension definitions, standard/elevated/
  critical coverage, evidence strength, and the review transport vocabulary.
- `engine/reference/authoring.md` explains the `Dimension` column and the
  authoring responsibility split.
- `engine/commands/plan.md` instructs authors to populate dimension coverage.
- `engine/commands/review.md`, `engine/commands/build.md`, `engine/commands/open.md`,
  `engine/commands/update.md`, `engine/reference/workflow.md`, and
  `engine/router.md` use `plan-auditor` / `change-reviewer` where the phase
  selects a review profile. `engine/commands/plan.md` must also update the
  pre-approval review wording so elevated-risk plan review no longer points at
  the old public mode name.
- `engine/templates/spec/spec.md` and `engine/templates/spec/spec.standard.md`
  replace `Persona` examples with `Dimension` examples. `spec.micro.md` remains
  optional for QA Scenarios.
- `engine/templates/spec/design.md` documents the optional `Review profile:
  plan-auditor | change-reviewer` line alongside the existing `Reviewer mode`
  and `Verdict` fields so new elevated/critical specs know how to distinguish
  review profiles.
- `engine/agents/plan-auditor.md` and `engine/agents/change-reviewer.md` define
  the two canonical contracts. `engine/agents/independent-reviewer.md` is
  deleted or reduced to a legacy compatibility wrapper that points to the new
  contracts.
- Kiro adapter generation should prefer new generated reviewer names such as
  `spec-plan-auditor` and `spec-change-reviewer`. If the old
  `spec-independent-reviewer` generated file must remain for one release, mark
  it as a legacy alias and keep it read-only.
- `cli/crates/mochiflow-cli/tests/conformance.rs` pins the new vocabulary and
  removes brittle assertions that require the old S0-S4-only model.
- CLI behavior tests and presentation helpers that hardcode
  `.kiro/agents/spec-independent-reviewer.json` are covered by the migration
  task so init/join/upgrade and blocked-adapter output do not regress outside
  conformance tests.

## Data Model / Interfaces

- No JSON schema change is planned.
- No AC Matrix column change is planned.
- `spec.md ## QA Scenarios` interface changes from `Persona` to `Dimension` for
  new templates and guidance.
- `design.md ## Review Results` remains compatible:

```md
- Review profile: plan-auditor | change-reviewer
- Reviewer mode: delegated | inline
- Verdict: pass | pass-with-comments | fail
```

`Review profile` is documentation-level metadata. Existing lint and accept logic
continue to rely on `Reviewer mode` and `Verdict`.

## Error Handling

- If adapter generation would require deleting or renaming the existing Kiro
  generated reviewer target without either a compatibility alias or a fully
  tested migration, stop and return to plan instead of forcing the rename.
- If conformance tests depend on old stage labels, update them to assert the new
  canonical contracts with short, stable substrings to avoid line-wrap fragility.
- If implementation discovers that lint or accept must understand `Review
  profile`, stop and return to plan because that may cross into schema/contract
  behavior not agreed here.
- If the new reviewer split would require write or shell tools, reject that path;
  the reviewer contract must stay read-only.

## Test Strategy

- Add or update conformance tests for dimension vocabulary, `Dimension` template
  columns, risk-to-dimension coverage ownership, `plan-auditor` contract,
  `change-reviewer` contract, retirement of the public `independent-reviewer`
  name, retirement/aliasing of `plan-quality mode` and
  `post-implementation mode`, read-only transport, adapter resources, and
  compatibility of `Reviewer mode` / `Verdict`.
- Run `cargo test --manifest-path cli/Cargo.toml --test conformance` after
  engine prompt/template edits because existing tests pin engine prose.
- Run dogfood integrity steps after engine edits: `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check`.
- Run the configured `cli` surface verification before completion.
- Confirm `contracts/contracts.lock` remains unchanged unless implementation
  discovers an actual schema/golden-fixture contract change; if it changes, stop
  and return to plan.

## ADR Alignment

This spec intentionally supersedes active guidance from:

- `2026-07-01-grounded-independent-reviewer` - public labels currently stay
  `plan-quality mode` and `post-implementation mode`, with one reviewer prompt
  differing only by whether S3 exists.
- `2026-06-25-qa-attack-matrix` - QA attack coverage is currently persona-based
  (`P1`-`P7`) with `reference/risk.md` owning the risk-to-persona mapping.

During open's fold, add new ADR decision record(s) for the dimension model and
split reviewer contracts, mark the two records above as superseded with
reciprocal `superseded_by`, and regenerate the gitignored ADR indexes without
staging them.

## Review Results
