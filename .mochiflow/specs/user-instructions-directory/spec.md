# Add user-owned MochiFlow instructions directories

## Background and Design Rationale

Users need a discoverable place under `.mochiflow/` for release procedures,
checklists, and other Markdown that should be read only when they explicitly
name its path. The current local fallback, `constitution.local.md`, is an
always-loaded rules file, so occasional operating notes placed there consume
standing context and blur the boundary between project rules and optional user
material.

The chosen design adds two user-owned directories without adding a loader,
registry, schema, or configuration surface:

- `.mochiflow/instructions/` is shareable and remains a normal Git tracking
  candidate.
- `.mochiflow/instructions.local/` is private by default through
  `.mochiflow/.gitignore`.

`init` creates both directories and seeds an English
`.mochiflow/instructions/README.md` only when that file is absent. The README
explains that MochiFlow does not automatically load, parse, index, validate, or
drift-check these files and that a user must explicitly cite a file path when
they want an agent to use it. Once created, the README and all other contents
are user-owned; even `init --force` must not overwrite them.

The local ignore rule is part of the safety contract. A fresh or forced init
writes `instructions.local/` in `.mochiflow/.gitignore`. When init encounters an
existing install-level ignore file without `--force`, it preserves the existing
content and appends only the missing `instructions.local/` rule. It never adds
`instructions/` to an ignore file.

The directories are intentionally outside MochiFlow's runtime model. They are
not config paths, standing inputs, adapter inputs, state, engine content, or
generated indexes. `join` and `upgrade` do not create or inspect them. `doctor`,
`freeze`, adapter generation, and engine drift checks do not require or inspect
them. A repeated explicit `init` may create missing scaffolding, but no other
repair or upgrade command backfills it.

Normal `detach` preserves both directories as user project data. Purge already
removes the entire `.mochiflow/` tree; its prompt, pre-action warning, and
reports must now name both instruction directories so users understand that
their own Markdown will be deleted. The warning must be emitted before any
removal without contaminating JSON stdout.

This repository will dogfood the boundary by moving its local maintainer
release procedure from the always-loaded `.mochiflow/constitution.local.md` to
`.mochiflow/instructions.local/release.md` and restoring the local constitution
to its default stub. Those ignored local files are working-copy state, not PR
content.

## User Story

As a MochiFlow user, I want an obvious shared and local-only place for optional
Markdown instructions, so that I can keep reusable operating notes near the
workflow without making MochiFlow load or manage them.

## Scope

- In: scaffold `.mochiflow/instructions/` and
  `.mochiflow/instructions.local/` during `init`.
- In: seed a user-owned `instructions/README.md` that documents explicit
  path-based use and the non-managed contract.
- In: keep `instructions/` trackable and ignore `instructions.local/`, including
  safely appending the missing local rule to an existing install-level ignore
  file during an explicit init.
- In: make init output and public English/Japanese docs explain the two paths.
- In: preserve both directories during normal detach and explicitly warn that
  purge deletes them.
- In: move this repository's ignored maintainer release notes into
  `instructions.local/release.md`.
- Out: frontmatter, `SKILL.md`, indexes, manifests, registries, loaders, search,
  filename rules, or plugin behavior.
- Out: config schema fields or path overrides for either directory.
- Out: adapter, router, or constitution references to the directories.
- Out: join or upgrade backfill and doctor/freeze/drift/adapter inspection.

## Edge Cases

- If `instructions/README.md` already contains user edits, repeated init and
  `init --force` leave it byte-for-byte unchanged.
- If either directory is absent in an existing project, an explicit init may
  create it; join and upgrade leave it absent.
- If `.mochiflow/.gitignore` already contains custom content, init without
  `--force` retains that content and appends `instructions.local/` only when the
  exact rule is missing.
- If `instructions/` or `instructions.local/` is occupied by a regular file,
  init fails clearly rather than deleting or replacing that file.
- Arbitrary Markdown names and contents, including files with frontmatter-like
  text, remain opaque to MochiFlow.
- Untracked Markdown under `instructions/` remains visible to normal Git status
  and may therefore block clean-tree delivery checks until the user tracks,
  ignores elsewhere, or removes it.
- Normal detach retains both shared and local instructions; confirmed purge
  removes both with the rest of `.mochiflow/`.

## Acceptance Criteria (EARS)

- AC-01: WHEN `mochiflow init` completes, THE SYSTEM SHALL create
  `.mochiflow/instructions/`, `.mochiflow/instructions.local/`, and an English
  `.mochiflow/instructions/README.md` when they are missing.
- AC-02: WHERE `.mochiflow/instructions/README.md` or any other instruction file
  already exists, THE SYSTEM SHALL preserve it unchanged during repeated init
  and `init --force`.
- AC-03: WHEN init manages `.mochiflow/.gitignore`, THE SYSTEM SHALL ensure an
  `instructions.local/` rule exists while preserving existing custom content
  without `--force`, AND SHALL NOT add an `instructions/` ignore rule.
- AC-04: THE SYSTEM SHALL keep both instruction directories outside automatic
  loading, parsing, indexing, validation, config, engine drift, doctor, freeze,
  adapter generation, join, and upgrade behavior.
