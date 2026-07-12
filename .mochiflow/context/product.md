# Product (foundational context)

## Purpose

MochiFlow is a spec-driven workflow engine for AI coding agents. It guides work
through `discuss → plan → build → open → update → close`, retains auditable
specification and decision records in the repository, and coordinates
verification and PR handoff across supported tools.

## Users / Actors

Developers using AI coding tools (Kiro, Claude Code, Copilot, and generic
`AGENTS.md` adapters) who want a disciplined spec-to-PR workflow.

## Domain Terms

| Term | Meaning | Source |
| --- | --- | --- |
| verb | A lifecycle phase: `discuss`, `plan`, `build`, `open`, `update`, or `close`. | `engine/reference/lifecycle.md` |
| non-phase command | A workflow utility such as `review`, `refresh-context`, or `onboard`. | `engine/router.md` |
| router | The sole standing routing contract; it selects a workflow before command procedures are read. | `engine/router.md` |
| foundational context | Code/config-derived orientation loaded only when a selected workflow or repository task needs it. | `engine/router.md` |
| AC Matrix | The table in `spec.md` that traces acceptance criteria to implementation and verification evidence. | `engine/reference/verification.md` |
| ADR | Per-file decision and pitfall records folded during `open`; current state remains code-derived. | `engine/reference/knowledge.md` |
| Agent Context API | Versioned, read-only JSON projection of repository/spec facts and lifecycle eligibility for coding agents. | `contracts/agent-context.schema.json` |

## Core Invariants

- Constitution files and `engine/router.md` are standing MochiFlow inputs; project
  configuration and foundational context are loaded on demand.
- Code is the source of truth for current state; context is regenerated and ADRs
  record durable rationale rather than implementation state.
- Exactly two human delivery approval gates exist: approve-to-build and
  approve-PR. `accepted` is set by open; merged/done is derived from VCS/provider state.
- Engine source, adapter templates, schemas, and frozen contract artifacts are
  verified through the Rust CLI and dogfood synchronization.
- Natural-language routing remains engine-owned; the CLI exposes only
  deterministic lifecycle eligibility and observation quality.

## Non-goals

- MochiFlow is not a CI system, runtime framework, or PR hosting service.
- It does not use token/word budgets or a prompt compiler to assemble context.

## Evidence

- `engine/router.md`: routing and standing-load contract
- `engine/reference/lifecycle.md`: lifecycle and approval gates
- `cli/Cargo.toml`: CLI workspace metadata

## Last refreshed

- Date: 2026-07-12
- Source commit: ebb0097
