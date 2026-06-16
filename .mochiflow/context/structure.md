# Structure (foundational context)

Coarse orientation map, regenerated from code via `refresh-context`. Code is the
source of truth; this is a forward-placed map to avoid first-move mistakes.

## Source vs generated vs vendored

- **Engine source (SoT)**: repo-root `engine/` — edit here. `engine/MANIFEST.json`
  is a generated hash map; regenerate after editing any engine file.
- **Vendored engine copy**: `.mochiflow/engine/` — gitignored install snapshot,
  **not** the source. Synced from repo-root `engine/` via `mochiflow upgrade`.
  Never edit; the dogfood `mochiflow` run reads it and may lag the source.
- **Generated adapters**: `engine/adapters/<tool>/*.tpl` render into repo-root
  tool entrypoints (`AGENTS.md`, `.kiro/`, `CLAUDE.md`, `.github/`). Regenerate
  with `mochiflow adapter generate`; never hand-edit the outputs.

## Code layout

- `cli/crates/mochiflow-cli` — binary (clap CLI, `main.rs`), conformance tests in
  `tests/conformance.rs` (+ fixtures at repo-root `tests/conformance/`).
- `cli/crates/mochiflow-core` — library: `config` · `init` · `doctor` · `adapter`
  · `lint` · `index` · `pr` · `upgrade` · `manifest` · `spec_meta`.
- `contracts/` — frozen JSON schemas + `contracts.lock` (the version-gate hash
  covers `contracts/*.json` + `golden/**` + MANIFEST `files`).
- `.mochiflow/constitution.md` — user-authored always-loaded project rules.
- `.mochiflow/constitution.local.md` — gitignored user/machine-local always-loaded rules.
- `.mochiflow/context/` — code/config-derived current-state maps (refresh:
  `product`, `structure`, `tech`).
- `.mochiflow/adr/` — durable decision and pitfall records (fold:
  `decisions`, `pitfalls`).
- `.mochiflow/specs/` — specs (`_backlog/`, `{slug}/`, `_done/`).

## Entry points

- `mochiflow <command>` — `init` · `onboard` · `config` · `lint` · `doctor` ·
  `adapter` · `index` · `ready` · `backlog` · `upgrade` · `pr`.
- Verification surface: `cli` → `cargo test --manifest-path cli/Cargo.toml`.
