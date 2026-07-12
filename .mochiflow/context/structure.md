# Structure (foundational context)

## Source vs Generated vs Vendored

- **Engine source (SoT):** `engine/`. Edit it here; `engine/MANIFEST.json` is
  regenerated with `mochiflow freeze`.
- **Vendored engine:** `.mochiflow/engine/`. It is regenerated from `engine/`
  with `mochiflow upgrade --source engine`; never edit it directly.
- **Generated adapters:** `engine/adapters/<tool>/*.tpl` render to repository
  tool entrypoints such as `AGENTS.md` and `.kiro/`; regenerate them with
  `mochiflow adapter generate`.

## Code Layout

- `cli/crates/mochiflow-cli` — Clap binary and CLI/conformance tests.
- `cli/crates/mochiflow-core` — workflow, adapter, ADR, verification, delivery,
  and engine-installation library modules.
- `cli/crates/mochiflow-core/src/inspect.rs` — shared read-only repository/spec
  snapshot, observation quality, and lifecycle eligibility.
- `engine/router.md` — the single route table and standing route contract.
- `engine/commands/` — selected workflow procedures with required and
  conditional load declarations.
- `engine/reference/` — responsibility-sized lifecycle, verification, risk,
  review, delivery, knowledge, language, and presentation policies.
- `engine/agents/` — plan-auditor and change-reviewer profiles sharing
  `reviewer-core.md`.
- `.mochiflow/specs/` — active specs and `_backlog/` seeds; specs remain flat
  through delivery and are never moved to `_done/`.
- `.mochiflow/adr/` — per-file decision and pitfall stores with generated,
  gitignored indexes.

## Entry Points

- `mochiflow <command>` — CLI commands including `accept`, `adapter`, `adr`,
  `freeze`, `inspect`, `lint`, `pr`, `ready`, `status`, and `upgrade`.
- `AGENTS.md` and `.kiro/steering/mochiflow.md` — generated adapter entrypoints
  that keep constitution plus router standing and defer the rest of the graph.

## Data / Artifact Locations

- `contracts/` — frozen schemas and `contracts.lock`.
- `contracts/agent-context.schema.json` — public Agent Context API contract.
- `engine/MANIFEST.json` — frozen source-engine manifest.
- `.mochiflow/context/` — regenerated current-state orientation.
- `.mochiflow/state/` — gitignored local review and delivery scratch.

## Evidence

- `engine/router.md`: router and load contract
- `engine/agents/reviewer-core.md`: shared reviewer contract
- `.mochiflow/config.toml`: configured paths and adapter targets

## Last refreshed

- Date: 2026-07-12
- Source commit: ebb0097
