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
- `cli/crates/mochiflow-core` — library modules: `adapter`, `adr`, `backlog`,
  `config`, `delivery`, `detach`, `detect`, `doctor`, `freeze`, `index`, `init`,
  `join`, `lint`, `manifest`, `pr`, `present`, `spec_meta`, `status`, `upgrade`.
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
- `.mochiflow/adr/` — durable decision and pitfall record stores (directory-
  rooted: `decisions/`, `pitfalls/`; each record is one `{date}-{slug}.md` with
  front-matter; a generated gitignored `INDEX.md` per store).
- `.mochiflow/specs/` — specs (`_backlog/`, `{slug}/`, `_done/`).

## Adapter output layout

| Adapter | Generated output | Mechanism |
| --- | --- | --- |
| `agents` | `AGENTS.md` | Embeddable managed block appended to existing file |
| `claude-code` | `CLAUDE.md` | Embeddable managed block |
| `copilot` | `.github/copilot-instructions.md` | Embeddable managed block |
| `kiro` | `.kiro/steering/mochiflow.md` | Full-file managed (always-on steering with `#[[file:]]` pointers) |
| `kiro` | `.kiro/agents/spec-plan-auditor.json` | Full-file managed (read-only plan-quality reviewer agent) |
| `kiro` | `.kiro/agents/spec-change-reviewer.json` | Full-file managed (read-only post-implementation reviewer agent) |

Kiro uses no dedicated build agent, no `toolsSettings`, and no per-verb steering.

## Entry points

- `mochiflow <command>` — `accept`, `adapter`, `adr`, `backlog`, `completions`,
  `config`, `detach`, `doctor`, `freeze`, `guide`, `index`, `init`, `join`,
  `lint`, `pr`, `ready`, `status`, `upgrade`.
- Verification surface: `cli` → `cargo test --manifest-path cli/Cargo.toml`.
