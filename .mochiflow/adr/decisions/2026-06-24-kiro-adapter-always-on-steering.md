---
id: 2026-06-24-kiro-adapter-always-on-steering
date: 2026-06-24
area: [cli]
status: active
---
## 2026-06-24 — Kiro adapter: always-on steering replaces dedicated agent

**Decision:** The Kiro adapter generates exactly two files:
`.kiro/steering/mochiflow.md` (`inclusion: always`, with `#[[file:]]` pointers
for the router, constitution, and context) and the read-only reviewer agent. The
former `spec-builder.json` (with `toolsSettings` / ~30 resources / subagent
trust) and eight per-verb `spec-*.md` steering files are removed. Permissions
are delegated entirely to the user's `permissions.yaml`.

**Why:** The baked tool policy duplicated what `permissions.yaml` now owns
(`deny > ask > allow`, per-user, outside the repo), drifted on every engine
release, bloated `MANIFEST.json`, and created asymmetry with the Claude/AGENTS
adapters that use prose-only guardrails. The router's existing lazy `fs_read` of
`commands/{verb}.md` covers verb procedure loading without mirrored steering.

**Rejected:** Keeping a minimal agent only to wire `subagent.trustedAgents` for
the reviewer (per-call allow is acceptable for a read-only agent). Hybrid:
delegate permissions but keep verb steering (breaks symmetry, adds 7 files +
manifest churn). Inlining context in `mochiflow.md` (would couple it to every
`refresh-context`, tripping `adapter generate --check`).
