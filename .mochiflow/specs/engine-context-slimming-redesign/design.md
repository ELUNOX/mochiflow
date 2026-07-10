# Redesign engine context loading and contract layout — Design

## Design Decisions

- **The router owns routing data and behavior.** Move explicit commands,
  natural-language hints, slug patterns, invalid numeric review forms, retired
  command handling, and delivery-event routes into one compact table in
  `router.md`. Remove `triggers` and `trigger_patterns` from command
  frontmatter. No CLI code currently consumes those fields; the existing
  conformance suite is their only non-engine consumer.
- **The standing layer is constitution plus router.** Adapter entrypoints retain
  pointers to the user-authored project/local constitution and the sole router.
  They describe project config, foundational context, commands, policies,
  templates, and ADR records as conditional inputs. Kiro removes the three
  foundational context file references from its always-on steering.
- **Use file-level progressive disclosure.** Do not introduce Markdown section
  anchors. Split policy files until each file is small enough to be a meaningful
  load unit and has one responsibility.
- **Make staged loading explicit in frontmatter.** Replace the flat
  `references` catalog with a `load.required` list and `load.conditional`
  entries containing a human-readable `when` condition and `files` list.
  Templates use the same conditional list. The router reads only the selected
  command, then the command's required list, then evaluates conditional entries.
- **Keep the execution model declarative.** The CLI does not parse, assemble, or
  serve prompt bundles. Markdown/frontmatter remains the agent contract; tests
  verify structure and referenced-path existence without measuring size.
- **Separate reviewer transport from reviewer judgment.** `reference/review.md`
  owns cadence, transport, verdict freshness, ad-hoc review, and review-fix
  behavior for the main workflow agent. `agents/reviewer-core.md` owns the
  common read-only audit method and output contract. The two profile files own
  only target-specific dimensions and inputs.
- **Delete compatibility-only engine files.** Delete the legacy
  `agents/independent-reviewer.md` wrapper and old monolithic reference paths
  once all live consumers move. Keep `DEPRECATED_KIRO_PATHS` cleanup in the CLI
  because it deletes obsolete generated user-worktree outputs during adapter
  regeneration; it is cleanup, not an engine instruction alias.
- **Preserve upgrade and public behavior.** The existing staged engine-directory
  replacement removes paths absent from the new source, and upgrade calls
  adapter generation afterward. No schema or CLI command change is required.
- **Separate feature delivery from release preparation.** Update non-frozen Rust
  comments and live engine/docs that call foundational context always-loaded,
  but do not edit `contracts/config.schema.json` or any version/release file in
  this feature. The frozen schema-description correction is recorded as a later
  release-preparation follow-up, when it can travel with the maintainer's chosen
  version, CHANGELOG/readme updates, derived lock outputs, and release branch.
- **Test contracts, not prose length.** Rehome behavioral assertions to the new
  owners, add structural checks for the load schema and live paths, and remove
  assertions whose only purpose is pinning the old prose layout. Do not add
  token, word, character, or file-size assertions.

## Architecture

### Standing and activation flow

```text
adapter entrypoint
  ├─ constitution.project + constitution.local
  └─ router.md
       ├─ no route → ordinary conversation
       └─ selected command
            ├─ command load.required
            ├─ resolve artifact/config condition
            └─ matching load.conditional files/templates only
```

Foundational context is loaded after selection when a command needs repository
orientation, or during ordinary work before making repository-specific claims.
Project config is loaded when route resolution, surfaces, verification, git, or
adapter paths require it.

### Target policy ownership

| Target file | Single responsibility | Source material moved from |
| --- | --- | --- |
| `reference/lifecycle.md` | asserted states, two approval gates, choice dispatch | `workflow.md`, router, plan/open summaries |
| `reference/specs.md` | artifact roles, depth, backlog promotion, authoring, consistency, session recovery | `workflow.md`, `authoring.md` |
| `reference/verification.md` | AC/Matrix contract, verification profiles, human/visual QA acceptance | `workflow.md`, open/build duplication |
| `reference/risk.md` | risk classification, design requirement, QA attack coverage | current `risk.md` |
| `reference/review.md` | reviewer cadence/transport, bounded fixes, freshness, ad-hoc review | current `risk.md`, command duplication |
| `reference/git.md` | branch, commit, trailers, explicit staging | current `git.md` |
| `reference/delivery.md` | persistence modes, PR handoff, derived state, post-merge cleanup | current `git.md`, open/update/close duplication |
| `reference/knowledge.md` | ADR lookup/fold/supersession and foundational-context refresh | current `git.md`, commands/adapters |
| `reference/language.md` | engine/artifact/conversation language and stable identifiers | current `language.md` |
| `reference/presentation.md` | generic user-facing summaries, action cards, internal-term suppression | router completion, command presentation sections |
| `reference/engineering-standards.md` | upstream-standard implementation discipline and instruction priority | unchanged responsibility |

