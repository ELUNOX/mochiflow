<p align="center">
  <img src="assets/mochiflow-logo.png" alt="MochiFlow" width="360">
</p>

<p align="center">
  A development workflow tool that helps AI coding agents move from discussion
  and design to implementation, review, and PR.
</p>

<p align="center">
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License: MIT OR Apache-2.0"></a>
  <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/version-1.2.2-informational.svg" alt="Version 1.2.2"></a>
  <a href="cli/Cargo.toml"><img src="https://img.shields.io/badge/rust-2024%20edition-orange.svg" alt="Rust 2024 edition"></a>
</p>

<p align="center">
  <b>English</b> | <a href="README.ja.md">日本語</a>
</p>

---

# MochiFlow

MochiFlow is a development workflow tool for working with AI coding agents.

It helps you take a change from discussion to PR without relying only on chat
history. Your agent can clarify the scope, plan the design, implement the
change, review it, and keep useful project knowledge in the repository.

MochiFlow is not an AI model or AI runtime. It is a single Rust CLI that adds a
`.mochiflow/` workspace and AI-tool entry files to your project.

## What MochiFlow Helps With

- **Plan before coding**
  Turn rough requests into scoped work before implementation starts.

- **Keep project memory**
  Save important decisions and pitfalls in the repository, so future work does
  not repeat the same reasoning or mistakes.

- **Review with context**
  Ask your AI agent to check specs, design, implementation, tests, and PR
  readiness against the MochiFlow workflow.

- **Deliver through PRs**
  Keep implementation, verification, review feedback, and PR handoff connected.

## Quick Start

Install MochiFlow:

```bash
# Homebrew, recommended on macOS and Linux
brew install ELUNOX/tap/mochiflow

# Shell installer
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/ELUNOX/mochiflow/releases/download/v1.2.2/mochiflow-cli-installer.sh | sh

# From source
git clone https://github.com/ELUNOX/mochiflow.git
cd mochiflow
cargo install --path cli/crates/mochiflow-cli
```

Set it up in a new project:

```bash
cd /path/to/project
mochiflow init
mochiflow doctor
```

Join a repository where MochiFlow is already set up:

```bash
git clone <repository-url>
cd <repository>
mochiflow join
mochiflow doctor
```

## Terminal Commands

These are the main commands you run in your terminal:

```bash
mochiflow init      # Set up MochiFlow in a new project
mochiflow join      # Repair local state for an existing MochiFlow project
mochiflow doctor    # Check project health
mochiflow status    # Show the current delivery board
```

Most day-to-day development happens in your AI coding tool, not in the terminal.

## Working With Your AI Agent

You can start naturally:

```text
I want to add saved filters to the search page.
Before coding, help me clarify the scope, edge cases, and design.
```

Then continue in plain language:

```text
Turn this into a plan.
```

```text
Build this plan.
```

```text
Open a PR.
```

When you want to be explicit, send one of these messages to your AI tool:

| Message | Meaning |
| --- | --- |
| `mochiflow-discuss` | Clarify the idea, scope, and acceptance criteria |
| `mochiflow-plan` | Write the spec, design, and task plan |
| `mochiflow-build` | Implement, test, and verify the change |
| `mochiflow-review` | Review the spec, design, implementation, or PR readiness |
| `mochiflow-open` | Prepare and open the PR |
| `mochiflow-update` | Apply PR feedback and re-verify |
| `mochiflow-close` | Clean up locally after the PR is merged |

These are not terminal commands. They are messages for your AI coding tool.

## Decisions, Pitfalls, And Review

MochiFlow keeps useful project knowledge in files, not only in chat.

| Knowledge | What it means |
| --- | --- |
| Decisions / ADRs | Why a design or implementation choice was made |
| Pitfalls | Gotchas, failure patterns, or things future agents should avoid |

An ADR is a decision note. A pitfall is a note about something easy to get
wrong.

This is one of MochiFlow's core values: each change can leave behind context
that helps the next change go better.

You can also ask for a MochiFlow review:

```text
mochiflow-review
```

The review checks whether the spec is clear, the design and implementation
match, the acceptance criteria are verified, and the PR is ready to hand off.

## What MochiFlow Adds

`mochiflow init` adds a `.mochiflow/` workspace and generates entry files for
your AI coding tools.

```text
.mochiflow/
  config.toml        # Project settings
  engine/            # Vendored workflow engine
  constitution.md    # Always-loaded project rules
  context/           # Current project map
  specs/             # Specs created during the workflow
  adr/               # Decisions and pitfalls

AGENTS.md / CLAUDE.md / .kiro/ / .github/
  # Entry files for AI coding tools
```

## What A Spec Contains

A spec is the project file that records what the change is, what is out of
scope, and how the result will be checked.

| File | Role |
| --- | --- |
| `spec.md` | What to build, what not to build, and how to verify it |
| `design.md` | Technical approach, alternatives, interfaces, and failure handling |
| `tasks.md` | Ordered implementation checklist for the AI agent |
| AC Matrix | Traceability from acceptance criteria to implementation, verification, evidence, and result |

Small fixes can use a lightweight spec. Larger changes can add more design and
task detail as needed.

## More Terminal Commands

These are useful once you are familiar with the basic flow:

```bash
mochiflow guide                        # Print the AI-tool usage card
mochiflow lint [--spec SLUG]           # Check spec consistency
mochiflow config show                  # Inspect resolved project settings
mochiflow adapter generate [--check]   # Generate or check AI-tool entry files
mochiflow index                        # Refresh generated indexes
```

## Temporarily Remove The Integration

To remove generated adapter content and local state while keeping project
knowledge:

```bash
mochiflow detach
```

To delete all MochiFlow project data, use the explicit purge command:

```bash
mochiflow detach --purge --confirm "delete mochiflow data"
```

## Supported AI Tools

| Tool | Integration |
| --- | --- |
| Kiro | Steering files and reviewer agents |
| Claude Code | `CLAUDE.md` |
| GitHub Copilot | `.github/` instructions |
| Generic agents | `AGENTS.md` |

## Learn More

- [Getting started](docs/getting-started.md)
- [Concepts](docs/concepts.md)
- [Configuration](docs/configuration.md)
- [Versioning](docs/versioning.md)
- [Release verification](docs/release-verification.md)
- [Changelog](CHANGELOG.md)

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for local
development, tests, and PR expectations, and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
for the community code of conduct.

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting instructions.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE). You may
choose either license.
