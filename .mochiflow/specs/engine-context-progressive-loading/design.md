# Define a compact standing router and verb-scoped engine loading — Design

## Design Decisions

- **Keep `engine/router.md` as the single standing route artifact.** The router
  should become compact enough to read up front, but it remains authoritative.
  A generated `router.card.md` is rejected because it would create a second
  routing source with drift risk.
- **Move detail by ownership, not by word-count alone.** Initial route
  selection stays in `router.md`; phase procedure detail stays in
  `commands/*.md`; shared policy remains in `reference/*.md`; artifact shapes
  remain in `templates/*`; read-only review remains in
  `agents/independent-reviewer.md`.
- **Adapters express load tiers explicitly.** Generated instructions should use
  labels such as standing inputs and load-on-demand inputs instead of one flat
  catalog. Kiro's `#[[file:...]]` references should remain limited to router,
  constitution, and context.
- **Preserve file-level references in v1.** Command frontmatter continues to
  reference whole files. Section-level anchors are deferred because agent/tool
  surfaces do not all provide stable section reads.
- **Use conformance tests as behavior guards.** Tests should assert the loading
  contract and critical routing parity without matching large prose blocks that
  make later wording improvements brittle.

## Architecture

The implementation touches three existing layers:

- `engine/router.md`: the standing routing contract loaded by adapter
  entrypoints. It should retain activation strength, patch/spec routing, active
  spec resolution, state/intent conflict handling, review transport boundary,
  PR feedback/merge event routing, and the command-selected lazy-load rule.
- `engine/adapters/**`: generated entrypoint templates. They should describe
  standing inputs and load-on-demand inputs consistently across AGENTS, Claude
  Code, Copilot, and Kiro.
- `cli/crates/mochiflow-cli/tests/conformance.rs` and adapter-generation tests:
  source-level and generated-output guards for the router/adapters contract.

The source of truth for engine edits is repo-root `engine/`. The dogfood
vendored copy under `.mochiflow/engine/` is regenerated from source after engine
edits, per the constitution.

## Data Model / Interfaces

- No config schema, spec schema, ADR record schema, or public CLI contract
  changes are planned.
- Adapter output text changes are user-facing generated artifact changes. The
  file targets and adapter manifests remain unchanged.
- Router command selection continues to depend on command frontmatter
  `triggers` and `trigger_patterns`; the implementation does not add a parsed
  route-card format.

## Error Handling

- If router compaction exposes a rule that cannot be moved without changing
  behavior, keep that rule in `router.md` and record the reason in the build
  notes or final AC evidence.
- If adapter generated output becomes too verbose while separating load tiers,
  prefer concise labels and references over explanatory prose.
- If conformance tests become brittle because they match long prose, replace
  them with targeted substring checks for stable contract phrases.
- If a new command, schema, migration, or section-anchor mechanism appears
  necessary, stop implementation and route back to plan because it exceeds this
  spec's v1 boundary.

## Test Strategy

- Add or update conformance assertions that verify:
  - router names `commands/{verb}.md` plus frontmatter references as
    command-selected lazy-loaded inputs;
  - raw backlog seeds still route to discuss and do not activate plan;
  - patch eligibility, review trigger, feedback, and merge event routing remain
    covered;
  - generated adapter templates distinguish standing inputs from load-on-demand
    inputs;
  - Kiro always-on steering file references only standing inputs.
- Run focused lint during planning and the full configured `cli` verification
  profile during build.
- After source-engine edits, run the constitution-required dogfood sync:
  `mochiflow freeze`, `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check`.

## Review Results

- Reviewer mode: delegated
  Verdict: pass
  Scope: full branch diff plus current generated/formatting changes after
  T-001 through T-004.
  Evidence: independent-reviewer reported no Stage 1 spec conformance findings
  and no Stage 2 code quality findings.
