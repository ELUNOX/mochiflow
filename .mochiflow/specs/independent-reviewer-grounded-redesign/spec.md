# Redesign the independent reviewer as a grounded adversary

## Background and Design Rationale

The independent reviewer (`engine/agents/independent-reviewer.md`) reads a spec as
a self-contained document. Its plan-quality mode is defined literally as "judge
the spec artifacts alone", it has no repository-grounding step, no whole-tree
impact sweep, and no ADR/pitfall input. As a result it structurally misses the
defect class that only appears when a spec is read as a change proposal against
the actual repository.

The `retire-patch-for-micro-spec-depth` review exposed this: several
High-severity conflicts (an existing `workflow.md` Depth-scaling micro row
contradicting the spec's redefinition; a `lint.rs` branch that cannot
distinguish a micro draft from a pitch-less standard draft; reverse-pinned
conformance tests; a recorded pitfall the change would re-trigger) were all
invisible to a reviewer that never leaves the spec folder.

**Chosen approach.** Recast the reviewer from a document proofreader into a
*grounded adversary* that reads the spec as a diff against repository reality —
applying mochiflow's "code is the source of truth" rule to the reviewer itself.
The procedure becomes an always-on core (Grounding, Internal Coherence, Impact &
Regression, Knowledge Confrontation, plus a cross-cutting Falsification pass) and
a Code Quality stage that is conditional on an implementation diff existing. The
two current modes stop being independent branches and collapse to a single
question — "is the Code Quality stage present" — while keeping their public
labels (`plan-quality mode`, `post-implementation mode`).

Findings gain a `Confidence` axis (`confirmed` vs `predicted`) so the reviewer
can raise grounded, code-verified defects to High/Critical (and drive `fail`)
while implementation-avoidable predictions are capped at Medium and never block
alone. Every finding must carry grounding evidence; anything unprovable drops to
an unverified note instead of a blocking claim.

**Rejected alternatives** (from the pitch): keep the two exclusive modes and
bolt the new stages onto plan-quality (double-manages the split, keeps the "spec
alone" blind spot); always run every stage regardless of diff (forces S3 `N/A`
noise); scope the impact sweep to `surfaces` + `Files` (makes coverage-gap
detection impossible); allow `predicted` findings to reach High (over-blocks at
plan time); pin conformance with multi-line substrings (re-triggers the recorded
line-wrap pitfall).

**Two design refinements discovered during plan grounding** (verified against
code, see `design.md`):

1. ADR `INDEX.md` files are a derived, gitignored cache (pitfall
   `2026-06-27-index-md-gitignored-derived-cache`) and the reviewer is generated
   for arbitrary consuming projects where those files may be absent. Making them
   *static* template `resources` is fragile, so ADR is loaded through the
   reviewer's existing `read` capability on demand (INDEX first) instead. The
   pitch's intent (S4 confronts ADR) is preserved; only the mechanism changes.
2. `commands/build.md` references only the reviewer *transport* and *cadence*,
   not the mode vocabulary, so it needs no edit. The mode-vocabulary update is
   scoped to `reference/risk.md`, `commands/plan.md`, and `commands/review.md`.

Origin: backlog seed `independent-reviewer-grounded-redesign` (discuss →
`pitch.md`).

## User Story

As a mochiflow maintainer relying on independent review as a quality assist, I
want the reviewer to read every spec as a change against the real repository —
grounding its claims in code, sweeping the whole tree for impact, and confronting
recorded decisions and pitfalls — so that out-of-spec conflicts like the
`retire-patch` breakages are caught before approval instead of slipping through a
"spec-alone" read.

## Scope

- In:
  - Recast `engine/agents/independent-reviewer.md` into the grounded-adversary
    stage model (S0-S4 + Falsification), redefine the two modes as "is S3
    present", add the `Confidence` finding axis and evidence discipline, and
    expand its frontmatter `references`.
  - Update reviewer mode vocabulary in `engine/reference/risk.md`,
    `engine/commands/plan.md`, and `engine/commands/review.md` to match the stage
    model.
  - Expand the Kiro reviewer template `resources`
    (`engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`) with the
    engine reference files the stages need; keep `tools` as `["read"]`; refresh
    its `description`.
  - Add self-conformance tests in `cli/crates/mochiflow-cli/tests/conformance.rs`
    using single-line heading/label and composition assertions.
  - Re-freeze so `engine/MANIFEST.json` matches the edited engine tree.
