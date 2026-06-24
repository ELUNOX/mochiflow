# Kiro adapter: drop dedicated agent, single always-on steering, delegate permissions

## Problem

The Kiro adapter is the only adapter that ships a dedicated runtime agent
(`spec-builder.json`) with tool policy baked into its `toolsSettings`
(`shell.allowedCommands`/`deniedCommands`, `write.allowedPaths`/`deniedPaths`
rendered from `config.toml [write]`, a `web_fetch.blocked` rule), a ~30-entry
`resources` list, and a `subagent` trust declaration. Claude (`CLAUDE.md`) and
the generic `AGENTS.md` adapter instead inject a single managed instruction file
into the host's default agent and rely on prose guardrails — no baked agent, no
baked permissions.

That baked policy is a duplicated, drift-prone surface: it must track engine
reality every release, it is hashed into `engine/MANIFEST.json` (so any template
edit needs a re-freeze), and Kiro CLI 3.0's capability-based `permissions.yaml`
(resolution order
`deny > ask > allow`) now overlaps it — mochiflow's allowlist cannot loosen
anything a user denied anyway, and workspace permissions live per-user outside
the repo so a clone cannot inject rules. The asymmetry buys nothing and costs
maintenance, fixture churn, and a weaker-than-it-looks guard.

Verified against code (2026-06-24): `spec-builder.json.tpl` carries the full
`toolsSettings` + ~30 `resources` + `subagent.trustedAgents` + `prompt →
router.md` (its `model` is `"auto"`, not `claude-opus-4.8` as an earlier note
claimed); `spec-independent-reviewer.json` is already a thin read-only subagent
(`tools:[read,grep,glob]`); all 8 `.kiro/steering/spec*.md` are `inclusion:
manual` with no `inclusion: always` file, so the always-load layer rides
entirely on `spec-builder`'s `prompt` + `resources`. `AGENTS.md`/`CLAUDE.md`
reference the context layer by path (pointer), not inlined.

## Appetite

A focused refactor of the Kiro adapter generation + detection + validation, plus
the engine-integrity re-freeze (`mochiflow freeze` regenerates
`engine/MANIFEST.json`) and vendored-engine sync that editing `engine/` requires.
No `contracts.lock` / engine `VERSION` bump is needed because no `contracts/*.json`
schema changes (confirmed in plan; `manifest.version` tracks `VERSION` and stays
equal across a re-freeze). This is an alpha,
breaking-changes-OK redesign — no backward-compatibility ceremony for existing
Kiro projects.

## Solution

Zero-based redesign aligning Kiro with the prose-enforcement model already used
by Claude/AGENTS, while using Kiro's `inclusion` semantics only where it earns
its keep. The generated Kiro surface shrinks to **two files**:

1. `.kiro/steering/mochiflow.md` — `inclusion: always`, the single standing
   layer. Pulls in via Kiro-native `#[[file:...]]` **pointers** (not inlined):
   `router.md`, `constitution.md`, `constitution.local.md`, and
   `context/{product,structure,tech}.md`. Carries a Rules prose block equivalent
   to the `AGENTS.md`/`CLAUDE.md` Rules (push/PR only via `mochiflow pr`, no
   force-push / `reset --hard` etc., patch↔plan escalation, spec lifecycle,
   verify via `[surfaces.*].verify`, `lint`/`doctor` gates, artifact/conversation
   language).
2. `.kiro/agents/spec-independent-reviewer.json` — kept as-is: read-only
   subagent (`tools:[read,grep,glob]`), no `toolsSettings`, no `subagent` trust
   block.

Removed: `spec-builder.json` and **all** `spec-*.md` verb steering. Verb
procedures are no longer mirrored as steering files; the host default agent
loads `mochiflow.md` as always-on steering, and the router's existing step-8
lazy `fs_read` of `commands/{verb}.md` covers per-verb procedure loading.

Permissions are fully delegated to the user's `permissions.yaml`. All
`toolsSettings` are dropped from generated JSON; guardrails are carried only by
the `mochiflow.md` Rules prose. mochiflow does **not** touch, scaffold, or nudge
about `permissions.yaml` — it is entirely the user's responsibility.

Likely scope (for plan to expand): `cli/crates/mochiflow-core/src/adapter.rs`
(stop generating `spec-builder.json`; generate `mochiflow.md`; drop verb
steering; update `is_kiro_agent_json` and Kiro detection), the Kiro
`manifest.toml` + templates under `engine/adapters/kiro/**`, `doctor` /
`adapter generate --check` Kiro validation, self-healing removal of deprecated
Kiro outputs, `mochiflow freeze` + vendored-engine sync to preserve engine
integrity (no engine `VERSION` bump unless a `contracts/*.json` schema changes),
and the README/docs Kiro row. Tests in
`cli.rs`, `conformance.rs`, and `present.rs` reference `spec-builder.json` and
must be updated.

## Rabbit Holes

- Inlining the context layer into `mochiflow.md`: rejected. `#[[file:]]`
  pointers give the same always-loaded guarantee without coupling the generated
  steering file to `context/*.md` edits (which would make every
  `refresh-context` trip `adapter generate --check` and bloat golden fixtures).
- Building a careful marker-verified migration path for existing Kiro projects:
  out of scope given alpha / breaking-OK. Use simple self-healing regeneration
  with a static deprecated-path removal instead of a migration ceremony or a
  separate `prune` command.
- Re-introducing per-verb steering "for discoverability": rejected — it
  duplicates the router's lazy `fs_read`, adds 7 files + manifest + fixture
  churn, and breaks symmetry with Claude/AGENTS which ship no verb steering.

## No-gos

- No baked `toolsSettings` of any kind in generated Kiro JSON.
- mochiflow does not write, scaffold, or doctor-nudge `permissions.yaml`.
- No machine-enforced `git push` / provider-PR deny in-repo (it cannot live in
  the per-user, out-of-repo `permissions.yaml`); the invariant downgrades to
  `mochiflow.md` Rules prose, matching the Claude/AGENTS trust model.
- No pre-authorization of the reviewer subagent: per-call allow is accepted and
  mochiflow does nothing about it (read-only, no real risk).
- No verb-level steering files.

## Alternatives Considered

- Keep a minimal Kiro agent solely to wire `subagent` trust for the reviewer:
  rejected. The default agent can invoke the reviewer with a per-call allow,
  which is acceptable, removing the only reason to keep a baked agent.
- Hybrid (delegate permissions but keep verb steering): rejected in favor of the
  leaner two-file design (decision A) for symmetry and minimal generated
  surface / version-gate liability.

## Open Questions

- None — ready for plan.

## Related

- `.mochiflow/specs/_backlog/router-standing-load-weight.md` — this redesign
  absorbs the Kiro-side standing-load-footprint concern (drops the ~30-resource
  agent and per-verb steering). Agreed to delete that seed.
