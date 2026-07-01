<!-- {{marker}} -->
# AGENTS.md

This project uses mochiflow (vendored engine at `{{engine}}`). Load the router
as the standing route contract before any spec / implementation / PR work, then
follow it to load only the command-specific details needed for the selected
workflow.

## mochiflow

### Standing inputs

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Project context — **read before any work**: `{{context.product}}`,
  `{{context.structure}}`, and `{{context.tech}}`. This is the always-loaded
  current-state orientation; load it first.
- Router (read this first): `{{engine}}/router.md`
- Project config (surfaces / verify commands / git): run `mochiflow config show`

### Load on demand

- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`
- Non-phase commands: `{{engine}}/commands/{review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*, not current state):
  per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (each store has
  a generated, gitignored `INDEX.md`). Load the `INDEX.md` first, then open only
  active records whose `area` intersects the spec's `surfaces`
  (`mochiflow adr list | show | search`).

### Artifact roles

- `spec.md` is the product contract, `design.md` is the
  technical contract when required, `tasks.md` is the executable checklist when
  required, and the AC Matrix in `spec.md` tracks AC → implementation →
  verification → evidence → result.

## Rules

- Do not start a spec verb unless the user clearly intends it (`router.md` routing principles).
- Concrete small fixes stay in the spec lane; use plan when no active spec
  already scopes the work.
- Specs live under `{{specs_dir}}/{slug}/`; metadata is `spec.yaml` (status `draft → approved → accepted`; `done` is derived/legacy).
- Run verification via the command for the spec's surface from `[surfaces.<surface>.verify]`.
- Validate specs with `mochiflow lint`; quality gate is `mochiflow doctor`.
- Artifact language: `{{artifact_language}}`; conversation language:
  `{{conversation_language}}`. Follow `{{engine}}/reference/language.md` for
  user-facing wording and `auto` conversation behavior.
- At open, fold durable knowledge as new per-file records under
  `{{adr.decisions}}` (decisions) / `{{adr.pitfalls}}` (pitfalls) in the PR's
  close-out commit, superseding earlier records via `supersedes` /
  `superseded_by` rather than rewriting them; regenerate each store's gitignored
  `INDEX.md` (never stage it). The spec stays flat (no `_done/` move, never
  `status: done`). The
  context layer (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never
  folded.