- Out:
  - Any change to `contracts/contracts.lock`, `engine/VERSION`, JSON schemas, or
    `tests/conformance/golden/**`.
  - Adding write or shell capability, or fine-grained Kiro tools
    (`grep`/`glob`/`bash`), to the reviewer.
  - Deterministic behavioral fixtures for the reviewer (it is an LLM prompt, not
    runnable logic).
  - `commands/build.md` mode-vocabulary edits (it carries none).
  - Regenerating the local dogfood copy under gitignored `.mochiflow/engine/`
    (an `upgrade`-time concern, not a tracked deliverable).

## Edge Cases

- A consuming project with **no ADR store**: the reviewer must still generate and
  load, and S4 reports "no ADR store found" / "no area-intersecting records
  found" rather than failing on a missing static resource.
- A consuming project with **ADR records but absent generated `INDEX.md`**: S4
  must not silently treat knowledge as empty. It reports the index as unavailable
  and either performs a bounded read-only directory scan of ADR records when the
  runtime exposes directory/search through `read`, or records an unverified
  knowledge-unavailable note when it cannot enumerate records.
- A **code-less spec** (plan-quality mode): the core stages run and S3 is
  reported `N/A (no implementation yet)`, never omitted.
- A **whole-tree impact sweep on a large repository**: verbatim reads are bounded
  to the spec's `surfaces` + declared `Files` + hit neighborhoods so the sweep
  stays tractable while remaining un-scoped in its *search*.
- A finding that **cannot be grounded**: it must be demoted to an unverified note,
  not raised as a blocking (High/Critical) finding.
- A user who has **pinned a custom review `model`** in the generated reviewer
  JSON: regeneration with the expanded `resources` must still preserve that model
  (existing adapter behavior must not regress).

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL define the independent reviewer's procedure as an
  always-on core of Grounding (S0), Internal Coherence (S1), Impact & Regression
  (S2), and Knowledge Confrontation (S4), plus a cross-cutting Falsification
  pass, WHERE Code Quality (S3) is conditional on an implementation diff
  existing.
- AC-02: THE SYSTEM SHALL define `plan-quality mode` as the core stages with S3
  reported `N/A`, and `post-implementation mode` as the core stages plus S3,
  retaining both mode labels.
- AC-03: WHEN reviewing any spec, THE SYSTEM SHALL require each current-state or
  change claim to be verified against code and require claims that cannot be
  grounded to be listed explicitly.
- AC-04: THE SYSTEM SHALL require the impact sweep to derive whole-repository
  search targets from the spec's current-state claims, changed concepts,
  retired/renamed terms, new or relocated responsibilities, contract/lifecycle
  vocabulary, declared files, surfaces, and AC nouns; search those targets across
  the whole repository (never scope-limited); report hits not covered by the
  tasks' declared `Files` or design scope as coverage-gap candidates; and, when
  no obvious target exists, perform a fallback sweep over the most distinctive
  nouns/identifiers from the spec rather than skipping S2, WHILE bounding
  verbatim reads to the spec's `surfaces`, declared `Files`, and hit
  neighborhoods.
- AC-05: THE SYSTEM SHALL require the reviewer to load area-intersecting ADR
  decisions and pitfalls via their `INDEX.md` first and to confront the spec
  against them; when no ADR store exists it reports no records found, and when
  ADR records exist but `INDEX.md` is unavailable it reports knowledge as
  unavailable and either performs a bounded read-only directory-scan fallback or
  records an unverified note rather than silently treating knowledge as empty.
- AC-06: THE SYSTEM SHALL add a `Confidence` field with values `confirmed` or
  `predicted` to the finding shape, WHERE a `confirmed` finding may be High or
  Critical and can drive a `fail` verdict, a `predicted` finding is capped at
  Medium and never alone causes `fail`, both require grounding evidence (a
  `path:line` and the observed fact), and an ungroundable suspicion is recorded
  as an unverified note rather than a blocking finding.