`reference/workflow.md` and `reference/authoring.md` are deleted after their
content and consumers move. `reference/git.md`, `reference/risk.md`, and
`reference/language.md` keep their names but become narrower owners.

### Command load contract

Each command frontmatter uses this conceptual shape:

```yaml
load:
  required:
    - reference/lifecycle.md
  conditional:
    - when: risk is elevated or critical
      files:
        - reference/review.md
    - when: a standard-or-larger spec template is selected
      files:
        - templates/spec/spec.standard.md
```

The exact condition is durable prose evaluated from the minimum available
metadata. A file must not appear in both required and conditional lists for the
same command. Every declared path must exist. Mutually exclusive templates are
separate conditional entries.

Expected ownership by command:

| Command | Required policy | Conditional policy/templates |
| --- | --- | --- |
| discuss | specs, git, knowledge, presentation | pitch/spec metadata templates; language/engineering when needed |
| plan | lifecycle, specs, risk, verification, git, presentation | selected spec/design/tasks/handoff templates; review for offered review actions |
| build | lifecycle, verification, risk, git, engineering | review for elevated/critical or ad-hoc fixes |
| open | lifecycle, verification, delivery, knowledge, git | review by risk/freshness; PR template |
| update | delivery, verification, git | review by risk/freshness; PR template when metadata changes |
| close | delivery, presentation | no unrelated git/fold/verification policy |
| review | review, presentation | selected reviewer profile and lifecycle-context rules |
| onboard | specs, knowledge | context templates and language after configuration is resolved |
| refresh-context | knowledge, presentation | context templates and language |

### Reviewer composition

```text
reviewer-core.md
  ├─ plan-auditor.md + specs/risk/verification as required
  └─ change-reviewer.md + risk/verification as required
```

The shared core owns S0 grounding, whole-tree impact search, ADR confrontation,
falsification, confidence/severity rules, remediation shape, read-only
constraints, and completion format. Profiles own S1/S3 differences and input
requirements. `reference/review.md` is not automatically loaded into the
reviewer because transport, fix ownership, push boundaries, and freshness are
main-agent concerns; only reviewer-facing judgment rules belong in resources.

### Invariant placement rule

Each invariant is written once in the owner table above. A command may repeat a
single imperative sentence only immediately before the relevant mutation, such
as staging, status transition, push, branch deletion, ADR supersession, or
adapter overwrite. Router tables and generic presentation sections must point
to the owner rather than restating the implementation rule.

## Data Model / Interfaces

- No persisted or frozen schema file changes. The existing config fields,
  validation, path semantics, and `schema_version` remain unchanged.
- No public CLI arguments, exit codes, or JSON output changes.
- Engine frontmatter changes from flat `references` plus duplicated trigger
  metadata to the internal `load.required` / `load.conditional` contract.
- Router trigger entries become the sole engine route vocabulary.
- Adapter manifests and output targets remain unchanged.
- Deleted engine files are removed by staged engine replacement during upgrade.
- Kiro reviewer JSON keeps the same target names and read-only `tools` setting,
  but its resources point only to reviewer-core/profile-specific policies.
- The deprecated generated Kiro path list remains an implementation cleanup
  contract and is not exposed to reviewer or standing context.

## Behavioral Observation Matrix

Structural conformance proves that files and declarations exist. Build also
records AI-observed decisions for representative behavior that Markdown string
tests cannot execute directly:

