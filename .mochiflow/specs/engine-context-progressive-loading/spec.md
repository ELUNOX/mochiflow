# Define a compact standing router and verb-scoped engine loading

## Background and Design Rationale

MochiFlow's engine guidance has grown across `router.md`, `commands/*.md`,
`reference/*.md`, templates, reviewer prompts, and adapter templates. The engine
is already structurally layered, but the generated adapter entrypoints still
present router, verb procedures, and cross-cutting references in one nearby
catalog. That shape can encourage cautious agents to load broad engine context
before the initial route is known.

The chosen approach is to make `engine/router.md` the compact standing router
instead of creating a second `router.card.md`. A second route card would add a
new generated artifact that could drift from the authoritative router. The
router remains the only initial route contract; detailed command and reference
rules are loaded after routing selects a lifecycle verb or non-phase command.

The standing layer continues to include the project constitution and
foundational context. Those are not engine procedure details: constitution is
user-authored standing guidance, and context is the code-derived current-state
orientation. ADR records remain on-demand historical rationale, loaded by store
index and relevant active records only.

This spec came from the `engine-context-progressive-loading` backlog seed. It
absorbs the seed's v1 boundary: compact the standing router and clarify adapter
loading language, while deferring section-level references and context-budget
commands.

## User Story

As a developer using MochiFlow through an AI coding tool, I want the generated
instructions to load only the routing layer up front and defer detailed rules
until the workflow path is known, so that the agent stays procedural without
burying important rules in unrelated context.

## Scope

- In:
  - Compact `engine/router.md` so it carries the initial routing and loading
    contract without duplicating detailed command procedures.
  - Update adapter templates for AGENTS, Claude Code, Copilot, and Kiro so
    always-loaded and load-on-demand inputs are visibly distinct.
  - Preserve the standing constitution/context layer and on-demand ADR access
    rule.
  - Add conformance coverage for adapter wording and routing parity cases that
    protect the lazy-loading contract.
  - Run the engine dogfood sync steps required by the constitution after editing
    `engine/`.
- Out:
  - Creating `router.card.md` or any second router artifact.
  - Adding section-level frontmatter references such as `reference/git.md#Branch`.
  - Adding a `context audit`, `guide --context-budget`, or other token-budget
    CLI command.
  - Delegating implementation to write-capable workers or changing reviewer
    transport.
  - Removing constitution or foundational context from the standing layer.

## Edge Cases

- Kiro steering is always-on and uses file references, so it must keep file
  references only for standing inputs and describe command/reference files as
  load-on-demand paths.
- Markdown adapters are appended into user-owned files, so the wording must be
  concise and not require users to read long generated blocks before acting.
- Existing command frontmatter remains file-level. The plan must not rely on
  section anchors that some agent surfaces cannot read consistently.
- Router compaction must not weaken ambiguous-intent handling, raw backlog seed
  routing, patch eligibility handoff, active spec resolution, review transport,
  or delivery event routing.
- Generated adapter outputs and frozen engine artifacts drift when source
  templates change; verification must include regeneration checks.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL make `engine/router.md` the only standing router
  artifact and SHALL describe the initial route decision plus the rule to
  lazy-load `commands/{verb}.md` and that command's frontmatter `references`
  after a lifecycle verb or non-phase command is selected.
- AC-02: THE SYSTEM SHALL NOT introduce `router.card.md`, section-level
  reference anchors, or a context-budget CLI command as part of this change.
- AC-03: THE SYSTEM SHALL make AGENTS, Claude Code, Copilot, and Kiro adapter
  templates visibly separate always-loaded inputs from load-on-demand engine
  procedure files.
- AC-04: WHEN the router handles explicit commands, natural-language hints,
  raw backlog seeds, patch eligibility, review triggers, PR feedback, and merge
  events, THE SYSTEM SHALL preserve the existing routing behavior.
- AC-05: WHEN engine source or adapter templates are changed, THE SYSTEM SHALL
  regenerate required frozen/generated artifacts and pass the configured CLI
  verification checks.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1 new user | cli | AI-observed | Read generated AGENTS-style instructions as a first-time MochiFlow user and identify what must be loaded before routing. | The standing inputs are obvious and the user is not instructed to eagerly read every command/reference file. |
