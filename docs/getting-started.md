# Getting started

This guide expands the short README path: install MochiFlow, run `init`, finish
onboarding when needed, and confirm the project with `doctor`.

## Install

Homebrew is the recommended install path:

```bash
brew install ELUNOX/tap/mochiflow
```

For Linux or macOS without Homebrew, copy the shell installer command from the
[latest release](https://github.com/ELUNOX/mochiflow/releases). To pin a
specific version, use the versioned release URL:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ELUNOX/mochiflow/releases/download/v1.2.0/mochiflow-cli-installer.sh | sh
```

From source:

```bash
git clone https://github.com/ELUNOX/mochiflow.git
cd mochiflow
cargo install --path cli/crates/mochiflow-cli
```

## Initialize a project

Run `init` from the target project root:

```bash
mochiflow init
```

On an interactive terminal, `init` asks which adapters to generate and which
project language to use. The adapter defaults to `agents`; language defaults
from your locale (`ja` for a Japanese locale, otherwise `en`).

For CI or scripts:

```bash
mochiflow init --yes
```

To pin artifact and conversation language responsibilities explicitly:

```bash
mochiflow init --artifact-language ja --conversation-language auto
mochiflow init --artifact-language en --conversation-language en
```

## Understand the result

`init` reports one of three outcomes:

- `Ready` — config, context, and generated adapter files are usable.
- `Needs AI review` — setup is valid; paste the printed setup prompt into your
  AI agent to confirm uncertain detected values and fill project context.
- `Blocked` — an existing structured adapter file needs a manual candidate
  merge; existing Markdown instruction files are extended with a managed
  MochiFlow block instead of being replaced.

When `Needs AI review` appears, paste the prompt printed by `init` into your AI
coding agent. `# mochiflow: confirm` markers and TODOs are setup questions for
uncertain detected values, not errors. The agent should resolve them with you,
fill `.mochiflow/context/product.md`, `.mochiflow/context/structure.md`, and
`.mochiflow/context/tech.md` from the codebase, regenerate adapters, and finish
with:

```bash
mochiflow doctor
```

When `doctor` passes, the project has the context and workflow instructions your
AI tool needs.

`init` also creates two optional Markdown directories:
`.mochiflow/instructions/` for shareable notes and
`.mochiflow/instructions.local/` for local-only notes. MochiFlow does not
automatically load, parse, index, validate, or drift-check files in either
directory. When you want an AI agent to use one of those files, cite its path
explicitly in your request.

## Join an initialized team project

If `.mochiflow/config.toml` and `.mochiflow/engine/` are already tracked in the
repository, the project has already been initialized. Clone or pull is enough
for the AI-tool entrypoints to resolve. If local runtime state, adapters, or
`INDEX.md` need repair, run:

```bash
mochiflow join
```

`join` repairs local generated state such as `.mochiflow/state/`, can restore a
missing `.mochiflow/engine/` for older or broken worktrees, and refreshes the
AI-tool entrypoints and `INDEX.md` when needed.

## Detach later

Use `mochiflow detach` when you want to remove MochiFlow from the active AI
tools without deleting project knowledge or user instructions. The command
removes generated adapter content plus `.mochiflow/state/`, but keeps the
tracked engine, `.mochiflow/config.toml`, specs, ADR, context, constitution
files, `.mochiflow/instructions/`, and `.mochiflow/instructions.local/`.
Running `mochiflow join` repairs local state and adapters from the preserved
config.

Use purge mode only for a full deletion:

```bash
mochiflow detach --purge --confirm "delete mochiflow data"
```

Purge removes all MochiFlow project data, including specs, ADR, context,
constitution, config, and both instruction directories.
