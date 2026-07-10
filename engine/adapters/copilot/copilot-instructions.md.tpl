<!-- {{marker}} -->
# Copilot Instructions

This project uses MochiFlow (engine at `{{engine}}`). Load the standing inputs
below before any spec / implementation / PR work, then follow the router to load
only the command-specific details needed for the selected workflow.

## MochiFlow

### Standing inputs

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Router (read first): `{{engine}}/router.md`

### Load on demand

- Project context (current-state orientation): `{{context.product}}`,
  `{{context.structure}}`, `{{context.tech}}` — load when a selected workflow or
  repository-specific task needs orientation, not merely to route.
- Project config: `mochiflow config show` when route resolution, verification,
  git, or adapter paths need it.
- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`
- Non-phase commands: `{{engine}}/commands/{review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{lifecycle,specs,verification,risk,review,git,delivery,knowledge,language,presentation,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*): per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (generated gitignored `INDEX.md` per store; `mochiflow adr list | show | search`).
