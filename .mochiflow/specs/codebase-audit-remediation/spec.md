# Codebase Audit Remediation

## Background and Design Rationale

The repository-wide audit found thirteen high-confidence issues spanning one
Rust CLI surface and its delivery automation. The highest-leverage work is to
establish one repository-containment primitive, then reuse it for configured
paths and adapter manifests before addressing independent recovery,
performance, release, language, documentation, and test-architecture findings.
Keeping the work in one spec avoids introducing several subtly different path
validators and lets the final regression pass exercise the combined safety
model.

The maintainer selected one spec for all thirteen findings and approved
`risk: critical` after review identified security, release-publication
permissions, and destructive filesystem boundaries in the combined scope. Each
task therefore receives adversarial negative tests, strong verification
evidence, an independent change review through its final code-changing commit,
and an integration-log entry. Invalid configurations and manifests are rejected
rather than migrated silently. Existing valid layouts, including symlinks whose
resolved targets remain inside the repository, remain supported.

Rust path handling follows the Rust 1.96 standard-library contract:
`std::fs::canonicalize` resolves intermediate components and symbolic links,
while lexical `Path` operations alone do not. GitHub workflow hardening follows
GitHub's least-privilege token and immutable full-SHA guidance. cargo-dist 0.32
remains the release generator; its supported `pr-run-mode = "skip"` separates
tag publication from a small, repository-owned, path-filtered `dist plan`
workflow.

Language handling preserves the existing division of responsibility: the Rust
CLI remains deterministic, while onboarding AI resolves an uncertain artifact
language from repository evidence and persists a concrete BCP 47-style tag.
Conversation language remains `auto` unless explicitly fixed. This extends the
existing onboarding contract rather than adding an AI runtime dependency to the
CLI.

## User Story

As a solo MochiFlow maintainer, I want the audited safety, recovery,
performance, automation, language, documentation, and test weaknesses repaired
as one verifiable change, so that normal development and release operations are
safer without adding recurring CI cost or manual recovery traps.

## Scope

- In:
  - Repository containment for every configured repository-owned artifact or
    directory path used by a mutating command, including absolute paths,
    parent traversal, and symlink escapes.
  - Adapter allowlist and manifest template/output confinement shared by
    generate and detach.
  - Graceful malformed-scalar errors in spec metadata parsing.
  - Current split language flags and AI-owned artifact-language confirmation.
  - Single-snapshot index rendering.
  - Least-privilege, immutable, cost-filtered release and macOS automation with
    tag provenance checks.
  - Resumable accepted-but-uncommitted close-out.
  - Complete contributor verification instructions.
  - Freeze traversal error propagation.
  - Fail-closed engine replacement and explicit rollback recovery.
  - Semantic parsing for structural engine conformance contracts.
  - Required schema/version/frozen-artifact updates caused by tightening the
    public path contract.
- Out:
  - Instruction discovery/catalog features, context staleness detection, and a
    non-GitHub reference PR driver.
  - Runtime AI calls from the Rust CLI or a translation catalog for every
    language.
  - A new spec lifecycle, approval gate, adapter format, or PR provider.
  - Automatic rollback that rewrites unrelated work or staging state.
  - Per-PR, scheduled, or multi-architecture macOS testing.
  - A wholesale rewrite of prose-level conformance checks.
  - Protection against a concurrent, same-privilege local process replacing a
    validated path ancestor between containment checking and filesystem use.

## Edge Cases

- A destination does not yet exist but its nearest existing ancestor is a
  symlink outside the repository.
- A valid repository-local symlink is used for a configured directory.
- A concurrent local process replaces a validated ancestor between check and
  use; this is outside the stable-filesystem-namespace threat model.
- Windows drive, UNC, slash, backslash, dot, and parent components appear in a
  path supplied by config or an adapter manifest.
- An adapter name itself attempts to traverse out of `engine/adapters`.
- A template path remains inside the adapter but its output or candidate path
  escapes through a symlink.
- `spec.yaml` contains a lone single quote, lone double quote, empty quoted
  string, or unterminated quoted scalar.
- `accept` fails in a pre-commit hook after metadata and matrix files have been
  staged; a retry occurs with or without an unrelated staged file.
- An accepted close-out was already committed and `accept` is invoked again.
- An installed engine has no manifest, malformed JSON, a version mismatch,
  local drift, a failed swap, a failed rollback, or a failed backup cleanup.
- Recursive freeze traversal cannot read a directory entry after another
  derived file has been computed.
- Index generation runs in GitHub-provider mode with many active specs and a
  failed provider probe.
- Initialization has no language flag, no existing i18n config, and repository
  evidence in a language other than Japanese or English.
- A regional language tag such as `ja-JP` is used for deterministic CLI output,
  while an unsupported fixed CLI language falls back to English.
