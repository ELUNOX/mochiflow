# Language Policy

## Response and generated artifacts

Conversational responses and generated spec artifacts (under `{specs_dir}`,
Decision summaries, AC matrices, QA notes) are written in the project language
declared by `language` in `config.toml`. Use `mochiflow config show` to read it.

## User-facing communication

MochiFlow uses precise internal vocabulary for routing and validation, but the
user experience should read like normal project collaboration. In ordinary
conversation and completion summaries, translate internal terms into plain
language in the configured project language. Keep internal terms for file names,
commands, metadata fields, schema enum values, headings inside generated spec
artifacts, and when the user explicitly uses the term.

Use these examples as meaning guides, not as a fixed dictionary:

| internal term | English user-facing phrasing | Japanese user-facing phrasing |
| --- | --- | --- |
| `spec` | work plan / implementation plan | 作業メモ / 計画 |
| `slug` | work ID (usually omit) | 作業ID（通常は省略） |
| `AC` / AC Matrix | acceptance checks / verification items | 確認項目 |
| `risk` | change impact / impact level | 影響範囲 |
| `lint` | consistency check | 整合性チェック |
| `doctor` | health check / quality check | 品質チェック |
| `build` phase | implementation work | 実装作業 |
| `ship` phase | wrap-up / PR preparation | 完了処理 / PR 作成準備 |
| `fold` | record durable learnings | 学びの記録 |
| `archive` | move to completed work | 完了済みに整理 |
| reviewer verdict | review result | レビュー結果 |

For other project languages, do not invent a fixed glossary. Preserve commands
and identifiers, and explain the same meaning in the user's language using
plain, non-internal wording. If an internal status matters, put it in a short
`MochiFlow:` note after the user-facing summary instead of making it the main
message.

## Engine documents

mochiflow engine files (`commands/**`, `reference/**`, `agents/**`,
`templates/**`) are written in **English** and stay project-agnostic. Add new
engine content in English; do not embed project-specific values, paths, or
commands (those belong in `config.toml`).

## Generated adapters

Adapter target files are generated artifacts (see `reference/git.md` and the
adapter templates). Their prose follows the project language; do not hand-edit
them.

## Git metadata

Commit messages, PR titles, and PR descriptions follow the project language.
Format is defined in `reference/git.md`.

## Code conventions

For source code, follow the existing convention of each file (identifiers,
comments, test names, error messages). Do not introduce a new language
convention into an existing file unless explicitly requested.
