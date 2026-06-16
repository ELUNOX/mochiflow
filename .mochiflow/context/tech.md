# Tech (foundational context)

Regenerated from code/config via `refresh-context`. Minimal, slow-changing slice.

## Stack

- Rust CLI workspace under `cli/` with `mochiflow-cli` and `mochiflow-core`.
- JSON schemas under `contracts/` define frozen project contracts.
- Engine source under `engine/` is embedded into the CLI and vendored into
  `.mochiflow/engine/` for dogfood runs.

## Verification

- Primary surface: `cli`.
- Default verification: `cargo test --manifest-path cli/Cargo.toml`.

## Generated / Frozen Artifacts

- `engine/MANIFEST.json` is generated from repo-root `engine/`.
- `contracts/contracts.lock` freezes `contracts/*.json` and conformance golden
  fixtures.
- Adapter templates in `engine/adapters/**` generate tool entrypoints such as
  `AGENTS.md`, `.kiro/**`, `CLAUDE.md`, and Copilot instructions.

