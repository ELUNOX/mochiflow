# Tech (foundational context)

## Stack

- Rust 2024 workspace under `cli/`, with `mochiflow-cli` and `mochiflow-core`.
- Workspace package version 1.2.3 and MSRV 1.96.
- CLI dependencies include Clap, Serde, TOML, Regex, SHA-2, ThisError, Anyhow,
  and `include_dir` for embedding the engine.
- Engine Markdown and adapter templates are source artifacts; the vendored engine
  and configured adapters are regenerated outputs.

## Verification

| Surface | Purpose | Command | Source |
| --- | --- | --- | --- |
| cli | MochiFlow Rust CLI | `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check` | `.mochiflow/config.toml` |

Additional project integrity checks include `mochiflow lint`, `mochiflow adr lint`,
`mochiflow adapter generate --check`, and source-engine dogfood synchronization.

## Generated / Frozen Artifacts

- `engine/MANIFEST.json` is generated from repo-root `engine/`.
- `contracts/contracts.lock` freezes contract schemas, conformance fixtures, and
  engine manifest files.
- `engine/adapters/**` generates `AGENTS.md`, Kiro steering, and the Kiro
  plan-auditor/change-reviewer resources.
- `.mochiflow/engine/` is refreshed from source with `mochiflow upgrade --source engine`.

## External Services / Tooling

- GitHub is the configured PR provider; `mochiflow pr` performs preflight,
  pushes the feature branch, and creates the provider handoff.

## Evidence

- `cli/Cargo.toml`: workspace edition, MSRV, version, and dependencies
- `.mochiflow/config.toml`: configured verification and provider
- `engine/MANIFEST.json`: engine artifact manifest

## Last refreshed

- Date: 2026-07-10
- Source commit: fe9a752
