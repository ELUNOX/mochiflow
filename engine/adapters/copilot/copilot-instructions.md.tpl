<!-- {{marker}} -->
# Copilot Instructions

This project uses MochiFlow (engine at `{{engine}}`). Follow the router as
your standing instruction for any spec / implementation / PR work.

## MochiFlow

- Constitution — **always loaded**: `{{constitution.project}}` and
  `{{constitution.local}}` (user-authored project / local rules).
- Project context — **read before any work**: `{{context.product}}`,
  `{{context.structure}}`, and `{{context.tech}}`. Always-loaded current-state
  orientation; load it first.
- Router (read first): `{{engine}}/router.md`
- Verb procedures: `{{engine}}/commands/{discuss,plan,build,ship}.md`; patch lane:
  `{{engine}}/commands/patch.md`
- Non-phase commands: `{{engine}}/commands/{patch,review,refresh-context,onboard}.md`
- Cross-cutting rules: `{{engine}}/reference/{workflow,risk,authoring,git,language,engineering-standards}.md`
- Decision history / pitfalls — **on-demand** (*why*): `{{adr.decisions}}` / `{{adr.pitfalls}}`.
- Project config: `mochiflow config show`
- Artifact roles: `spec.md` is the product contract, `design.md` is the
  technical contract when required, `tasks.md` is the executable checklist when
  required, and the AC Matrix in `spec.md` tracks AC → implementation →
  verification → evidence → result.

## Rules

- Do not start a spec verb unless the user clearly intends it (see router.md routing principles).
- Use patch for concrete small fixes that do not need a spec; escalate to plan
  when a design decision, contract, migration, or higher risk appears.
- Specs live under `{{specs_dir}}/{slug}/`; metadata is `spec.yaml` (status: draft → approved → done).
- Run verification via `[surfaces.<surface>.verify]` in config.toml.
- Validate: `mochiflow lint` / `mochiflow doctor`.
- Artifact language: `{{artifact_language}}`; conversation language:
  `{{conversation_language}}`. Follow `{{engine}}/reference/language.md` for
  user-facing wording and `auto` conversation behavior.
- At ship, fold durable knowledge into `{{adr.decisions}}` / `{{adr.pitfalls}}` before
  archiving. Context (`{{context.product}}` / `{{context.structure}}` /
  `{{context.tech}}`) is refreshed from code (onboard / refresh-context), never folded.
- Do not call direct `git push` or provider PR creation commands; PR handoff
  goes through `mochiflow pr` after the PR content approval gate.