- AC-07: THE SYSTEM SHALL preserve the verdict rule: `fail` on any Critical or
  High finding, `pass-with-comments` on Medium or Low findings only, and `pass`
  when clean.
- AC-08: THE SYSTEM SHALL keep the Kiro reviewer template `tools` as exactly
  `["read"]` and SHALL NOT grant write or shell capability.
- AC-09: THE SYSTEM SHALL expand the Kiro reviewer template `resources` to
  include `reference/risk.md`, `reference/authoring.md`, and `reference/git.md`
  alongside the existing `agents/independent-reviewer.md`,
  `reference/workflow.md`, and `reference/language.md`.
- AC-10: THE SYSTEM SHALL NOT list ADR `INDEX.md` files as static template
  `resources`; instead the reviewer SHALL load them through its `read` capability
  at review time, so a project with absent or derived ADR indexes does not break
  reviewer generation or loading.
- AC-11: THE SYSTEM SHALL pin the redesign with conformance tests that assert the
  single-line stage headings/labels, the `Confidence` field, the preserved QA
  attack-coverage duty label, the template's `tools` and `resources` composition,
  and the reviewer doc frontmatter `references` composition, and SHALL NOT
  introduce multi-line substring assertions.
- AC-12: THE SYSTEM SHALL update the reviewer mode descriptions in
  `reference/risk.md`, `commands/plan.md`, and `commands/review.md` to match the
  stage model, AND SHALL update every by-name reference to the reviewer's former
  `Stage 1` / `Stage 2` in `reference/risk.md` — in both the `## Review transport`
  and `## QA attack coverage` sections — to the new stage names, without
  contradicting the reviewer doc, retaining the conformance-pinned phrase
  `reviewer's plan-quality mode` in `commands/plan.md`.
- AC-13: THE SYSTEM SHALL NOT modify `contracts/contracts.lock` or
  `engine/VERSION`, and freeze SHALL update only `engine/MANIFEST.json` among the
  tracked frozen files as a consequence of the engine-doc edits.
- AC-14: THE SYSTEM SHALL preserve, in S1, the reviewer's current Stage-1 duties —
  spec/AC conformance, the `session-recoverability` judgment, and the QA
  attack-coverage judgment defined by `reference/risk.md ## QA attack coverage` —
  rather than dropping them in the recast.
- AC-15: THE SYSTEM SHALL expand the reviewer doc frontmatter `references` to
  include `reference/authoring.md` and `reference/git.md` so its declared
  dependencies track the stages' actual use.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P3, P7 | cli | AI-observed | Replay the `retire-patch` review against the redesigned reviewer doc: for each prior High finding (workflow.md micro row conflict, lint.rs indistinguishable-draft branch, reverse-pinned conformance tests, re-triggered pitfall) identify which stage (S0/S2/S4) would now surface it. | Every replayed High-class defect maps to a concrete stage duty in the redesigned doc; none depends on "spec alone" reading. |
| QA-02 | P6 | cli | Automated | Run the `default` cli verify profile after all edits (`cargo test` + `fmt --check` + `clippy -D warnings` + `freeze --check`). | Existing reviewer-related tests (`plan_offers_pre_approval_review_before_confirm_for_elevated`, `session_recoverability_is_authoring_rule_not_lint`, adapter generate/upgrade tests) stay green; freeze is clean. |
| QA-03 | P5 | cli | Automated | Regenerate the Kiro reviewer over an existing project that pinned a custom review `model`, then run `adapter generate --check`. | The pinned `model` is preserved despite the expanded `resources`; generation is deterministic (no drift). |
| QA-04 | P1 | cli | Automated / AI-observed | Generate the reviewer for both a project with no ADR store and a project with ADR records but no generated `INDEX.md`. | The reviewer JSON is written with no ADR resource dependency and remains loadable; S4 reports no records for a missing store, and reports knowledge unavailable or uses a bounded directory-scan fallback when records exist but the index is absent. |
| QA-05 | P4 | cli | Automated | N/A check: inspect the change for any persisted data, migration, or state mutation. | N/A: docs + adapter template + Rust test change only; no data-integrity surface is touched. |
| QA-06 | P2 | cli | AI-observed | Read the redesigned S2 against a large-repo scenario: confirm the sweep derives targets for rename, additive, relocation, contract, lifecycle, and cross-cutting specs; remains un-scoped in search; and bounds verbatim reads to `surfaces` + `Files` + hit neighborhoods. | Target derivation and the cost-bounding rule are present and explicit, so a whole-tree grep stays tractable at scale without skipping non-rename specs. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- The `default` cli verification profile passes and its result is recorded.
- Engine-source synchronization checks required by the constitution pass after
  editing repo-root `engine/**`: `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check`. Only intended tracked deliverables are
  staged; generated gitignored dogfood files remain unstaged.
