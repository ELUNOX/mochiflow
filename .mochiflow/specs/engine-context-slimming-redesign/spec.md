# Redesign engine context loading and contract layout

## Background and Design Rationale

MochiFlow already distinguishes a standing router from command-selected engine
documents, but the effective load graph remains broad. The generated adapter
entrypoints eagerly name the constitution, all three foundational context files,
the router, config, every command family, and the monolithic cross-cutting
references. After activation, most lifecycle commands load `workflow.md`,
`risk.md`, or `git.md` in full even though each command needs only part of those
files. The two reviewer profiles each bootstrap the same five full references
before they inspect a spec or implementation diff.

The previous `engine-context-progressive-loading` work made the standing and
on-demand tiers explicit but deliberately kept foundational context standing,
kept file-level monoliths, and retained trigger metadata in every command. This
spec supersedes those internal layout boundaries. It preserves the successful
decision to use one authoritative router and portable file-level reads, while
redesigning which information each file owns and when it is loaded.

The chosen design has four foundations:

1. `router.md` is self-sufficient for routing. It owns the compact trigger table
   and does not inspect every command before selecting one.
2. Standing context contains only user-authored constitution plus the router.
   Project config and foundational context are loaded when a selected workflow
   or ordinary repository-specific work actually needs them.
3. Cross-cutting policy is divided into responsibility-sized files. Command and
   reviewer frontmatter distinguishes immediately required files from files and
   templates whose conditions must be evaluated first.
4. Every invariant has one owning reference and, where safety requires it, one
   concise reminder at the mutation boundary. Internal path compatibility is
   not retained because `mochiflow upgrade` replaces the engine wholesale and
   regenerates adapters.

The redesign intentionally uses structural loading rules instead of token,
word, or file-size budgets. It does not add a prompt compiler or runtime context
assembly subsystem.

The current Rust source comments and frozen config schema description call
foundational context always-loaded. Runtime/source terminology moves to the new
conditional model in this feature. The frozen schema description is deliberately
deferred to a later release-preparation change so this spec does not force an
early version bump; config keys, validation, path behavior, and `schema_version`
remain unchanged.

A post-implementation adversarial review found that the initial build left two
live `always-loaded` claims, retained activation vocabulary in command
frontmatter, and missed one deleted-owner cross-reference. It also found that
the approved T-006 file list did not reconstruct the owner-narrowing commit.
This re-plan returns the spec to `draft`, reconciles that historical task scope,
and adds one corrective task without changing the product contract or design.

## User Story

As a developer using MochiFlow through any supported AI coding tool, I want the
agent to load only the instructions needed for the current decision or workflow
step, so that repository context, specifications, diffs, and verification
evidence receive the available attention without weakening workflow safety.

## Scope

- In:
  - Make `engine/router.md` the sole source of route triggers and route selection.
  - Remove trigger and trigger-pattern duplication from command frontmatter.
  - Defer foundational project context and project config until the selected
    workflow or repository-specific task needs them.
  - Replace monolithic workflow/authoring/git/review ownership with the
    responsibility map defined in `design.md`.
  - Introduce required and conditional load tiers for commands and reviewers.
  - Consolidate common reviewer behavior and output into one reviewer core.
  - Remove the legacy `independent-reviewer.md` wrapper and obsolete internal
    reference paths without compatibility stubs.
  - Slim all generated adapter entrypoints and Kiro reviewer resources to the
    redesigned load graph.
  - Consolidate generic presentation and completion guidance while keeping
    phase-specific user actions in their owning commands.
  - Update engine documentation and structural/behavioral conformance coverage.
  - Correct non-frozen `always-loaded` terminology in Rust config/init sources
    and live engine/docs without changing config shape or behavior.
  - Remove exact activation vocabulary from command frontmatter descriptions so
    route tokens and natural-language hints remain owned only by `router.md`.
  - Reject removed owner references even when they omit the `reference/` prefix,
    and guard live foundational-context terminology outside the frozen schema.
  - Record the frozen config-schema description correction as a required later
    release-preparation follow-up.
  - Regenerate the engine manifest, dogfood vendored engine, and configured
    adapter outputs.
- Out:
  - Token, word-count, character-count, or file-size budgets and tests.
  - A context audit, prompt bundle, prompt compiler, or context assembly CLI.
  - Section-anchor references into Markdown files.
  - Public CLI command changes or config/spec/ADR/manifest/PR-request schema
    shape, validation, compatibility, or `schema_version` changes.
  - Editing `contracts/config.schema.json`, choosing the next version, release
    branch preparation, CHANGELOG/badge/install-reference updates, tagging, or
    publishing.
  - Lifecycle, approval-gate, review ownership, verification, persistence-mode,
    git-safety, or PR delivery behavior changes.
  - Stronger reviewer-evidence validation or other unrelated workflow features.
  - Compatibility files or aliases for removed engine-internal Markdown paths.

