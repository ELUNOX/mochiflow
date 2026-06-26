---
name: mochiflow-onboard
description: |
  MochiFlow's onboard command. Bootstraps MochiFlow into the current project
  by reading the project, turning its workflow into config.toml, generating
  adapters, and verifying with doctor. Activate when the user requests setup /
  onboarding / installation of MochiFlow.
triggers:
  - オンボーディングして
  - MochiFlow 入れて
  - mochiflow セットアップ
  - mochiflow setup
  - setup mochiflow
trigger_patterns: []
references:
  - reference/workflow.md
  - templates/context/product.md
  - templates/context/structure.md
  - templates/context/tech.md
---

# Onboard

Bootstrap MochiFlow into a project through guided automation.

## Purpose

Install the minimal MochiFlow skeleton if needed, then resolve `config.toml`
into an explicit operating contract. `mochiflow init` already machine-detects
the obvious facts (surfaces / verify commands / branch) and writes
them, flagging anything that needs a human decision with an inline
`# mochiflow: confirm` comment. Onboard's job is to read those markers and the
project, then confirm or correct each one with the user. The markers in
`config.toml` *are* the CLI→AI handoff — there is no separate envelope or prompt
file to paste. File-structure heuristics are input, not authority: make
defensible choices for surfaces, verification, and git.

## Procedure

1. **Confirm the adapter(s)**: which AI tool(s) the project uses is user/
   environment information, not something to infer from file structure. Ask the
   user which adapter(s) to generate (multiple allowed). Recognized: `agents`
   (AGENTS.md — generic industry default), `codex` (AGENTS.md, OpenAI Codex),
   `kiro`, `claude-code`, `copilot`. If the repo already shows traces
   (`.kiro/`, `CLAUDE.md`, `.github/copilot-instructions.md`, `AGENTS.md`),
   propose them as the default; otherwise recommend `agents`. Pass the chosen
   value(s) to `--adapter` (repeatable or comma-separated).

2. **Ensure the skeleton exists**: if `.mochiflow/config.toml` is missing, run
   `mochiflow init --target <project> --adapter <tool>[ --adapter <tool2>] --artifact-language <tag>[ --conversation-language auto]`
   first. `init` machine-detects surfaces / verify / branch, writes
   them, and attaches `# mochiflow: confirm` markers to values that need a human
   decision. If the Rust CLI is unavailable, report that `mochiflow` must be
   installed before onboarding. If config already exists, do not overwrite it;
   continue with the read/confirm steps.

3. **Read the confirm markers from the raw config text**: read
   `.mochiflow/config.toml` as **raw text** (not via a parsed/loaded config —
   TOML comments, including `# mochiflow: confirm`, are dropped when the file is
   deserialized). Collect every line carrying a `# mochiflow: confirm` marker;
   these are exactly the values `init` could not settle on its own and that you
   must confirm with the user.

4. **Read the project structure**: list top-level dirs, find `package.json` /
   `pyproject.toml` / `Project.swift` / `Makefile` / `Cargo.toml` etc.
   Identify git remote, default branch, package manager locks, workspace files,
   and CI config if present — enough to make a defensible call on each marked
   value.

5. **Confirm / refine surfaces and verify commands**: for each build target or
   responsibility boundary (subdir with its own manifest, workspace package,
   app, service, CLI, docs), confirm the detected surface name and verify
   command, resolving any `# mochiflow: confirm` marker with the user.
   - `package.json` → look at `scripts` for `build` / `test` / `check` / `lint`.
   - `pyproject.toml` → look for ruff / pytest / mypy scripts.
   - `Cargo.toml` → look for workspace/package shape and cargo test/check/clippy.
   - `Project.swift` / `*.xcodeproj` → iOS surface.
   - Multiple manifests in subdirs → monorepo; one surface per logical group,
     not necessarily one surface per directory.

