<!-- {{marker}} -->
# AGENTS.md

This project uses mochiflow (vendored engine at `{{engine}}`). Load the router as
your standing instruction and follow it for any spec / implementation / PR work.

## mochiflow

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Project context — **read before any work**: `{{context.product}}`,
  `{{context.structure}}`, and `{{context.tech}}`. This is the always-loaded
  current-state orientation; load it first.
- Router (read this first): `{{engine}}/router.md`
- Lifecycle verbs: `{{engine}}/commands/{discuss,plan,build,ship}.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language}.md`
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
- Response and generated-artifact language: `{{language}}`; in user-facing
  speech, translate MochiFlow internal terms into plain project-language wording
  per `{{engine}}/reference/language.md`.
- At ship, fold durable knowledge into `{{adr.decisions}}` (decisions) /
  `{{adr.pitfalls}}` (pitfalls) before archiving to `{{specs_dir}}/_done/`. The
  context layer (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never
  folded.
- Do not call direct `git push` or provider PR creation commands; PR handoff
  goes through `mochiflow pr` after the PR content approval gate.
