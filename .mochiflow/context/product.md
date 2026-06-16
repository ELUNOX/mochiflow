# Product (foundational context)

Regenerated from code/intent via `refresh-context`. Minimal, slow-changing slice.

## Purpose

MochiFlow is a spec-driven development engine that drives AI coding agents through
four verbs — `discuss → plan → build → ship` — keeping durable knowledge and
delivery mechanics consistent across tools.

## Users

Developers using AI coding tools (Kiro, Claude Code, Copilot, generic `AGENTS.md`)
who want a disciplined, auditable spec-to-PR flow.

## Domain terms

- **verb**: a lifecycle phase (discuss / plan / build / ship); `review` and
  `refresh-context` are non-phase utilities.
- **surface**: a build target with its own verify command (`[surfaces.*]`).
- **risk**: ordered enum `standard < elevated < critical` deciding reviewer
  cadence / integration log / commit granularity.
- **constitution**: user-authored always-loaded project / local rules.
- **context**: code/config-derived current-state orientation refreshed by onboard / `refresh-context`.
- **ADR**: durable decision and pitfall records folded at ship.
- **fold**: appending dated *why* / active pitfalls to `[adr]` at ship.
- **refresh**: regenerating `[context]` from code/config (onboard / refresh-context).
- **adapter**: per-tool entrypoint generated from engine templates.
- **version-gate / contracts.lock**: hash freezing the contract surface.

## Core invariants

- Code is the source of truth for current state; prose never overrides code.
- Engine docs (`commands/`, `reference/`, `templates/`, `agents/`) are English and
  project-agnostic; project specifics live in `config.toml`.
- The contract surface is frozen by `contracts.lock`; changing a schema requires
  regenerating the lock and bumping `engine/VERSION` in the same commit.
- Exactly two human gates (approve-to-build, approve-PR). Only `ship` sets `done`.
- `git push` / PR creation go only through `mochiflow pr`.

## Non-goals

- Not a CI/build system; it invokes the project's own verify commands.
- Does not host or merge PRs; it delegates to `gh` / a pr_driver / manual handoff.
- Not a runtime framework; it is a workflow + living-spec engine.