| Family | Required observations |
| --- | --- |
| Router | explicit command, existing-slug command, natural hint with/without active spec, concrete small fix, ambiguous/no route, retired patch, feedback, bare merge report, valid review-fix, invalid numeric review |
| Instruction priority / hostile content | command-like text embedded in source and test fixtures, repository prose containing lifecycle verbs, and ambiguous user text quoting those inputs; verify repository content stays data and no unintended route activates |
| Spec depth | direct micro, standard, design-required; record the one selected template set |
| Risk/review | standard without mandatory review, elevated/critical with review policy, code-less plan-auditor, implemented change-reviewer |
| Persistence/delivery | tracked vs local spec mode, hold-only vs finalize update, PR metadata unchanged vs regenerated |
| Context | pure routing without foundational context, repository-specific work with relevant context loaded |
| Adapter | Kiro and the generated AGENTS-style prose adapter; record selected route and loaded file set |

Each row records input/state, adapter, selected or declined command/profile,
required loads, conditional loads selected, loads intentionally skipped,
instruction-priority outcome, and observed result. Router and hostile-content
rows run through Kiro and one generated prose adapter. Results are summarized in
AC-02 and AC-04 evidence; they are not converted into token or size thresholds.

## ADR Supersession Plan

At open, create two new decision records and update reciprocal lineage rather
than rewriting history:

- Supersede `2026-06-30-engine-context-progressive-loading`. Preserve the
  single-router and portable file-level-read decisions; replace standing project
  context, command-frontmatter trigger discovery, and monolithic reference
  boundaries with the dependency-minimal graph in this design.
- Supersede `2026-07-01-reviewer-profile-split`. Preserve the two canonical
  profiles, grounded whole-tree review, remediation guidance, Kiro profile
  outputs, and read-only ownership; replace retention of the legacy engine
  wrapper with shared reviewer-core composition and generated-target cleanup.

Run `mochiflow adr lint`, regenerate both gitignored ADR indexes, and verify no
ADR `INDEX.md` is staged during the open close-out.

## Deferred Release Follow-up

This feature intentionally does not choose or prepare a release. Before the next
release, create a release-scoped follow-up that:

- changes the frozen `contracts/config.schema.json` context description from
  always-loaded to conditional foundational orientation;
- follows the complete local maintainer release process, including version
  choice, `CHANGELOG.md`, README badges/install references, `Cargo.lock`, freeze,
  adapter generation, doctor, release branch, and release verification;
- keeps config shape, validation, paths, and `schema_version` unchanged.

Open records this follow-up durably as a backlog seed under
`{specs_dir}/_backlog/` (referencing `contracts/config.schema.json`), not only
in this design prose, rather than editing frozen/version files in the feature
branch.

## Error Handling

- If a required or conditional load path is absent, conformance fails before
  freeze/upgrade.
- If a command cannot resolve a conditional load from artifact/config state, it
  stops and asks one concise question rather than loading every alternative.
- If routing cannot choose between active specs, router disambiguation remains
  mandatory.
- If adapter regeneration encounters a user-owned structured target, upgrade
  keeps the installed engine, writes the existing candidate, reports adapter
  merge required, and exits non-zero as it does today.
- If staged engine installation fails before the swap, the existing engine stays
  in place; if the final rename fails, the existing backup is restored by the
  current upgrade implementation.
- If removing a duplicate exposes genuinely different behavior, stop and decide
  which behavior is canonical instead of silently merging the text.
- If a public CLI/schema/lifecycle behavior change becomes necessary, stop and
  return to plan; it is outside this refactor.

## Test Strategy

- Rewrite routing conformance around the router-owned route table and existing
  behavioral cases; commands no longer need trigger assertions.
- Observe command-like source/test-fixture content and ambiguous quoted user
  text through Kiro and an AGENTS-style adapter, recording that repository data
  does not activate a route and instruction priority remains intact.
- Execute the durable Behavioral Observation Matrix across every route family
  and representative conditional-load class, recording observed command/profile
  and loaded-file sets as AC-02/AC-04 evidence.
- Add structural tests that parse enough frontmatter to verify required and
  conditional paths exist, no command eagerly lists mutually exclusive
  templates, and no removed live path remains. Add a single-ownership
  graph-integrity check that treats `## Target policy ownership` as the owner
  map and asserts each migrated invariant resolves to exactly one owner file, so
  NFR-02/AC-06 has mechanical evidence rather than inspection alone. These are
  graph-integrity tests, not context-size budgets.
