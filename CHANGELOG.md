# Changelog

All notable changes to MochiFlow are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.3] - Unreleased

### Changed

- New projects track `.mochiflow/engine/` by default and ignore only local
  runtime state plus `constitution.local.md`; `join` is now positioned as a
  repair helper after clone/pull.
- `mochiflow detach` now preserves the tracked engine by default while removing
  adapter integration and local state.
- Ship now writes the reviewer-facing `qa-instructions.md` worksheet into
  ignored state, while durable QA evidence remains in the AC Verification Matrix
  that is reviewed and archived with the spec.
- Replaced the legacy top-level `language` config with `[i18n]`
  `artifact_language` and `conversation_language`.
- `mochiflow init` now writes `[i18n]` and uses `--artifact-language` /
  `--conversation-language`; the old `--language` flag has been removed.
- Engine language policy now treats engine docs/templates as English source and
  rendered durable artifacts as artifact-language output.
- Build now confirms eligibility through `mochiflow ready {slug}` (lint + approved
  status + runnable verification) instead of an ad-hoc manual `spec.yaml` read.
- Documented the provisional build-time AC Matrix tokens `UNVERIFIED` (automated
  AC not yet verified) and the `N/A: <reason>` ASCII equivalent of
  `対象外（<reason>）`, clarifying that only `PASS` / `人間確認済み` /
  `対象外（<reason>）` are done-eligible.
- Clarified that `status` transitions (`approved`, `done`) are direct `spec.yaml`
  edits validated by `lint`, not a CLI transition command ("mechanically" wording
  removed).
- Spec archival in the ship close-out commit now specifies `git mv` so the rename
  stages as a paired delete + add.

### Fixed

- Onboarding default config no longer ships an unreachable `pr_command` alongside
  `provider`; `provider = "none"` keeps manual PR handoff as the first-class
  default per the engine git policy.

### Deprecated

- Existing top-level `language` config remains readable as a legacy artifact
  language source and is reported by `doctor config` as deprecated.

## [1.1.2] - 2026-06-19

### Added

- `mochiflow join` for setting up local generated state and refreshing
  MochiFlow-managed entrypoints after cloning or pulling an already-initialized
  team project.
- `mochiflow index --check` and doctor warnings for stale `INDEX.md`.

### Changed

- Re-running `mochiflow init` in a repository with an existing config now follows
  the safe join-style local setup path unless `--force` is provided.

## [1.1.1] - 2026-06-18

### Added

- Shell completion script generation via `mochiflow completions`.
- Machine-readable `mochiflow doctor --json` output.
- The plan phase now presents a copy-paste session handoff prompt so you can
  continue implementation cleanly in a new session.

### Changed

- `mochiflow init` now installs the engine through the same staged path as
  `mochiflow upgrade`; an already-modified installed engine requires `--force`.
- `mochiflow init` detects the repository's default branch instead of assuming
  the current branch.

## [1.1.0] - 2026-06-15

### Changed

- Replaced the legacy memory living-spec layer with `[constitution]`, `[context]`,
  and `[adr]`.
- Added `context.tech` to the refreshable current-state context layer.
- Moved ship-time decision and pitfall folds to ADR files.

## [1.0.0] - 2026-06-13

Initial public release.

### Added

- Rust CLI with `init`, `config`, `lint`, `doctor`, `adapter`, `index`, `ready`,
  `backlog`, `upgrade`, and `pr` commands.
- Embedded project-agnostic engine for the `discuss -> plan -> build -> ship`
  workflow.
- Adapter generation for Kiro, Claude Code, GitHub Copilot, and generic
  `AGENTS.md`-based agents.
- `mochiflow init` as the standard first-run path, with `Ready`,
  `Needs AI review`, and `Blocked` outcomes plus `--yes` for non-interactive
  setup.
- `mochiflow init` prompts for project language on interactive terminals and
  uses locale-based language defaults for non-interactive setup.
- `mochiflow upgrade` installs the engine bundled with the current CLI and
  regenerates adapters; `--source` remains available for development workflows.
- Contract schemas and conformance tests for config, spec metadata, manifests,
  PR requests, golden output, and engine drift checks.