| QA-02 | P2 power user | cli | Automated | Exercise conformance assertions for explicit commands and slug patterns, including `mochiflow-plan` and `{slug} discuss`. | Fast command routing remains unambiguous and does not require broad eager loading. |
| QA-03 | P3 malicious user | cli | Automated / AI-observed | Check ambiguous or concrete-small-fix language against patch/spec routing rules. | Ambiguous intent does not activate a spec verb, and concrete small fixes still route through patch eligibility. |
| QA-04 | P4 data integrity | cli | Automated / AI-observed | Inspect the change set for persisted project data, spec schema, config schema, or ADR record format changes. | N/A: this refactor changes engine instructions, adapter templates, and tests only; it does not migrate user data or alter persisted schemas. |
| QA-05 | P5 migration | cli | Automated | Run adapter generation/check coverage against existing Markdown targets and Kiro full-file steering behavior. | Existing adapter target semantics are preserved; generated content changes intentionally without requiring migration. |
| QA-06 | P6 regression | cli | Automated | Run the `cli` verification profile after engine source edits and dogfood sync. | Existing CLI, lint, adapter, and freeze checks pass. |
| QA-07 | P7 spec skeptic | cli | AI-observed | Compare final router/adapters against this spec's In/Out boundaries and AC list. | The implementation matches the compact-router, verb-scoped loading contract and does not include out-of-scope artifacts or commands. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- The constitution-required engine sync steps are complete after source-engine
  edits.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | AI-observed + automated | QA-01, QA-07; conformance assertions for router load contract | `engine/router.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `router_defines_lazy_load_contract_without_second_card`; reviewer verdict pass | Router remains the sole standing route artifact. |
| AC-02 | cli | AI-observed + automated | QA-07; repository search for out-of-scope files/commands/anchors | `engine/`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `router_defines_lazy_load_contract_without_second_card`; reviewer verdict pass | No `engine/router.card.md` was introduced; section-anchor and context-budget command scope stayed out. |
| AC-03 | cli | AI-observed + automated | QA-01, QA-05; source-template assertions for AGENTS, Claude Code, Copilot, and Kiro; generated-output checks for configured adapters | `engine/adapters/agents/AGENTS.md.tpl`; `engine/adapters/claude-code/CLAUDE.md.tpl`; `engine/adapters/copilot/copilot-instructions.md.tpl`; `engine/adapters/kiro/steering/mochiflow.md.tpl`; `cli/crates/mochiflow-cli/tests/conformance.rs`; `AGENTS.md`; `.kiro/steering/mochiflow.md` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `adapters_separate_standing_inputs_from_load_on_demand`; `mochiflow adapter generate --check`; reviewer verdict pass | Project config generates AGENTS + Kiro only; Claude/Copilot are covered at source-template conformance level unless the local adapter config changes. |
| AC-04 | cli | automated | QA-02, QA-03, QA-06; conformance assertions naming explicit command routing, natural-language hints, backlog discuss promotion, backlog plan rejection, patch eligibility, review trigger, PR feedback/update routing, and merged-event close routing | `engine/router.md`; `engine/commands/*.md`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml`; `router_preserves_named_routing_branches`; `router_plan_requires_existing_draft_spec`; `router_merged_event_is_cleanup_only`; `pr_feedback_routes_to_update_without_restore`; reviewer verdict pass | Existing behavior is preserved while wording changes; each listed routing branch has named evidence. |
| AC-05 | cli | automated | QA-06; full configured verification profile; dogfood sync; adapter generation check for configured adapters plus source-template conformance for non-configured adapters | `engine/MANIFEST.json`; `.mochiflow/engine/`; `AGENTS.md`; `.kiro/steering/mochiflow.md`; adapter templates | PASS | `mochiflow freeze`; `mochiflow upgrade --source engine`; `mochiflow adapter generate --check`; `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`; reviewer verdict pass | Local config exercises generated output for AGENTS + Kiro; Claude/Copilot remain template-level coverage in this repo. |
