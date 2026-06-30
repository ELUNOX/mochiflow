<!-- {{marker}} -->
# CLAUDE.md

This project uses MochiFlow (engine at `{{engine}}`). Load the standing inputs
below before any spec / implementation / PR work, then follow the router to load
only the command-specific details needed for the selected workflow.

## MochiFlow

### Standing inputs

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Project context — **read before any work**: `{{context.product}}`,
  `{{context.structure}}`, and `{{context.tech}}`. Always-loaded current-state
  orientation; load it first.
- Router (read first): `{{engine}}/router.md`
- Project config: `mochiflow config show`

### Load on demand

- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`; patch lane:
  `{{engine}}/commands/patch.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*): per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (generated gitignored `INDEX.md` per store; `mochiflow adr list | show | search`).
- Artifact roles: `spec.md` is the product contract, `design.md` is the
  technical contract when required, `tasks.md` is the executable checklist when
  required, and the AC Matrix in `spec.md` tracks AC → implementation →
  verification → evidence → result.

## Rules

- Do not start a spec verb unless the user clearly intends it (see router.md routing principles).
- Use patch for concrete small fixes that do not need a spec; escalate to plan
  when a design decision, contract, migration, or higher risk appears.
- Specs live under `{{specs_dir}}/{slug}/`; metadata is `spec.yaml` (status: draft → approved → accepted; `done` is derived/legacy).
- Run verification via `[surfaces.<surface>.verify]` in config.toml.
- Validate: `mochiflow lint` / `mochiflow doctor`.
- Artifact language: `{{artifact_language}}`; conversation language:
  `{{conversation_language}}`. Follow `{{engine}}/reference/language.md` for
  user-facing wording and `auto` conversation behavior.
- At open, fold durable knowledge as new per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` into
  the PR's close-out commit (supersede earlier records, never rewrite; regenerate the gitignored `INDEX.md`); the spec stays flat (no `_done/` move, never
  `status: done`). Context (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never folded.
