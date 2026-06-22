# Structure (foundational context)

Coarse orientation map, regenerated from code via `refresh-context`. Code is the
source of truth; this is a forward-placed map to avoid first-move mistakes.

## Source vs generated vs vendored

- **Engine source (SoT)**: repo-root `engine/` — edit here. It is embedded into
  the CLI binary and is also the source for dogfood updates. `engine/MANIFEST.json`
  is generated with `mochiflow freeze`.
- **Vendored engine copy**: `.mochiflow/engine/` — installed project-local copy
  used by generated adapters and dogfood runs, **not** the source. For this
  repo, sync it from repo-root `engine/` with `mochiflow upgrade --source engine`
  after source-engine edits.
- **Generated adapters**: `engine/adapters/<tool>/*.tpl` render into repo-root
  tool entrypoints (`AGENTS.md`, `.kiro/`, `CLAUDE.md`, `.github/`). Regenerate
  with `mochiflow adapter generate`; never hand-edit the outputs.

## Code layout

- `cli/crates/mochiflow-cli` — clap binary (`main.rs`) and CLI integration tests
  (`tests/cli.rs`, `tests/conformance.rs`, `tests/first_run.rs`, `tests/pr.rs`).
- `cli/crates/mochiflow-core` — library modules: `adapter`, `backlog`,
  `config`, `detach`, `detect`, `doctor`, `index`, `init`, `join`, `lint`,
  `manifest`, `pr`, `present`, `spec_meta`, `upgrade`.
- `docs/` — user-facing concepts, setup, configuration, versioning, and release
  verification.
- `assets/` — logo / mark images used by README and distribution material.
- `contracts/` — frozen JSON schemas + `contracts.lock` (the version-gate hash
  covers `contracts/*.json`, conformance golden fixtures, and engine manifest
  files).
- `tests/conformance/` — schema fixtures and golden files used by CLI
  conformance tests.
- `.mochiflow/constitution.md` — user-authored always-loaded project rules.
- `.mochiflow/constitution.local.md` — gitignored user/machine-local always-loaded rules.
- `.mochiflow/context/` — code/config-derived current-state maps (refresh:
  `product`, `structure`, `tech`).
- `.mochiflow/adr/` — durable decision and pitfall records (fold:
  `decisions`, `pitfalls`).
- `.mochiflow/specs/` — specs (`_backlog/`, `{slug}/`, `_done/`).

## Entry points

- `mochiflow <command>` — `config`, `index`, `lint`, `doctor`, `adapter`,
  `upgrade`, `ready`, `backlog`, `init`, `join`, `detach`, `guide`,
  `completions`, `freeze`, `pr`.
- Verification surface: `cli` → `cargo test --manifest-path cli/Cargo.toml`.
