# Configuration

`mochiflow init` writes project configuration under `.mochiflow/` and generates
adapter entrypoints for the AI tools you use.

## Adapters

Adapters tell each AI tool how to load the MochiFlow workflow:

| Adapter | Generated entrypoint |
| --- | --- |
| `agents` | `AGENTS.md` |
| `claude-code` | `CLAUDE.md` |
| `kiro` | `.kiro/` agents and steering |
| `copilot` | `.github/` integration |

Choose adapters during setup:

```bash
mochiflow init --adapter agents,claude-code
```

Regenerate adapter files after config or engine updates:

```bash
mochiflow adapter generate
```

Existing Markdown instruction files are preserved; MochiFlow updates only its
managed block inside them. Structured adapter files that cannot be safely
embedded may require a candidate merge or explicit `--force` replacement.

The `codex` alias resolves to the neutral `agents` adapter.

## Language

Project language controls generated user-facing artifacts and defaults from your
locale during `init`. Pin it explicitly when needed:

```bash
mochiflow init --language ja
mochiflow init --language en
```

## Surfaces and verification

A surface is a build or test target with its own verification command. The AI
agent uses surfaces to know what to run after implementation. For this
repository, the primary surface is the Rust CLI:

```bash
cargo test --manifest-path cli/Cargo.toml
```

Use `mochiflow config show` to inspect the active project configuration.

## What to track

Track authored project knowledge:

- `.mochiflow/config.toml`
- `.mochiflow/context/`
- `.mochiflow/specs/`
- `.mochiflow/adr/`
- `.mochiflow/INDEX.md`

Ignore regenerated or runtime-derived state:

```gitignore
.mochiflow/engine/
.mochiflow/state/
```

The vendored engine copy is restored by `init` or `upgrade`; runtime state is
derived from commands and should not be committed.
