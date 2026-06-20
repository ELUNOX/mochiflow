<!-- {{marker}} -->
# CLAUDE.md

This project uses MochiFlow (engine at `{{engine}}`). Follow the router as
your standing instruction for any spec / implementation / PR work.

## MochiFlow

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Project context — **read before any work**: `{{context.product}}`,
  `{{context.structure}}`, and `{{context.tech}}`. Always-loaded current-state
  orientation; load it first.
- Router (read first): `{{engine}}/router.md`
- Lifecycle verbs: `{{engine}}/commands/{discuss,plan,build,ship}.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language}.md`
- Decision history / pitfalls — **on-demand** (*why*): `{{adr.decisions}}` / `{{adr.pitfalls}}`.
- Project config: `mochiflow config show`

## Rules

- Do not start a spec verb unless the user clearly intends it (see router.md routing principles).
- Use patch for concrete small fixes that do not need a spec; escalate to plan
  when a design decision, contract, migration, or higher risk appears.
- Specs live under `{{specs_dir}}/{slug}/`; metadata is `spec.yaml` (status: draft → approved → done).
- Run verification via `[surfaces.<surface>.verify]` in config.toml.
- Validate: `mochiflow lint` / `mochiflow doctor`.
- Response and generated-artifact language: `{{language}}`; in user-facing
  speech, translate MochiFlow internal terms into plain project-language wording
  per `{{engine}}/reference/language.md`.
- At ship, fold durable knowledge into `{{adr.decisions}}` / `{{adr.pitfalls}}` before
  archiving. Context (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never folded.
