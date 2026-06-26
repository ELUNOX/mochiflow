# Configuration

`mochiflow init` writes project configuration under `.mochiflow/` and generates
adapter entrypoints for the AI tools you use.

## Adapters

Adapters tell each AI tool how to load the MochiFlow workflow:

| Adapter | Generated entrypoint |
| --- | --- |
| `agents` | `AGENTS.md` |
| `claude-code` | `CLAUDE.md` |
| `kiro` | `.kiro/steering/mochiflow.md` (always-on) + read-only reviewer agent |
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

MochiFlow does not generate or manage tool permission settings. File, shell, and
network permissions remain the responsibility of the AI tool, user environment,
and operating system running the agent.

## Joining an existing project

`mochiflow init` creates the shared project configuration and vendored engine.
In a team repository where `.mochiflow/config.toml` and `.mochiflow/engine/` are
already tracked, clone or pull is enough for the AI-tool entrypoints to resolve.
If local runtime state, adapters, or `INDEX.md` need repair, use:

```bash
mochiflow join
```

`join` repairs local generated state (`.mochiflow/state/`), can restore a missing
`.mochiflow/engine/` for older or broken worktrees, and refreshes shared adapter
files and `INDEX.md` when needed. Existing Markdown instructions keep their
custom content; MochiFlow only updates its managed block.

Remove adapter integration with:

```bash
mochiflow detach
```

The default detach removes only generated adapter content and runtime state,
leaving the tracked engine, config, and project knowledge in place. To delete
everything under `.mochiflow/`, use `mochiflow detach --purge --confirm "delete
mochiflow data"`.

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

Track project files:

- `.mochiflow/config.toml`
- `.mochiflow/engine/`
- `.mochiflow/context/`
- `.mochiflow/specs/`
- `.mochiflow/adr/`
- `.mochiflow/INDEX.md`

Ignore regenerated or runtime-derived state:

```gitignore
.mochiflow/state/
.mochiflow/constitution.local.md
```

The vendored engine copy is created by `init` and updated by `upgrade`; because
it is tracked, engine upgrades are reviewed and committed like other project
changes. Runtime state is derived from commands and should not be committed.