- A release tag is not reachable from `origin/main`, does not match the
  workspace version, or is created from a stale branch.
- A normal source-code PR changes no release metadata and therefore must not run
  cargo-dist planning or macOS tests.

## Acceptance Criteria (EARS)

- AC-01: WHEN config loading or a mutating command resolves any configured
  repository-owned artifact or directory path, THE SYSTEM SHALL reject empty,
  absolute, parent-traversing, or symlink-escaping paths before reading or
  mutating a location outside the canonical repository root, while accepting
  repository-local symlinks, under the documented stable-filesystem-namespace
  threat model. Executable/process configuration and explicit caller-supplied
  paths retain their separate contracts.
- AC-02: WHEN adapter configuration or a manifest is loaded, THE SYSTEM SHALL
  reject unknown adapter IDs and any template, output, candidate, generate, or
  detach path that escapes its allowed adapter or repository root under the
  documented stable-filesystem-namespace threat model.
- AC-03: WHEN spec metadata contains a lone, unterminated, or otherwise malformed
  quoted scalar, THE SYSTEM SHALL return a normal parse error without panicking,
  while continuing to accept valid empty quoted strings.
- AC-04: WHEN a user follows the public initialization documentation, THE SYSTEM
  SHALL show only supported `--artifact-language` and
  `--conversation-language` examples and accurately explain their separate
  responsibilities.
- AC-05: WHEN `mochiflow index` generates Markdown and JSON outputs, THE SYSTEM
  SHALL collect filesystem, Git, and provider delivery signals exactly once and
  render both outputs from the same snapshot without changing their public
  content contract.
- AC-06: WHEN release automation runs for a pull request or tag, THE SYSTEM
  SHALL use least-privilege job permissions, immutable action references, a
  locked and integrity-checked cargo-dist installation path, path-filtered plan
  validation, and a pre-publication check that the tag is reachable from
  `origin/main` and matches the workspace version.
- AC-07: IF `mochiflow accept` reaches `accepted` but staging or commit fails,
  THEN THE SYSTEM SHALL safely resume only the exact uncommitted close-out,
  rerun lint and final verification, reject unrelated changes, validate staged
  paths, and retry the commit without automatic rollback.
- AC-08: WHEN contributors read local verification instructions, THE SYSTEM
  SHALL identify the complete default test, formatting, Clippy, and freeze gate
  as completion verification and identify test-only execution as a fast loop.
- AC-09: IF recursive freeze manifest traversal cannot read a directory or entry,
  THEN THE SYSTEM SHALL return the failing path and abort before writing an
  incomplete manifest or related frozen artifact.
- AC-10: WHEN an existing installed engine lacks trustworthy manifest integrity
  or an engine swap cannot complete, THE SYSTEM SHALL stop unless overwrite
  intent is explicit, preserve recoverable backup data, and report rollback or
  cleanup failures with the recovery path.
- AC-11: WHEN initialization cannot reuse an explicit or existing artifact
  language, THE SYSTEM SHALL mark the provisional concrete value for AI review;
  onboarding SHALL determine and persist a concrete BCP 47-style artifact
  language from repository evidence, keep conversation language `auto` by
  default, and document deterministic CLI fallback without runtime AI calls.
- AC-12: WHEN conformance tests validate engine frontmatter, route tables, or
  load contracts, THE SYSTEM SHALL parse and compare their semantic structure
  so harmless line wrapping does not fail structural checks, while retaining
  intentional assertions for agent-visible wording.
- AC-13: WHEN relevant CLI, Cargo, or Rust toolchain files reach `main`, THE
  SYSTEM SHALL run the Rust test suite once on `macos-latest`; ordinary pull
  requests and unrelated main changes SHALL not consume macOS runner time, and
  manual dispatch SHALL remain available.