- Preserve lifecycle, approval, review-fix, freshness, git staging, delivery,
  fold, and language assertions by pointing them to their new owner.
- Verify reviewer profiles include the shared core, retain their distinct S1/S3
  responsibilities, remain read-only, and omit unrelated full-policy resources.
- Verify Kiro always-on steering references constitution and router but not the
  foundational context files.
- Preserve adapter managed-block, full-file, model-override, candidate, force,
  deprecated-target cleanup, upgrade, and deterministic regeneration tests.
- Exercise bundled and source-engine upgrades from an old-layout fixture and
  confirm deleted files do not survive the staged swap.
- Search config/init Rust sources, adapters, and live non-frozen documentation
  for stale foundational-context `always-loaded` claims; separately assert that
  frozen schema/version/release files are unchanged by this feature.
- Run `mochiflow freeze`, `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check` after source-engine edits.
- Run the configured CLI verification command:
  `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`.

## Review Results

No plan review has run yet.

## Integration Log

Build records only ownership drift, unexpected cross-file consumers, removed
path cleanup, and adapter/runtime constraints discovered while executing this
design. Do not record ordinary file moves or restate this plan.

### T-001 (policy graph created)

- The frozen-surface hash (`contracts/contracts.lock`) covers only
  `contracts/*.json` and `tests/conformance/golden/**`
  (`freeze::compute_contracts_hash`), not engine markdown or `MANIFEST.json`.
  Engine-layout edits therefore need `mochiflow freeze` to regenerate
  `MANIFEST.json` but never a version bump; the version triple stays driven by
  `cli/Cargo.toml`. This is why every task commit stays verifiable under
  `freeze --check` without touching version/release files.
- `conformance.rs` pins literal substrings inside `workflow.md` / `authoring.md`
  / `risk.md` / `git.md` / `language.md` and is not in T-001's file set, so the
  split is staged additive-first: the seven new owner files
  (`lifecycle`, `specs`, `verification`, `review`, `delivery`, `knowledge`,
  `presentation`) are created now with content moved verbatim, while the
  monoliths and the narrowed owners (`risk`, `git`, `language`) stay intact until
  their consumers and the matching conformance assertions migrate (T-002–T-006).
  No differing duplicate behavior was found; content was moved, not rewritten.

### T-002 (router self-sufficient; planning/setup commands migrated)

- PITFALL (recovered): the first pass of the router + discuss/plan/onboard/
  refresh-context edits was mistakenly applied to the gitignored vendored copy
  `.mochiflow/engine/` instead of the repo-root `engine/` source — the exact
  hazard the constitution warns about. It was caught by `cargo test`, which
  reads repo-root `engine/router.md` (the assertion failed because the source
  was unchanged). Fixed by re-applying every edit to repo-root `engine/` and
  running `upgrade --source engine --force` to discard the divergent vendored
  edits and re-sync from source. Guardrail for the rest of this build: all
  engine edits target repo-root `engine/`; the vendored copy is only regenerated.
- The router now owns a compact `## Route table` (all explicit commands, JA/EN
  natural-language hints, slug/event patterns) and routes from it without reading
  command frontmatter. The standing layer is reduced to constitution + router;
  foundational context/config are deferred. discuss/plan/onboard/refresh-context
  frontmatter moved from flat `references` to `load.required` / `load.conditional`
  (mutually exclusive spec templates split into separate conditional entries),
  triggers/trigger_patterns removed. Body cross-refs in those commands and the
  router were repointed to the new owners.
- Behavioral observation (router family, validated against the new table): every
  existing route — explicit `mochiflow-<verb>`, JA/EN hints, `{slug} <verb>`, the
  discuss-seed exception, plan-requires-draft, the small-fix→plan hint, ad-hoc
  review plus numeric review-fix forms, feedback→update, and merged→close cleanup
  — still selects the same command or the same clarification class; retired
  `mochiflow-patch` still routes to the plan proposal. The full adapter-channel
  matrix (Kiro + AGENTS) is consolidated in T-007.

### T-003 (implementation / delivery / review commands migrated)