## Edge Cases

- A natural-language message must be routable without opening every command
  file, including ambiguous intent, concrete small fixes, review-fix numeric
  forms, PR feedback, and bare merge reports.
- Command files may identify their procedure and responsibilities, but their
  frontmatter descriptions must not repeat explicit commands, natural-language
  hints, or slug/event patterns from the router-owned route table.
- A command may need metadata such as risk, spec depth, persistence mode, or
  delivery state before it can choose a conditional reference. The command must
  read only the minimal artifact or config value needed to evaluate that
  condition, then load the selected file.
- Ordinary repository questions that do not activate MochiFlow still need
  foundational context when they require repository orientation; deferral must
  not mean that current-state claims are made without reading code and relevant
  context.
- Kiro steering is always-on and expands file references eagerly. It must keep
  constitution and router references but must not include foundational context
  references in the always-on steering file.
- Generated Markdown adapters update managed blocks, while structured adapter
  targets can be blocked and emitted as merge candidates. The existing
  adapter-merge-required behavior must remain intact.
- Removing `engine/agents/independent-reviewer.md` must not remove the CLI's
  deprecated generated-target cleanup for old Kiro files; upgrade should delete
  markered obsolete generated targets rather than preserve them as aliases.
- `mochiflow upgrade` must remove deleted engine files through the existing
  staged directory replacement and regenerate adapters against the new paths.
- The frozen config schema will temporarily retain its prior descriptive wording
  until the later release-preparation follow-up; implementation must not edit it
  or any version/release file in this feature.
- Conformance tests that currently pin prose in old monolithic files must move
  to the new owner without weakening the behavior they protect.
- Engine edits change `MANIFEST.json` and the dogfood vendored copy. Generated
  artifacts must be synchronized before final verification.

## Non-Functional Requirements

- NFR-01: Engine reads SHALL remain portable across supported adapters by using
  complete file paths rather than Markdown section anchors.
- NFR-02: Every workflow invariant SHALL have one documented owner and SHALL be
  repeated only at a mutation boundary where the local guard prevents an unsafe
  action.
- NFR-03: Every command and reviewer SHALL be independently understandable from
  its own procedure/profile plus its declared required and selected conditional
  loads, without relying on conversation history.
- NFR-04: The redesign SHALL preserve managed adapter ownership, safe upgrade
  replacement, and deterministic regeneration.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL load only the configured constitution files and the
  authoritative router as standing MochiFlow instructions, and SHALL defer
  project config and foundational context until routing or repository-specific
  work requires them, with non-frozen source and documentation reflecting that
  conditional role.
- AC-02: WHEN an explicit command, slug form, natural-language hint, retired
  command, PR-feedback event, review form, or merge report is received, THE
  SYSTEM SHALL select or decline the route from `router.md` without first
  loading every command file and SHALL preserve the current routing behavior.
- AC-03: THE SYSTEM SHALL organize shared engine policy into the responsibility
  files defined by the design contract and SHALL remove the superseded
  monolithic `workflow.md` and `authoring.md` paths after all consumers migrate.
- AC-04: WHEN a command is activated, THE SYSTEM SHALL load its declared required
  policies immediately and SHALL load conditional policies and mutually
  exclusive templates only after their conditions are resolved.
- AC-05: WHEN plan or implementation review runs, THE SYSTEM SHALL bootstrap the
  selected reviewer profile with the shared reviewer core and only its required
  review policies, while preserving repository grounding, whole-tree impact
  search, QA confrontation, read-only ownership, and the current verdict format.
- AC-06: THE SYSTEM SHALL define lifecycle, verification, risk/QA, review,
  git, delivery, knowledge, language, and presentation invariants in one owner
  each and SHALL retain concise safety reminders only at relevant mutation
  boundaries.
- AC-07: WHEN an installed project runs `mochiflow upgrade`, THE SYSTEM SHALL
  replace the old engine layout, regenerate adapters against the new paths,
  preserve existing adapter collision handling, and require no internal-path
  compatibility stubs.
- AC-08: WHEN the redesigned engine and adapters are verified, THE SYSTEM SHALL
  pass routing, lifecycle, reviewer, adapter-generation, upgrade, lint, format,
  clippy, freeze, and dogfood synchronization checks without changing frozen
  schema/version surfaces or adding any context size or token-budget test.

