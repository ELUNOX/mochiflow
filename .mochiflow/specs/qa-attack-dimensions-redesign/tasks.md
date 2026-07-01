# Redesign QA attack coverage and independent review contracts - Tasks

Implementation Summary: Replace persona QA guidance with dimension coverage and split the reviewer contract into plan audit and change review profiles.
risk: elevated
Critical Stop Conditions:
- Stop if the reviewer rename cannot provide either a compatibility alias or a fully tested migration.
- Stop if the change requires a schema or `contracts.lock` update.
- Stop if a reviewer contract needs write or shell capability.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03] Replace persona QA guidance with dimensions
  - Depends on: none
  - Files:
    - `engine/reference/risk.md`
    - `engine/reference/authoring.md`
    - `engine/commands/plan.md`
    - `engine/templates/spec/spec.md`
    - `engine/templates/spec/spec.standard.md`
    - `engine/templates/spec/design.md`
  - Done: Risk owns the `QA-FUNC` / `QA-UX` / `QA-ABUSE` / `QA-DATA` / `QA-COMPAT` / `QA-RESIL` / `QA-REG` mapping; authoring and templates use `Dimension` instead of `Persona`; `QA-XX` remains the scenario ID; conformance-asserted phrases stay on single lines where needed.
  - Stop: any need for semantic CLI lint or AC Matrix schema changes.
- [x] T-002 [AC-01, AC-04, AC-05, AC-06, AC-07] Split and rename review contracts
  - Depends on: T-001
  - Files:
    - `engine/agents/plan-auditor.md`
    - `engine/agents/change-reviewer.md`
    - `engine/agents/independent-reviewer.md`
    - `engine/reference/risk.md`
    - `engine/reference/workflow.md`
    - `engine/commands/plan.md`
    - `engine/commands/review.md`
    - `engine/commands/build.md`
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/router.md`
    - `engine/README.md`
    - `README.md`
    - `docs/configuration.md`
  - Done: `plan-auditor` is the canonical code-less spec/design/task/QA/ADR audit; `change-reviewer` is the canonical post-implementation code review with refactor safety; both contracts use QA dimensions instead of personas, preserve repository grounding and whole-tree impact/regression search, and document claim evidence requirements; `independent-reviewer` is no longer public/canonical and exists only as a documented legacy alias/wrapper when needed; `plan-quality mode` and `post-implementation mode` are retired as public terms or explicitly mapped as legacy aliases; `plan.md` pre-approval review wording uses the new naming; review remains read-only and `Reviewer mode` / `Verdict` remain accepted record fields.
  - Stop: the split changes lifecycle gates, reviewer write permissions, or accept/lint verdict parsing.
- [x] T-003 [AC-08] Update Kiro adapter resources and reviewer artifact names
  - Depends on: T-002
  - Files:
    - `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`
    - `engine/adapters/kiro/agents/spec-plan-auditor.json.tpl`
    - `engine/adapters/kiro/agents/spec-change-reviewer.json.tpl`
    - `engine/adapters/kiro/manifest.toml`
    - `cli/crates/mochiflow-core/src/adapter.rs`
    - `cli/crates/mochiflow-core/src/present.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Generated Kiro reviewer targets use the new canonical review names, remain read-only (`tools: ["read"]`), and either alias or deliberately migrate the old `spec-independent-reviewer` target with conformance coverage; CLI behavior tests and presentation output that hardcode `.kiro/agents/spec-independent-reviewer.json` are updated for the chosen alias/migration behavior; resources/tools remain pinned without static ADR index resources.
  - Stop: Kiro tools require anything beyond coarse `read`, or adapter output rename lacks a tested compatibility/migration path.
- [x] T-004 [AC-01, AC-04, AC-05, AC-06, AC-07, AC-08] Update conformance guards for new review vocabulary
  - Depends on: T-001, T-002, T-003
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-core/src/present.rs`
  - Done: Conformance no longer requires the old single `S0-S4 + S3 N/A` contract as the public model; tests assert dimension coverage, `plan-auditor`, `change-reviewer`, preserved grounding/whole-tree impact duties, retirement of public `independent-reviewer` naming, retirement or legacy-alias mapping for `plan-quality mode` / `post-implementation mode`, compatibility wrapper/alias behavior, and unchanged `Reviewer mode` / `Verdict` parsing; no runtime lint/accept behavior is changed unless tests reveal a pure wording fixture update.
  - Stop: implementation needs to change accepted status or verdict parsing semantics.
- [x] T-005 [AC-09] Regenerate engine artifacts and verify
  - Depends on: T-001, T-002, T-003, T-004
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/**`
    - `.kiro/agents/spec-independent-reviewer.json`
    - `.kiro/agents/spec-plan-auditor.json`
    - `.kiro/agents/spec-change-reviewer.json`
    - `AGENTS.md`
    - `CLAUDE.md`
    - `.github/copilot-instructions.md`
    - `.mochiflow/specs/qa-attack-dimensions-redesign/spec.md`
    - `.mochiflow/specs/qa-attack-dimensions-redesign/design.md`
    - `.mochiflow/specs/qa-attack-dimensions-redesign/tasks.md`
  - Done: Run `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, `mochiflow lint --spec qa-attack-dimensions-redesign`, and the configured `cli` surface verification; confirm `contracts/contracts.lock` is unchanged unless an approved schema/golden contract change was added; record AC Matrix evidence; prepare open-fold notes to supersede ADR decisions `2026-07-01-grounded-independent-reviewer` and `2026-06-25-qa-attack-matrix`; commit only tracked generated outputs and never stage gitignored `INDEX.md`.
  - Stop: generated adapter drift is unrelated to this spec or default verification fails for an unrelated environmental reason.
