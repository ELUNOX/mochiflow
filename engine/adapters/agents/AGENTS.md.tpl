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
- Router (read this first): `{{engine}}/router.md`

### Load on demand

- Project context (current-state orientation): `{{context.product}}`,
  `{{context.structure}}`, `{{context.tech}}` — load when a selected workflow or
  repository-specific task needs orientation, not merely to route.
- Project config (surfaces / verify commands / git): run `mochiflow config show`
  when route resolution, verification, git, or adapter paths need it.
- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`
- Non-phase commands: `{{engine}}/commands/{review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{lifecycle,specs,verification,risk,review,git,delivery,knowledge,language,presentation,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*, not current state):
  per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (each store has
  a generated, gitignored `INDEX.md`). Load the `INDEX.md` first, then open only
  active records whose `area` intersects the spec's `surfaces`
  (`mochiflow adr list | show | search`).
