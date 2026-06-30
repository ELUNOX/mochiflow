# Define a compact standing router and verb-scoped engine loading — Tasks

Implementation Summary: Refactor engine routing/adapters text to make the standing-vs-lazy loading boundary explicit while preserving behavior.
risk: elevated
Critical Stop Conditions:
- Stop if preserving routing behavior requires a second router artifact or a new parsed route-card format.
- Stop if the change requires section-level references or a new context-budget CLI command.
- Stop if adapter output changes imply removing constitution or foundational context from the standing layer.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-04] Compact the router into a standing load contract
  - Depends on: none
  - Files:
    - `engine/router.md`
  - Done: `router.md` keeps the initial routing behavior and explicitly tells agents to load `commands/{verb}.md` plus command frontmatter `references` only after route selection; it does not introduce a second router artifact, section anchors, or context-budget command.
  - Stop: moving a routing invariant out of `router.md` would make initial routing ambiguous or dependent on a command file that has not yet been selected.
- [x] T-002 [AC-03, AC-05] Separate standing and load-on-demand sections in adapter templates
  - Depends on: T-001
  - Files:
    - `engine/adapters/agents/AGENTS.md.tpl`
    - `engine/adapters/claude-code/CLAUDE.md.tpl`
    - `engine/adapters/copilot/copilot-instructions.md.tpl`
    - `engine/adapters/kiro/steering/mochiflow.md.tpl`
  - Done: all adapter templates consistently distinguish standing inputs from load-on-demand engine procedure files; Kiro file references remain limited to router, constitution, and context; generated adapter semantics and target files remain unchanged. AGENTS/Kiro generated output is covered by this repo's adapter config, while Claude/Copilot are covered by source-template conformance unless the local config changes.
  - Stop: template wording needs adapter-specific behavior beyond prose changes or changes to adapter manifests.
- [x] T-003 [AC-01, AC-02, AC-03, AC-04] Add focused conformance coverage
  - Depends on: T-001, T-002
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: tests guard the router lazy-load contract, no-go decisions, adapter standing/load-on-demand wording, Kiro always-on references, and the existing routing parity cases without brittle long-prose matching. Routing parity evidence names explicit command routing, natural-language hints, backlog discuss promotion, backlog plan rejection, patch eligibility, review trigger, PR feedback/update routing, and merged-event close routing.
  - Stop: adequate coverage requires a new test harness or parser for command frontmatter.
- [x] T-004 [AC-05] Regenerate engine artifacts and verify
  - Depends on: T-003
  - Files:
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - generated adapter outputs if `mochiflow adapter generate` updates them
    - `.mochiflow/specs/engine-context-progressive-loading/spec.md`
    - `.mochiflow/specs/engine-context-progressive-loading/design.md`
    - `.mochiflow/specs/engine-context-progressive-loading/tasks.md`
  - Done: constitution-required `mochiflow freeze`, `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check` have run; the configured `cli` verification profile passes; generated-output verification is recorded for configured AGENTS/Kiro adapters and source-template coverage is recorded for Claude/Copilot; AC Matrix rows are updated with implementation paths, evidence, and results; mandatory elevated-risk review result is recorded in `design.md ## Review Results`.
  - Stop: freeze/adapter generation drift points to generated files outside the planned engine/adapters contract.