6. **Resolve git config — never auto-adopt a provider**: read the current branch
   for `base_branch`. `git remote -v` may reveal a hosting provider, but keep
   `provider = "none"` (manual PR is the first-class default in
   `reference/git.md`). Present any detected provider as a *confirm item* — "the
   remote looks like X; keep manual PR handoff, or automate?" — and set
   `provider` / `pr_driver` **only** when the user explicitly opts in. Do not
   infer a `pr_command` from the provider.

7. **Fill config.toml**: update `.mochiflow/config.toml` with the confirmed
   surfaces, verify commands, git config, adapter, and `[i18n]`.
   Set `[i18n].artifact_language` to the language used for durable project
   artifacts. Set `[i18n].conversation_language` to `auto` unless the user wants
   a fixed conversation language.
   Remove a `# mochiflow: confirm` marker once its value is settled; keep the
   marker for anything still genuinely open.

8. **Run adapter generate safely**: run `mochiflow adapter generate` without
   `--force` first. Existing Markdown instruction files are preserved and
   extended with a MochiFlow-managed block. If structured adapter targets block
   generation, inspect the candidates under `.mochiflow/state/adapters/...` and
   merge or replace only with the user's approval. Use `mochiflow adapter
   generate --force` only after the user explicitly approves replacing existing
   adapter files.

9. **Generate the foundational context layer from code**: use
   `templates/context/{product,structure,tech}.md` for structure and write only
   the context files from what the code/config actually show:
   - `[context].product`: purpose / users / domain terms / core invariants / non-goals.
   - `[context].structure`: coarse code layout / entry points / "source is X,
     generated is Y, vendored is Z" map.
   - `[context].tech`: technology stack, verification surfaces, primary commands,
     generated artifacts, and contract/version gates.
   Include evidence pointers and the source commit/date for each context file.
   Keep the context layer to the minimal slice that is costly to re-derive yet
   rarely changes. Have the human confirm it matches current code. (The same
   procedure backs `commands/refresh-context.md`; onboard is its first run.)

10. **Leave constitution and ADR for their owners**: do not fill
   `[constitution].project` / `[constitution].local`; they are user-authored
   always-loaded rules. Do not fill `[adr].decisions` / `[adr].pitfalls` during
   onboarding; they grow only through ship-time folds. Present their paths and
   explain their purpose.

11. **Run doctor**: verify 0 fail. If fail, diagnose and fix (most common:
   adapter drift → regenerate, missing/unfilled context stubs → create or fill them).

12. **Present results and the usage card**: summarize what was done, list any
   `# mochiflow: confirm` values that are still open and need the user's OK, then
   run `mochiflow guide` and present its usage card (the four verbs and the two
   delivery approval gates) so the user knows how to drive MochiFlow next.
   Declare ready.

## Stop conditions

- Do not infer the adapter from file structure; confirm the tool(s) with the
  user (file traces may seed the default, but the choice is the user's).
- Do not auto-adopt a git provider or infer a `pr_command` from the remote. Keep
  `provider = "none"` (manual PR default) and only set `provider` / `pr_driver`
  when the user explicitly opts into automation.
- Read `# mochiflow: confirm` markers from the **raw** `config.toml` text; a
  parsed/loaded config drops comments and will miss them.
- Do not create a separate handoff / envelope / prompt file. The confirm markers
  in `config.toml` are the handoff.
- Do not modify existing source code (only `.mochiflow/`, `.kiro/`, adapter output).
- If the project already has a `.mochiflow/config.toml`, preserve existing
  choices unless they are clearly broken; ask before replacing the whole file.
- If `doctor` fails after 2 attempts, report the issue and stop.

## Examples

Input: "このプロジェクトに MochiFlow 入れて"
Output:
```
MochiFlow をセットアップしました。

検出:
- ビルド対象: web (Astro sites), api (Cloudflare Workers)
- 既定ブランチ: main
- 検証: web → bun run build, api → bun run build

確認事項:
- リモートは github のようです。PR は手動ハンドオフ（既定）のままにしますか、
  自動化しますか？ → 既定のまま（provider=none）

doctor: 0 fail ✓

使い方（mochiflow guide）:
  discuss / plan / build / ship の4動詞。承認は build 前とPR作成前の2回。
```
