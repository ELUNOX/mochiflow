# Language Policy

MochiFlow separates language into three domains:

1. Engine language
2. Artifact language
3. Conversation language

## Engine Language

Engine files are always authored in English and are not configurable:

- `commands/**`
- `reference/**`
- `agents/**`
- `adapters/**`
- `templates/**`

Templates are English source templates. They describe structure, required
sections, and constraints; they are not the final prose language of rendered
artifacts.

## Artifact Language

Durable artifacts must be written in `[i18n].artifact_language` from
`config.toml`. Use `mochiflow config show` to read the resolved value.

Artifact language applies to generated project prose such as:

- `spec.md`, `design.md`, and `tasks.md`
- PR titles and descriptions
- QA instructions
- backlog seeds
- integration logs and review summaries
- session handoff prompts

Do not infer artifact language from the current chat turn unless onboarding has
no better signal. During setup, prefer explicit config / CLI options and
repository human-facing docs over the current conversation.

## Conversation Language

Conversation should use `[i18n].conversation_language`.

If `[i18n].conversation_language = "auto"`, respond in the user's current
language. If the current language cannot be inferred in a non-conversational
context, use the artifact language as a deterministic fallback.

MochiFlow uses precise internal vocabulary for routing and validation, but the
user experience should read like normal project collaboration. In ordinary
conversation and completion summaries, translate internal terms into plain
language. Keep internal terms for file names, commands, metadata fields, schema
enum values, and canonical table tokens required by tooling.

Choice-card labels are user-facing action labels, so they follow the
conversation language. Compatibility keywords such as `build`, `open`, `review`,
`later`, and `approved` remain stable inputs, but they should be secondary to the
plain action label displayed to the user.

## Delivery Next Actions

Delivery guidance is conversational and follows
`[i18n].conversation_language`. This covers the PR-created next action (merge the
PR, then report the merge in chat), the in-review and `local cleanup pending`
next-action hints in status / board output, and the `close` start and completion
wording. When `conversation_language = auto`, resolve per the Conversation
Language rule; CLI-only output (no live conversation context) falls back to
`[i18n].artifact_language` deterministically.

PR titles, PR descriptions, and other durable artifacts stay in
`[i18n].artifact_language`. The post-merge next action is therefore never written
into the PR body — it is local workflow guidance for the author, not review
material. Merge-report phrasings such as `merged` / `マージした` are illustrative
intent examples, not fixed trigger strings.

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
| `open` phase | PR preparation | PR 作成準備 |
| `update` phase | addressing PR feedback | PR フィードバック対応 |
| `close` phase | post-merge wrap-up | マージ後の整理 |
| `fold` | record durable learnings | 学びの記録 |
| reviewer verdict | review result | レビュー結果 |
| `seed` / backlog file | add to backlog | バックログに追加 |

## Stable Identifiers

Machine-readable identifiers and fixed workflow values remain stable and must
not be reworded or translated after selection:

- filenames and paths
- command names
- YAML keys and TOML keys
- status enum values
- branch prefixes
- adapter names
- canonical IDs such as `AC-01`, `QA-01`, `T-001`, `NFR-01`
- AC Verification Matrix result values from `workflow.md`: `PASS`,
  `CONFIRMED`, `N/A: <reason>`, `FAIL`, `PENDING_HUMAN`
  (deprecated aliases also accepted: `人間確認済み`, `対象外（<reason>）`)
- lifecycle and metadata enum values such as `draft`, `approved`, `done`,
  `feature`, `fix`, `refactor`, `docs`, `chore`, `standard`, `elevated`,
  `critical`

It is fine to explain these values in prose using the conversation or artifact
language, but preserve the token itself exactly where tooling expects it.

## Generated Adapters

Adapter target files are generated artifacts (see `reference/git.md` and the
adapter templates). Their prose follows the configured artifact/conversation
rules; do not hand-edit generated adapter targets.

## Git Metadata

Commit messages, PR titles, and PR descriptions follow artifact language. Format
is defined in `reference/git.md`.

## Code Conventions

For source code, follow the existing convention of each file (identifiers,
comments, test names, error messages). Do not introduce a new language
convention into an existing file unless explicitly requested.
