# Agent Context API

## Background and Design Rationale

MochiFlow exposes repository state through several surfaces that answer
different questions. `status` renders a human delivery board, `ready` checks
build entry, `lint` reports consistency failures, and `index` writes a derived
JSON board. AI coding agents therefore have no single side-effect-free contract
for resolving the active spec, distinguishing unavailable observations from
negative facts, or determining which lifecycle actions are currently allowed.

The agreed solution is a versioned CLI JSON API backed by one shared read-only
snapshot model. `inspect` provides a lightweight repository view without a slug
and a detailed diagnostic view with a slug. Rust owns deterministic repository
facts and lifecycle eligibility; `engine/router.md` remains the sole owner of
natural-language intent and route selection; command documents continue to own
execution procedures. Existing `status`, `index`, `ready`, `lint`, and
`state/index.json` surfaces remain compatible and progressively consume the
shared model instead of maintaining parallel state logic.

The JSON contract follows JSON Schema Draft 2020-12, matching the repository's
existing schemas and the official specification published on 2022-06-16:
https://json-schema.org/draft/2020-12. Git observations use the installed Git
2.50.1 capabilities documented by the official `git-for-each-ref` and pretty
format manuals; provider batching uses the installed GitHub CLI 2.96.0
`gh pr list --json` contract. These batch interfaces make process and network
probe counts independent of the number of specs while keeping O(N) file
discovery acceptable.

This is an additive public contract and a responsibility relocation across the
CLI state surfaces, so it remains `risk: elevated` and `integration: contract`.
Per the repository versioning policy, the new frozen schema is released as the
feature version `1.3.0`; it does not change consumer `schema_version` because no
existing config or spec format is broken.

## User Story

As an AI coding agent operating in a MochiFlow repository, I want one stable,
side-effect-free context API for repository and spec state, so that I can select
and validate workflow actions without reconstructing state from prose or
multiple command outputs.

## Scope

- In:
  - Add `mochiflow inspect [slug] [--json] [--fetch]` with repository and
    per-spec views.
  - Add a frozen `contracts/agent-context.schema.json` Draft 2020-12 contract
    covering successful, degraded, partial, and error documents.
  - Represent Git/provider observations and lifecycle eligibility as known,
    unknown, or not applicable where a Boolean would lose information.
  - Evaluate deterministic eligibility for `discuss`, `plan`, `build`, `open`,
    `update`, and `close`; keep workflow suggestions distinct from human next
    actions.
  - Resolve active spec binding from deterministic branch evidence and expose
    unresolved or ambiguous candidates without modification-time guessing.
  - Introduce a shared repository/spec snapshot consumed by `inspect`,
    `status`, `index`, and `ready` while preserving existing public output.
  - Batch Git refs, base-branch trailers, ignore classification, worktree state,
    and GitHub PR state so probes do not scale per spec.
  - Expose structured lint/readiness findings without printing during snapshot
    construction.
  - Sanitize machine output to stable identifiers, safe messages, and
    repository-relative normalized paths.
  - Update engine guidance so routing selects intent and consults the API for
    deterministic eligibility before a lifecycle procedure runs.
  - Apply the feature-version and frozen-artifact protocol, documentation, and
    dogfood engine synchronization.
- Out:
  - HTTP, MCP, daemon, socket, background, or persistent service surfaces.
  - Prompt assembly, Markdown body embedding, automatic context loading, or
    token budgeting.
  - A persistent context cache or generated `agent-context.json` file.
  - Natural-language classification or route selection in Rust.
  - Eligibility for `review`, `onboard`, `refresh-context`, or other non-phase
    commands.
  - Verification Receipt, recovery automation, Provider Driver v2, or
    adapter-specific automatic invocation.
  - New lifecycle statuses, spec directory moves, or persisted delivery facts.
  - Removal or incompatible output changes for existing CLI commands or
    `state/index.json`.

## Edge Cases

- No config exists or config parsing fails before normal command dispatch.
- The repository has no specs, one branch-bound spec, multiple plausible specs,
  a detached HEAD, or a branch that does not match a spec branch.
- A spec directory is missing `spec.yaml`, contains malformed YAML, or contains
  metadata whose slug disagrees with the directory name.
- Valid and malformed specs coexist; the repository response must preserve the
  valid entries while making the malformed entry impossible to overlook.
- The requested slug is absent, ambiguous between active and legacy archived
  paths, or contains a path-like value outside the slug contract.
