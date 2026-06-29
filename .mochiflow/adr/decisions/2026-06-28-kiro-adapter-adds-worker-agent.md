---
id: 2026-06-28-kiro-adapter-adds-worker-agent
date: 2026-06-28
area: [cli]
spec: build-orchestrator-subagent-execution
status: active
supersedes: 2026-06-24-kiro-adapter-always-on-steering
---
## 2026-06-28 — Kiro adapter generates a third file: the write-capable worker agent

**Decision:** The Kiro adapter now generates **three** files, not two: the
always-on steering (`.kiro/steering/mochiflow.md`), the read-only
`.kiro/agents/spec-independent-reviewer.json`, and the new write-capable
`.kiro/agents/spec-worker.json` (`tools: read/write/shell` — Kiro's coarse tool
categories, not finer names like grep/glob/edit/bash which Kiro renders as
"unknown"; prompt
→ `engine/agents/worker.md`, top model). It is generated from a
`spec-worker.json.tpl` + `manifest.toml` entry and recognized by
`adapter.rs is_kiro_agent_json`. This supersedes the "exactly two files" count in
`2026-06-24-kiro-adapter-always-on-steering`.

**Why:** The orchestrator/worker execution model
(`2026-06-28-build-orchestrator-disposable-workers`) needs a delegable
write-capable role on Kiro, analogous to the reviewer agent. The new agent keeps
the same generated, per-call-permission model as the reviewer (no baked tool
policy, no `toolsSettings`) — so the rationale that motivated the original ADR
(permissions belong to the user's `permissions.yaml`, not a baked policy) is
preserved; only the file count changes from two to three.

**Model enforcement (distinct from the reviewer):** unlike the reviewer, the
worker is **not** model-customizable. A reviewer-only `kiro_agent_preserves_model`
predicate keeps model-preservation for the reviewer, so the worker always renders
the top model (no downgrade): a user-pinned worker model is real drift that
`adapter generate --check` flags and regenerate overwrites.

**Rejected:** baking a deny/trust tool policy into the worker agent (reintroduces
the very `toolsSettings` coupling the original ADR removed); opting the worker
into the reviewer's model-preservation (would silently allow a worker model
downgrade, violating the no-downgrade contract).
