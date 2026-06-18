# Changelog

All notable changes to MochiFlow are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
