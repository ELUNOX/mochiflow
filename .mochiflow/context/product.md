# Product (foundational context)

Regenerated from code/intent via `refresh-context`. Minimal, slow-changing slice.

## Purpose

MochiFlow is a spec-driven workflow engine for AI coding agents. It drives work
through `discuss → plan → build → ship`, keeps state in repository files, and
keeps durable knowledge, verification, and PR handoff mechanics consistent
across supported tools.

## Users

Developers using AI coding tools (Kiro, Claude Code, Copilot, generic `AGENTS.md`)
who want a disciplined, auditable spec-to-PR flow.

## Domain terms

- **verb**: a lifecycle phase (`discuss`, `plan`, `build`, `open`, `update`,
  `close`).
- **non-phase command**: workflow utilities outside the lifecycle, including
  `patch`, `review`, and `refresh-context`.
- **patch**: a no-spec lane for concrete, local, reversible fixes.
- **surface**: a build target with its own verify command (`[surfaces.*]`).
- **risk**: ordered enum `standard < elevated < critical` deciding reviewer
  cadence / integration log / commit granularity.
- **AC Matrix**: the traceability table in `spec.md` connecting acceptance
  criteria to implementation, verification, evidence, and result.
- **constitution**: user-authored always-loaded project / local rules.
- **context**: code/config-derived current-state orientation refreshed by onboard / `refresh-context`.
- **ADR**: durable decision and pitfall records under directory-rooted stores
  (`[adr].decisions` / `[adr].pitfalls`), each one immutable per-file record
  with front-matter + supersession lifecycle; folded at open.
- **fold**: appending a per-file ADR record (dated *why* / active pitfall) to
  `[adr]` at open, with supersession via `supersedes`/`superseded_by`.
- **refresh**: regenerating `[context]` from code/config (onboard / refresh-context).
- **adapter**: per-tool entrypoint generated from engine templates.
- **vendored engine**: project-local `.mochiflow/engine` copy used by generated
  adapters and dogfood runs.
- **version-gate / contracts.lock**: hash freezing the contract surface.

## Core invariants

- Code is the source of truth for current state; prose never overrides code.
- Engine docs (`commands/`, `reference/`, `templates/`, `agents/`) are English and
  project-agnostic; project specifics live in `config.toml`.
- The contract surface is frozen by `contracts.lock`; changing a schema requires
  regenerating the lock and bumping `engine/VERSION` in the same commit.
- Exactly two delivery approval gates: approve-to-build and approve-PR.
  `open` sets `accepted`; `done` is derived from merge, never written.
- PR handoff is produced through `mochiflow pr`.
- Engine source, adapter templates, schemas, and golden fixtures are the frozen
  contract surface guarded by manifests / locks and tests.

## Non-goals

- Not a CI/build system; it invokes the project's own verify commands.
- Does not host or merge PRs; it delegates to `gh` / a pr_driver / manual handoff.
- Not a runtime framework; it is a workflow + living-spec engine.
