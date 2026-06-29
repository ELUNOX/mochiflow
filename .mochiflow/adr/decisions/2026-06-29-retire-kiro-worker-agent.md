---
id: 2026-06-29-retire-kiro-worker-agent
date: 2026-06-29
area: [cli]
spec: retire-build-worker-orchestrator
status: active
supersedes: 2026-06-28-kiro-adapter-adds-worker-agent
---
## 2026-06-29 - Retire the generated Kiro worker agent

**Decision:** The Kiro adapter returns to the two-file generated shape:
`.kiro/steering/mochiflow.md` and
`.kiro/agents/spec-independent-reviewer.json`. It no longer generates
`.kiro/agents/spec-worker.json`; markered legacy worker files are treated as
deprecated generated residue and removed by adapter generation, while markerless
files at the same path are preserved as user-owned.

**Why:** Once implementation no longer delegates to a write-capable worker, the
generated Kiro worker agent has no active contract to serve. Keeping it would
make adapter output imply an execution path the engine no longer supports.

**Relation:** This effectively restores the two-file adapter shape from
`2026-06-24-kiro-adapter-always-on-steering` while preserving the later
read-only independent reviewer. The pitfall
`2026-06-28-kiro-agent-tools-are-coarse-categories` remains relevant for
generated Kiro agents in general, but its worker-specific guidance and tests are
obsolete because the worker agent is removed.

**Rejected:** leaving the generated worker file orphaned, or preserving the
worker model-enforcement branch in `adapter.rs`. Both would keep maintenance
surface for a removed role.