- Git refs, base refs, worktree status, ignore classification, or trailer
  traversal cannot be observed.
- GitHub CLI is absent, unauthenticated, offline, returns malformed JSON, or
  reaches the configured batch limit.
- `provider = none` makes provider observations not applicable rather than
  unknown.
- An accepted spec has an open PR, a merged PR, only a base-reachable trailer,
  or only a local-mode branch-tip merge signal.
- An approved spec has no tasks, has unchecked tasks, has all tasks checked but
  unsettled automated verification, or has complete build evidence.
- An elevated spec lacks a required current implementation review result.
- `--fetch` fails after older local refs remain available.
- Dirty paths contain spaces, non-ASCII characters, backslashes, or names that
  resemble credentials; output remains normalized without file contents.
- Human output is localized while machine identifiers and JSON enums remain
  stable English tokens.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL expose `mochiflow inspect --json` as a lightweight
  repository snapshot and `mochiflow inspect <slug> --json` as a detailed spec
  snapshot, with every success document validating against the versioned Agent
  Context JSON Schema.
- AC-02: WHEN a valid slug is requested, THE SYSTEM SHALL report its metadata,
  persistence mode, structured health findings, tri-state delivery signals,
  six lifecycle eligibility results, workflow suggestion, human next action,
  and related repository-relative paths without embedding source contents.
- AC-03: WHEN lifecycle eligibility is evaluated, THE SYSTEM SHALL treat Rust
  as the deterministic eligibility authority for `discuss`, `plan`, `build`,
  `open`, `update`, and `close`, reuse one shared readiness core from `ready`
  and engine preconditions, expose branch/worktree entry checks only through
  the lifecycle action result, and leave natural-language intent selection
  exclusively to the router.
- AC-04: IF a Git or provider fact cannot be observed, THEN THE SYSTEM SHALL
  preserve it as unknown with a stable warning code, keep usable local facts,
  return `result: degraded` with exit 0 when no repository integrity error
  exists, and never reinterpret unknown as a confirmed negative fact.
- AC-05: WHEN inspection encounters malformed repository specs, a malformed
  requested spec, a missing slug, or invalid config, THE SYSTEM SHALL emit one
  schema-valid JSON document in JSON mode and apply the agreed `partial` or
  `error` result and exit code without mixing diagnostics into stdout.
- AC-06: WHILE `--fetch` is absent, THE SYSTEM SHALL perform no file write,
  generated-artifact refresh, Git ref update, staging, commit, push, or provider
  mutation; WHEN `--fetch` is present, THE SYSTEM SHALL attempt only the
  explicit origin refresh and degrade safely if it fails.
- AC-07: WHEN paths, dirty state, blockers, warnings, or process failures are
  represented, THE SYSTEM SHALL emit stable machine codes and normalized
  repository-relative paths while excluding absolute machine paths, file
  contents, diffs, configured command bodies, raw stderr, and credential-like
  provider details.
- AC-08: WHEN `status`, `index`, and `ready` consume the shared snapshot and
  eligibility model, THE SYSTEM SHALL preserve their existing human output,
  exit behavior, golden dashboard, and `state/index.json` contract, including
  legacy `ready` success when its existing lint/status/verification checks pass
  despite a missing expected branch or unrelated worktree dirt.
- AC-09: WHEN a repository contains any number of specs, THE SYSTEM SHALL use
  O(N) file discovery but a constant-bounded set of Git and provider batch
  probes, detect provider result truncation, and avoid a per-spec subprocess or
  network call pattern.
- AC-10: WHEN the current branch deterministically binds one spec and its state
  yields one unambiguous next workflow, THE SYSTEM SHALL report that binding and
  suggestion; OTHERWISE THE SYSTEM SHALL return unresolved or ambiguous
  candidates and no guessed workflow, while keeping `update` unsuggested without
  explicit feedback intent.
- AC-11: WHEN the new public schema is introduced, THE SYSTEM SHALL release it
  coherently as version `1.3.0`, update all public version references and frozen
  artifacts, retain config `schema_version`, and pass schema, manifest, version,
  engine-sync, adapter-drift, and supply-chain checks.