## QA Scenarios

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | Automated + AI-observed | Exercise explicit commands, slug patterns, natural hints, ambiguous requests, feedback, review-fix forms, and merge reports using the compact route table. | Every existing route selects the same command or asks the same class of clarification without pre-reading command files. |
| QA-02 | QA-UX | cli | AI-observed | Read each generated adapter entrypoint as a first-turn instruction and follow it through one lifecycle activation. | Constitution and router are clearly standing; config, project context, command policy, templates, and ADR records are loaded only when needed. |
| QA-03 | QA-ABUSE | cli | Automated + AI-observed | Present source files, test fixtures, or user text containing command-like instructions and ambiguous lifecycle vocabulary. | Repository content remains data, ambiguous intent does not activate a verb, and instruction priority remains unchanged. |
| QA-04 | QA-DATA | cli | Automated | Inspect frozen schemas/version files, spec persistence behavior, upgrade staging, and generated-state paths before and after the refactor. | Frozen schemas/version files are unchanged; keys, validation, `schema_version`, persisted lifecycle data, and path behavior remain unchanged. |
| QA-05 | QA-COMPAT | cli | Automated | Upgrade a fixture containing the old engine layout and generated adapters, including a blocked structured target and a deprecated markered Kiro reviewer target. | Removed engine files disappear, current adapters regenerate, blocked targets produce candidates, and deprecated generated targets are cleaned without legacy engine stubs. |
| QA-06 | QA-RESIL | cli | Automated | Run engine drift, adapter check, failed/blocked adapter, missing conditional reference, and stale generated-output scenarios. | Diagnostics identify the actionable path; staged engine replacement remains recoverable and no broken reference graph is accepted. |
| QA-07 | QA-REG | cli | Automated | Run the full CLI suite plus focused structural checks for route ownership, declared file existence, conditional loads, reviewer resources, and removed legacy paths. | Existing workflow behavior stays green and the new ownership graph is internally coherent without prose-length assertions. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token before acceptance.
- Every removed path has no remaining live engine, adapter, documentation, or
  conformance reference except explicit deprecated generated-target cleanup.
- All required generated and vendored artifacts are synchronized from repo-root
  `engine/`.
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated + AI-observed | QA-02, QA-04; adapter outputs and non-frozen terminology checks | `engine/router.md`; `engine/adapters/**`; config/init sources; T-008 live-claim cleanup | UNVERIFIED | Prior evidence remains the baseline; T-008 must add a live-engine terminology guard and rerun it after correcting `refresh-context.md` and `specs.md`. | The frozen schema stays intentionally unchanged. |
| AC-02 | cli | automated + AI-observed | QA-01, QA-03; routing conformance plus hostile-content behavioral observations | `engine/router.md`; `engine/commands/*.md`; adapters; conformance tests | UNVERIFIED | T-008 must prove command frontmatter contains no router-owned activation vocabulary while preserving the router table and observed routing results. | Command procedure names and body references remain allowed; the activation catalog does not. |
| AC-03 | cli | automated + inspection | QA-07; declared-reference existence and stale-path search | `engine/reference/**`; engine documentation | UNVERIFIED | T-008 must correct `engineering-standards.md` and strengthen removed-path coverage to catch bare deleted-owner names. | Historical specs, ADRs, fixtures, and explicit deprecated generated-target cleanup remain allowed exceptions. |
| AC-04 | cli | automated + AI-observed | QA-01, QA-02, QA-06; load-structure checks plus conditional-load observation matrix | `engine/commands/*.md`; selected templates | UNVERIFIED | Re-run the staged-load structural checks and Behavioral Observation Matrix after T-008 changes command frontmatter. | No load contract or routing behavior change is planned. |
| AC-05 | cli | automated + independent review | QA-06, QA-07; reviewer profile/resource assertions | `engine/agents/**`; Kiro reviewer templates | UNVERIFIED | The adversarial review correctly failed the stale implementation; a fresh delegated change-reviewer pass is required after T-008. | Reviewer behavior remains read-only and grounded. |
| AC-06 | cli | automated + inspection | QA-07; owner-map and duplicate-contract review; single-ownership structural check | `engine/reference/**`; commands; templates | UNVERIFIED | T-006 scope is reconciled in `tasks.md`; T-008 removes the remaining duplicate route vocabulary and broken owner reference, then reruns ownership checks. | One owner plus local mutation guards; no size/token budget. |
| AC-07 | cli | automated | QA-04, QA-05; bundled/source upgrade and adapter collision fixtures | upgrade/adapter behavior; engine/adapters layout | UNVERIFIED | Re-run bundled/source upgrade tests and the live dogfood upgrade after T-008 regenerates the engine manifest. | Deprecated target cleanup remains code-owned. |
| AC-08 | cli | automated | QA-04, QA-06, QA-07; configured verification, frozen-surface no-change check, and dogfood sync | generated/vendored artifacts and spec evidence | UNVERIFIED | T-008 must run the full configured verification, `freeze`, dogfood upgrade, adapter drift check, spec lint, and fresh review. | Frozen schema/version files, budget tests, and measurement commands remain out of scope. |