- For `risk: elevated`, an independent-reviewer verdict is recorded in
  `design.md ## Review Results`.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | new conformance test (stage headings) + QA-01 | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | Single-line heading assertions |
| AC-02 | cli | automated | new conformance test (mode labels) | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | Labels retained; internals = S3 presence |
| AC-03 | cli | automated | new conformance test (grounding label) + QA-01 | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | |
| AC-04 | cli | automated | new conformance test (impact-sweep target derivation + label) + QA-06 | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | Whole-tree search, non-rename target derivation, bounded reads |
| AC-05 | cli | automated | new conformance test (knowledge-confrontation label) + QA-01 | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | INDEX-first |
| AC-06 | cli | automated | new conformance test (Confidence field) | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | confirmed/predicted caps + evidence |
| AC-07 | cli | automated | new conformance test (verdict rule) | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | Preserved from current doc |
| AC-08 | cli | automated | new conformance test (tools composition) | `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`, `.kiro/agents/spec-independent-reviewer.json` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`kiro_reviewer_template_resources_are_grounded_and_read_only`) + `mochiflow adapter generate --check` | `tools == ["read"]` |
| AC-09 | cli | automated | new conformance test (resources composition) + QA-03 | `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`, `.kiro/agents/spec-independent-reviewer.json` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`kiro_reviewer_template_resources_are_grounded_and_read_only`, `behavioral_upgrade_*`, `kiro_agent_json_matches_reviewer_only`) + `mochiflow adapter generate --check` | 6 engine resource entries |
| AC-10 | cli | automated | new conformance test (no ADR resource) + QA-04 | `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`, `.kiro/agents/spec-independent-reviewer.json`, `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`kiro_reviewer_template_resources_are_grounded_and_read_only`, `independent_reviewer_grounded_adversary_contract_is_pinned`) + `mochiflow adapter generate --check` | ADR via read capability (doc side is AC-05) |
| AC-11 | cli | automated | the new conformance tests compile and pass; grep shows no multiline-substring assertion | `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`; `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` | Composition (tools/resources/references) + single-line only |
| AC-12 | cli | automated | new conformance test (cross-doc vocab) + `plan_offers_pre_approval_review_before_confirm_for_elevated` | `engine/reference/risk.md`, `engine/reference/authoring.md`, `engine/commands/plan.md`, `engine/commands/review.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`, `plan_offers_pre_approval_review_before_confirm_for_elevated`) | plan.md keeps pinned phrase; risk.md QA-coverage + transport refs |
| AC-13 | cli | automated | `freeze --check` in `default`; `git diff --name-only` review | `engine/MANIFEST.json` | PASS | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; `git diff -- contracts/contracts.lock engine/VERSION` showed no diff | contracts.lock / VERSION unchanged |
| AC-14 | cli | automated | new conformance test (QA attack-coverage duty) | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`, `session_recoverability_is_authoring_rule_not_lint`) | Sibling pin to session-recoverability |
| AC-15 | cli | automated | new conformance test (frontmatter references) | `engine/agents/independent-reviewer.md` | PASS | `cargo test --manifest-path cli/Cargo.toml` (`independent_reviewer_grounded_adversary_contract_is_pinned`) | +authoring.md, +git.md |