## QA Scenarios

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | Automated | Exercise repository and detailed inspection across empty, draft, approved-incomplete, build-complete, accepted-ready, in-review, merged, and cleanup-pending fixtures. | Each document selects the correct scope, state, eligibility, suggestion, and next action and validates against the public schema. |
| QA-02 | QA-UX | cli | Automated | Run human and JSON modes in English and Japanese configurations, including success and error paths. | Human output is concise and localized; JSON identifiers remain stable; stdout contains exactly one JSON document in JSON mode. |
| QA-03 | QA-ABUSE | cli | Automated | Supply path-like or unknown slugs, dirty paths with unusual characters, malformed process output, raw stderr containing absolute paths or credential-like text, and malicious spec metadata. | Inputs do not escape the configured spec boundary; output contains no file body, raw stderr, command body, secret-like value, or absolute repository path. |
| QA-04 | QA-DATA | cli | Automated | Snapshot tracked and local-only fixtures before and after inspection, with and without a failing `--fetch`. | Ordinary inspection changes no file or ref; `--fetch` changes only fetched refs when successful; failure preserves usable local state and returns a warning. |
| QA-05 | QA-COMPAT | cli | Automated | Compare `status`, `index`, `ready`, dashboard golden, and `state/index.json` before and after shared-snapshot migration; validate all Agent Context fixtures. | Existing contracts and exits remain byte/semantically compatible as applicable, and the additive schema accepts every supported response variant. |
| QA-06 | QA-RESIL | cli | Automated | Use many-spec fixtures with counted fake Git/provider runners; exercise missing base refs, offline/unauthenticated provider, malformed provider JSON, and a full batch result at the configured limit. | Probe counts remain constant-bounded, unknowns remain explicit, truncation is warned, and valid local facts survive every degraded case. |
| QA-07 | QA-REG | cli | Automated + AI-observed | Run focused unit/integration tests, the complete default gate, cargo-deny, engine freeze/upgrade, adapter generation check, spec lint, and a final source review. | No regression, generated drift, contract/version mismatch, supply-chain failure, or duplicated lifecycle authority remains. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- The required elevated-risk change review records a passing result through the
  final code-changing commit.
- The new schema, release version, engine manifest, contract lock, public docs,
  and generated adapter state agree.
- Direct inspection and compatibility commands leave no unexpected worktree or
  generated-file changes.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-05 | `contracts/agent-context.schema.json`, `inspect.rs`, CLI conformance tests | PASS | `schema_agent_context_accepts_all_result_variants`; `inspect_json_is_one_schema_valid_document` | repository and spec variants |
| AC-02 | cli | automated | QA-01, QA-03 | `inspect.rs`, structured lint and persistence APIs | PASS | live schema-valid detail test asserts structured payload; schema fixtures pin observation qualities | no content embedding |
| AC-03 | cli | automated + AI-observed | QA-01, QA-07 | eligibility evaluator, `ready`, router/command contracts | PASS | canonical suite 202 tests; `engine_keeps_agent_context_eligibility_separate_from_intent` | six lifecycle actions only |
| AC-04 | cli | automated | QA-06 | delivery observation snapshot and warning model | PASS | `failed_git_batches_remain_unavailable`; `provider_truncation_never_becomes_known_false` | tri-state facts |
| AC-05 | cli | automated | QA-01, QA-02 | inspect presenters and config/error dispatch | PASS | `inspect_json_is_one_schema_valid_document`; `inspect_missing_slug_returns_structured_error`; Japanese human-output test | partial exits 1 |
| AC-06 | cli | automated | QA-04 | inspect command and fetch boundary | PASS | read-only snapshot construction; full status/index mutation regression suite | no persistent cache |
| AC-07 | cli | automated | QA-03 | serialization and diagnostic sanitization | PASS | negative absolute-path fixture; path-like slug structured-error behavior | repository-relative paths |
| AC-08 | cli | automated | QA-05, QA-07 | `status.rs`, `index.rs`, `ready`, shared snapshot | PASS | unchanged index golden, status read-only tests, ready conformance, full default suite | existing JSON unchanged |
| AC-09 | cli | automated | QA-06 | batched Git/provider collector | PASS | `repository_probe_count_is_independent_of_spec_count` with 0 vs 25 specs | O(N) files only |
| AC-10 | cli | automated | QA-01 | active resolution and suggestion policy | PASS | exact expected-branch implementation; suggestion precedence unit test; full inspect CLI tests | update never auto-suggested |
| AC-11 | cli | automated + AI-observed | QA-05, QA-07 | versioned contracts, docs, freeze and adapter artifacts | PASS | default gate; cargo-deny all categories ok; adapter 0 drift; spec/ADR lint 0 fail | config schema version unchanged |
