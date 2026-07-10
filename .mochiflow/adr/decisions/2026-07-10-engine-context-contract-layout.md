---
id: 2026-07-10-engine-context-contract-layout
date: 2026-07-10
area: [cli]
spec: engine-context-slimming-redesign
status: active
supersedes: 2026-06-30-engine-context-progressive-loading
---
## 2026-07-10 - Make routing the sole standing engine contract

**Decision:** Keep only the user-authored constitution and `engine/router.md`
standing. The router owns all route vocabulary and selection. Project config,
foundational context, command procedures, policy references, templates, and ADR
records are loaded only when the selected workflow or repository task needs them.
Cross-cutting workflow rules live in responsibility-sized owner files and command
frontmatter declares required and conditional loads.

**Why:** The earlier progressive-loading layout still placed foundational context
and config in every adapter entrypoint and retained duplicated trigger discovery
in command metadata. A single routing authority plus conditional load graph keeps
first-turn instructions small while preserving portable file-level reads and the
existing workflow safety boundaries.

**Rejected:** keeping foundational context standing (broadens every interaction);
reintroducing command trigger metadata or a second route card (creates competing
route authorities); adding a prompt compiler, token budget, or section-anchor
read protocol (outside the portable engine contract).
