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
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ELUNOX/mochiflow/releases/download/v1.1.0/mochiflow-cli-installer.sh | sh
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

To pin the language explicitly:

```bash
mochiflow init --language ja
mochiflow init --language en
```

## Understand the result

`init` reports one of three outcomes:

- `Ready` — config, context, and generated adapter files are usable.
- `Needs AI review` — setup is valid, but project-specific TODOs need an AI
  agent and your judgment before the workflow is ready.
- `Blocked` — an existing hand-written adapter file needs a manual candidate
  merge; `init` exits `1` so scripts can detect it.

When `Needs AI review` appears, paste the prompt printed by `init` into your AI
coding agent. The agent should resolve `# mochiflow: confirm` and TODO items,
fill `.mochiflow/context/product.md`, `.mochiflow/context/structure.md`, and
`.mochiflow/context/tech.md` from the codebase, regenerate adapters, and finish
with:

```bash
mochiflow doctor
```

When `doctor` passes, the project has the context and workflow instructions your
AI tool needs.