## QA Scenarios

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | Automated | Run targeted unit and integration tests for every changed module, then run the complete default verification profile. | All thirteen behaviors pass and the full existing suite remains green. |
| QA-02 | QA-UX | cli | Automated + AI-observed | Run init with explicit, existing, default, regional, and non-English language inputs; inspect human and JSON output plus English/Japanese docs. | Uncertain artifact language is handed to AI review, explicit values remain stable, conversation stays `auto` by default, and CLI output has no mixed-language empty state. |
| QA-03 | QA-ABUSE | cli | Automated | Exercise absolute paths, Unix and Windows parent components, unknown adapter IDs, malicious manifest mappings, and inside/outside symlinks against config, generate, candidate, and detach operations; detach a local link pointing to an in-repository sentinel. | Every escape is rejected before outside mutation; valid repository-local mappings and symlinks still work; deleting the local link preserves its resolved sentinel target. |
| QA-04 | QA-DATA | cli | Automated | Force an accept hook failure, retry with the exact close-out, retry with unrelated staging, and exercise missing/malformed/drifted engine manifests plus swap and rollback failures. | Exact recovery succeeds; unrelated state blocks; engine backups are never silently lost and recovery locations are reported. |
| QA-05 | QA-COMPAT | cli | Automated | Validate tightened config schema fixtures, current adapter manifests, generated/frozen version artifacts, cargo-dist plan output, and semantic engine load/route contracts. | Existing valid projects and shipped adapters remain compatible; invalid paths fail; frozen/version gates and structural contracts agree. |
| QA-06 | QA-RESIL | cli | Automated | Inject freeze traversal failure, failed provider probes, duplicate index generation pressure, engine rename failure, and backup cleanup failure. | Freeze/engine filesystem errors are explicit and write no partial authoritative output; provider unavailability retains the existing graceful fallback and unchanged Markdown/JSON contract while the probe remains single-pass and deterministic. |
| QA-07 | QA-REG | cli | Automated + AI-observed | Run `cargo test`, format check, Clippy with denied warnings, freeze check, adapter regeneration and drift check after engine sync, spec lint, doctor, and cargo-deny; inspect both contributor guides, the pull-request template, and their focused conformance assertions. | No regression, generated drift, supply-chain violation, or new health-check failure remains; all three contributor-facing locations identify the complete default gate, and both guides separately label test-only execution as the fast loop. |
| QA-08 | QA-COMPAT, QA-REG | cli | Automated + AI-observed | Inspect workflow triggers and permissions, run local `dist plan`, test valid/invalid tag provenance logic, and validate the macOS workflow's event/path matrix. | Release planning runs only for release metadata changes, publication is tag-only and provenance-gated, and macOS runs only for relevant main changes or manual dispatch. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Every task records strong evidence as test output, logs, or explicit human
  confirmation for its applicable QA scenarios.
- After every task, the critical change review records `pass` or
  `pass-with-comments` and `Reviewed through: <sha>` in `design.md` through that
  task's final code-changing commit, and the required integration-log entry is
  appended before the next task starts.
- Any `engine/` edit is followed by `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate` followed by
  `mochiflow adapter generate --check` before final verification.
- Any frozen contract edit includes the required workspace patch-version bump
  and coherent Cargo, engine, contract-lock, changelog, and public install
  references.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-03, QA-05 | `config.rs`, repository-owned configured-path mutation call sites, `config.schema.json` | UNVERIFIED | | excludes executable/caller paths; check-only witness preserves local-link semantics |
| AC-02 | cli | automated | QA-01, QA-03, QA-05 | `adapter.rs`, `detach.rs`, adapter integration tests | UNVERIFIED | | one checked manifest resolver |
| AC-03 | cli | automated | QA-01 | `spec_meta.rs` parser tests | UNVERIFIED | | malformed quotes never panic |
| AC-04 | cli | automated + AI-observed | QA-02, QA-07 | getting-started and configuration docs, CLI help conformance | UNVERIFIED | | removed flag absent |
| AC-05 | cli | automated | QA-01, QA-06, QA-07 | `index.rs`, delivery probe test seam | UNVERIFIED | | one snapshot, unchanged render contract |
| AC-06 | cli | automated + AI-observed | QA-05, QA-08 | dist config, workflow-used provenance helper with dynamic Git fixtures, and GitHub release workflows | UNVERIFIED | | least privilege and tag provenance |
| AC-07 | cli | automated | QA-01, QA-04, QA-07 | `accept.rs`, accept integration fixtures | UNVERIFIED | | retry exact accepted close-out only |
| AC-08 | cli | automated + AI-observed | QA-07 | conformance assertions and direct inspection of both contributor guides plus the pull-request template | UNVERIFIED | | complete default gate everywhere; guides distinguish fast loop |
| AC-09 | cli | automated | QA-01, QA-06 | `freeze.rs` recursive error propagation | UNVERIFIED | | no partial authoritative writes |
| AC-10 | cli | automated | QA-01, QA-04, QA-06 | `upgrade.rs`, `doctor.rs`, init/join/upgrade tests | UNVERIFIED | | explicit force and recoverable backup |
| AC-11 | cli | automated + AI-observed | QA-02, QA-07 | init/presentation/config language code, onboard engine contract, public docs | UNVERIFIED | | AI persists concrete artifact language |
| AC-12 | cli | automated | QA-01, QA-05, QA-07 | conformance semantic parser and selected migrated assertions | UNVERIFIED | | prose behavior guards retained |
| AC-13 | cli | automated + AI-observed | QA-08 | cost-filtered macOS workflow | UNVERIFIED | | main/path/manual event matrix |
