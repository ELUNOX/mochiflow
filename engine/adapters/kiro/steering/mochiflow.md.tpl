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

- Verb procedures: `{{engine}}/commands/{discuss,plan,build,ship}.md`; patch lane:
  `{{engine}}/commands/patch.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*, not current state):
  `{{adr.decisions}}` / `{{adr.pitfalls}}`.
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
- At ship, fold durable knowledge into `{{adr.decisions}}` (decisions) /
  `{{adr.pitfalls}}` (pitfalls) before archiving to `{{specs_dir}}/_done/`. The
  context layer (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never
  folded.
- Do not call direct `git push` or provider PR creation commands; PR handoff
  goes through `mochiflow pr` after the PR content approval gate.
- Permissions (shell / write / fetch) are owned entirely by your Kiro
  `permissions.yaml` (`deny > ask > allow`); mochiflow ships no baked tool policy.
