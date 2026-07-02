---
id: 2026-07-02-local-only-spec-mode
date: 2026-07-02
area: [cli]
spec: local-only-spec-mode
status: active
---
## 2026-07-02 - Local-only specs use Git ignore state as the persistence contract

**Decision:** MochiFlow now treats the concrete spec artifact's Git ignore state
as the source of truth for delivery persistence. If the configured spec path is
not ignored, tracked mode keeps the existing accepted close-out commit and
`Spec:` trailer contract. If the path is ignored, local mode still runs the same
acceptance quality checks but updates local spec artifacts without staging or
committing ignored `.mochiflow/` paths, and PR handoff relies on local accepted
evidence plus an evidence-rich PR body.

**Why:** Repositories already express their persistence policy through
`.gitignore`. Adding a separate config switch would create two authorities that
can disagree, while force-adding ignored spec artifacts would violate the
repository policy. Using the actual ignored/not-ignored path keeps the behavior
observable, deterministic, and compatible with existing tracked repositories.

**Key sub-decisions:**
- One shared core detector classifies spec persistence mode so `accept`, `pr`,
  and delivery derivation do not invent separate rules.
- Local mode is a persistence difference only. Final verification, lint, AC
  Matrix completeness, and required reviewer results remain mandatory before
  acceptance or PR handoff.
- Tracked mode remains strict. A committed accepted spec with a `Spec:` trailer
  is still required before tracked-mode PR handoff.
- Manual local-mode delivery can derive merge completion from source branch-tip
  reachability when provider state and committed trailers are unavailable.
- The PR body becomes the reviewer-facing evidence carrier in local mode because
  ignored local spec artifacts do not travel with the branch.

**Rejected:** A `specs.mode` override was rejected because it could drift from
Git policy. Force-adding ignored `.mochiflow/` artifacts was rejected because it
turns a local-only policy into a tracked artifact by surprise. Weakening
tracked-mode trailer validation was rejected because tracked repositories still
use the close-out commit as their audit contract.
