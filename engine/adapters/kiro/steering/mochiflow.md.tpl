---
inclusion: always
---
<!-- {{marker}} -->
# mochiflow

This project uses mochiflow (vendored engine at `{{engine}}`). This file is the
single always-on standing layer for Kiro: load the standing file references
below, then follow the router to load only the command-specific details needed
for the selected workflow.

## Always loaded

The following are pulled in as Kiro file references so they ride along with this
always-on steering file:

- Router (read this first): #[[file:{{engine}}/router.md]]
- Constitution (user-authored project / local rules): #[[file:{{constitution.project}}]] · #[[file:{{constitution.local}}]]

## mochiflow

### Load on demand

- Project context (current-state orientation): `{{context.product}}`,
  `{{context.structure}}`, `{{context.tech}}` — load when a selected workflow or
  repository-specific task needs orientation, not merely to route.
- Verb procedures: `{{engine}}/commands/{discuss,plan,build,open,update,close}.md`
- Non-phase commands: `{{engine}}/commands/{review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{lifecycle,specs,verification,risk,review,git,delivery,knowledge,language,presentation,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*, not current state):
  per-file records under `{{adr.decisions}}` / `{{adr.pitfalls}}` (each store has
  a generated, gitignored `INDEX.md`). Load the `INDEX.md` first, then open only
  active records whose `area` intersects the spec's `surfaces`
  (`mochiflow adr list | show | search`).
- Project config (surfaces / verify commands / git): run `mochiflow config show`
