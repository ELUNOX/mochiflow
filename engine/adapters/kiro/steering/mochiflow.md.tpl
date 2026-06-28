---
inclusion: always
---
<!-- {{marker}} -->
# mochiflow

This project uses mochiflow (vendored engine at `{{engine}}`). This file is the
single always-on standing layer for Kiro: load the router and the references
below, then follow the router for any spec / implementation / PR work.

## Always loaded

The following are pulled in as Kiro file references so they ride along with this
always-on steering file:

- Router (read this first): #[[file:{{engine}}/router.md]]
- Constitution (user-authored project / local rules): #[[file:{{constitution.project}}]] · #[[file:{{constitution.local}}]]
- Project context (current-state orientation): #[[file:{{context.product}}]] · #[[file:{{context.structure}}]] · #[[file:{{context.tech}}]]

## mochiflow

- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`; patch lane:
  `{{engine}}/commands/patch.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*, not current state):
  per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (each store has
  a generated, gitignored `INDEX.md`). Load the `INDEX.md` first, then open only
  active records whose `area` intersects the spec's `surfaces`
  (`mochiflow adr list | show | search`).
- Project config (surfaces / verify commands / git): run `mochiflow config show`
- Artifact roles: `spec.md` is the product contract, `design.md` is the
  technical contract when required, `tasks.md` is the executable checklist when
  required, and the AC Matrix in `spec.md` tracks AC → implementation →
  verification → evidence → result.

## Rules

- Do not start a spec verb unless the user clearly intends it (`router.md` routing principles).
- Use patch for concrete small fixes that do not need a spec; escalate to plan
  when a design decision, contract, migration, or higher risk appears.
- Specs live under `{{specs_dir}}/{slug}/`; metadata is `spec.yaml` (status `draft → approved → done`).
- Run verification via the command for the spec's surface from `[surfaces.<surface>.verify]`.
- Validate specs with `mochiflow lint`; quality gate is `mochiflow doctor`.
- Artifact language: `{{artifact_language}}`; conversation language:
  `{{conversation_language}}`. Follow `{{engine}}/reference/language.md` for
  user-facing wording and `auto` conversation behavior.
- At open, fold durable knowledge as new per-file records under
  `{{adr.decisions}}` (decisions) / `{{adr.pitfalls}}` (pitfalls) in the PR's
  close-out commit, superseding earlier records via `supersedes` /
  `superseded_by` rather than rewriting them; regenerate each store's gitignored
  `INDEX.md` (never stage it). The spec stays
  flat (no `_done/` move, never `status: done`). The
  context layer (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never
  folded.
