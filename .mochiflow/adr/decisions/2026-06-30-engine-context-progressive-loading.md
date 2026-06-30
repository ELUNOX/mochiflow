---
id: 2026-06-30-engine-context-progressive-loading
date: 2026-06-30
area: [cli]
spec: engine-context-progressive-loading
status: active
---
## 2026-06-30 - Keep routing compact and load engine detail by selected verb

**Decision:** `engine/router.md` remains the only always-read router artifact,
but it now defines a compact standing load contract instead of acting as a broad
index of all engine procedure detail. Adapters present constitution, project
context, router, and config as standing inputs, while command procedures,
cross-cutting references, and ADR history are explicitly load-on-demand. After
route selection, agents load the selected `commands/{verb}.md` file and that
command's file-level frontmatter references.

**Why:** The router is the correct place to decide intent, lifecycle state, and
which verb owns the next action, but keeping every procedure detail in the
standing layer makes first-turn context expensive and obscures the active
contract. Moving detail to verb-owned files preserves existing routing behavior
while making the initial instruction set smaller and easier to audit.

**Boundaries:** The change does not introduce `router.card.md`, section-level
frontmatter references, or a context-budget CLI command. File-level references
stay the compatibility boundary because supported agent surfaces do not all have
stable section-read semantics.

**Rejected:** generating a second route card from the router and command
frontmatter (adds a drift-prone authority); making adapter output a flat catalog
of engine files (keeps the original context-loading ambiguity); adding section
anchors as a shortcut (not portable enough across adapter surfaces).