- build/open/update/close/review frontmatter moved from flat `references` to
  `load.required` / `load.conditional` per the design ownership table (build:
  lifecycle/verification/risk/git/engineering + review conditional; open:
  lifecycle/verification/delivery/knowledge/git + review & PR-template
  conditional; update: delivery/verification/git + review & PR-template
  conditional; close: delivery/presentation; review: review/presentation +
  reviewer-profile conditional). triggers/trigger_patterns removed from all
  five; the router route table (T-002) is the sole route owner.
- `engine_open_update_close_defined_no_ship_verb` was updated: it no longer
  requires a `triggers:` block in the command files; it asserts the router route
  table owns each `mochiflow-<verb>` and its `commands/<verb>.md` target. The
  pr-description template needed no change (no old-owner references).
- Command bodies still cross-reference the not-yet-narrowed monoliths
  (`risk.md` / `git.md` / `workflow.md`); those refs still resolve and are
  repointed together with the risk.md/git.md narrowing and workflow/authoring
  deletion in T-006.
- PLAN-ACCURACY NOTE: narrowing `risk.md` / `git.md` / `language.md` (removing
  the content that moved to review/delivery/knowledge) is required by the
  approved design ("narrower owners") and by the AC-06 single-ownership check,
  but no task `Files:` list enumerates those three files. It is folded into T-006
  ("remove old owners / all live references use the new ownership graph") rather
  than re-planning, since the design already specifies it.


### T-004 (reviewer contracts consolidated)

- Created `engine/agents/reviewer-core.md` owning the shared review method once
  (S0 Grounding, S2 Impact & Regression, S4 Knowledge Confrontation,
  Falsification, operating rules, finding shape, completion output). Slimmed
  `plan-auditor.md` / `change-reviewer.md` to compose the core and carry only
  their target-specific S1/S3 stages and inputs; both use `load.required`
  (reviewer-core + risk) with language conditional.
- Deleted `engine/agents/independent-reviewer.md` and removed its only live
  reference (the compatibility-wrapper sentence in `risk.md ## Review
  transport`). The historical ADR / `_done` records that mention the old name are
  immutable and untouched. `adapter.rs` `DEPRECATED_KIRO_PATHS` still lists the
  generated `.kiro/agents/spec-independent-reviewer.json` target, so upgrade
  cleanup of that old generated file is unchanged (no adapter.rs edit needed).
- Kiro reviewer templates now resource only reviewer-core + the profile + risk +
  language (four), dropping workflow/authoring/git; the regenerated
  `.kiro/agents/spec-{plan-auditor,change-reviewer}.json` outputs are committed
  in sync so `adapter generate --check` stays at 0 drift.
- conformance: `canonical_reviewers_grounded_adversary_contract_is_pinned` now
  checks the shared method in reviewer-core and only S1/S3 in the profiles, drops
  the deleted-wrapper read, and repoints the session-recoverability assertion to
  `specs.md`; `kiro_reviewer_template_resources_are_grounded_and_read_only`
  expects the four-resource set and asserts the unrelated files are absent;
  `review_fix_loop_boundaries_are_pinned` splits profile-input vs core
  operating-rule checks.


### T-005 (adapter entrypoints slimmed; foundational context deferred)

- All four adapter templates now list only constitution + router as standing
  MochiFlow inputs. Foundational context (product/structure/tech) and project
  config moved into the load-on-demand section with an explicit "load when a
  workflow or repository-specific task needs orientation, not merely to route"
  condition. Kiro steering dropped the eager `#[[file:{{context.*}}]]`
  references (keeping only router + constitution `#[[file:]]`) and now describes
  context as a load-on-demand input. The load-on-demand reference list names the
  new owner set (lifecycle/specs/verification/risk/review/git/delivery/knowledge/
  language/presentation/engineering-standards).
- conformance `adapters_separate_standing_inputs_from_load_on_demand` now slices
  the standing vs load-on-demand sections and asserts context/config are absent
  from standing and present in load-on-demand, the new reference list is used,
  and Kiro carries no eager context `#[[file:]]` refs. cli.rs needed no change
  (no adapter-output content assertion depends on the moved wording).
- Regenerated tracked outputs `AGENTS.md` + `.kiro/steering/mochiflow.md` in sync
  (this repo generates the agents + kiro adapters); `adapter generate --check` =
  0 drift. Managed-block / full-file / model-override / candidate semantics are
  unchanged.