- AC-05: WHEN normal detach runs, THE SYSTEM SHALL preserve both instruction
  directories, AND WHEN purge is requested, THE SYSTEM SHALL explicitly name
  both directories as user-authored data that will be deleted in the
  confirmation path, emit the warning before any removal, include it in text
  and JSON reports, and preserve the existing purge confirmation phrase.
- AC-06: THE SYSTEM SHALL explain in init output, the seeded README, and public
  English/Japanese documentation that instruction Markdown is user-owned, used
  only by explicit file path, shared from `instructions/` by normal Git policy,
  and local-only from `instructions.local/` by default.
- AC-07: WHERE this repository dogfoods the feature, THE SYSTEM SHALL place the
  maintainer release procedure at `.mochiflow/instructions.local/release.md`
  and restore `.mochiflow/constitution.local.md` to the default local
  constitution stub without committing either ignored file.

## QA Scenarios

| QA | Dimension | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | QA-FUNC | cli | Automated | Run init in an empty fixture and inspect both directories, the seeded README, and init output. | Both directories exist, README states the explicit-path/non-managed contract, and output advertises the paths. |
| QA-02 | QA-ABUSE, QA-DATA | cli | Automated | Edit the seeded README and add arbitrary Markdown, then run init again and with `--force`. | User files remain byte-for-byte unchanged while managed init artifacts may refresh. |
| QA-03 | QA-ABUSE, QA-COMPAT | cli | Automated | Pre-create a custom `.mochiflow/.gitignore`, run init, and inspect ignore behavior with Git. | Custom lines remain, `instructions.local/` is ignored exactly once, and `instructions/` remains a normal tracked candidate. |
| QA-04 | QA-RESIL, QA-DATA | cli | Automated | Place a regular file at one of the directory paths and run init. | Init fails with a clear filesystem error and does not delete or replace the conflicting file. |
| QA-05 | QA-DATA, QA-UX | cli | Automated | Put distinct Markdown in both directories, run normal detach, then exercise text and JSON purge without and with the exact confirmation. | Normal detach retains both files; purge names both paths before removal and in its report without mixing JSON stdout; unconfirmed purge retains them; confirmed purge removes the tree. |
| QA-06 | QA-COMPAT, QA-REG | cli | Automated | In an initialized fixture, remove both directories and run join and bundled upgrade; separately run repository doctor, freeze, adapter-generation, and engine-drift checks. | Fixture join and upgrade do not recreate the directories; repository diagnostics and generated contracts remain independent of them. |
| QA-07 | QA-UX, QA-COMPAT | cli | Automated / AI-observed | Inspect the English/Japanese public docs, seeded README, generated adapters, and standing router/constitution artifacts. | Documentation explains explicit path use and Git ownership; standing/generated contracts contain no instructions-directory references. |
| QA-08 | QA-REG | cli | Automated | Run the configured CLI verification profile and spec consistency check after implementation. | Tests, formatting, clippy, freeze check, and spec consistency all pass. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- The elevated-risk reviewer result is recorded in `design.md ## Review Results` before acceptance.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | Init fixture assertions; QA-01, QA-04 | `cli/crates/mochiflow-core/src/init.rs`; `cli/crates/mochiflow-cli/tests/cli.rs`; `cli/crates/mochiflow-cli/tests/first_run.rs` | UNVERIFIED | | Includes human-readable and JSON/dry-run output expectations plus path-collision safety. |
| AC-02 | cli | automated | Repeated and forced init preservation test; QA-02 | `cli/crates/mochiflow-core/src/init.rs`; `cli/crates/mochiflow-cli/tests/cli.rs` | UNVERIFIED | | Locks the README as user-owned after creation. |
| AC-03 | cli | automated | Custom ignore and Git status fixture; QA-03 | `cli/crates/mochiflow-core/src/init.rs`; `.mochiflow/.gitignore`; `cli/crates/mochiflow-cli/tests/cli.rs` | UNVERIFIED | | Existing custom content is preserved without force. |
| AC-04 | cli | automated | Join/upgrade/diagnostic regression and standing-contract content checks; QA-06, QA-07, QA-08 | `cli/crates/mochiflow-cli/tests/cli.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | No config, engine, or adapter implementation file is added to the feature. |
| AC-05 | cli | automated | Detach/purge fixture assertions; QA-05 | `cli/crates/mochiflow-core/src/detach.rs`; `cli/crates/mochiflow-cli/tests/cli.rs` | UNVERIFIED | | Covers text and structured reporting without changing the confirmation phrase. |
| AC-06 | cli | automated / AI-observed | Output/golden assertions and documentation read-through; QA-01, QA-07 | `cli/crates/mochiflow-core/src/init.rs`; `cli/crates/mochiflow-cli/tests/golden/init_npm.json`; `README.md`; `README.ja.md`; `docs/getting-started.md`; `docs/configuration.md` | UNVERIFIED | | Seeded README content is generated from the CLI implementation. |
| AC-07 | cli | AI-observed | Inspect ignored local files after migration; QA-07 | `.mochiflow/constitution.local.md`; `.mochiflow/instructions.local/release.md` | UNVERIFIED | | Local working-copy evidence only; neither path is staged. |
