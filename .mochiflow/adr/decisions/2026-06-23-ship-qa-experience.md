---
id: 2026-06-23-ship-qa-experience
date: 2026-06-23
area: [cli]
spec: ship-qa-experience
status: active
---
## 2026-06-23 — ship-qa-experience: QA single source + round-trip protocol

**Decision:** spec.md QA Scenarios is the sole QA source of truth.
`qa-instructions.md` (template and ephemeral generation) is removed. Ship
acceptance uses a structured round-trip protocol (numbered list → free-form
response → canonical token recording → rework loop). PR reviewers read a derived
`## Testing` section in the PR body.

**Why:** QA information was split across three locations (spec.md, ephemeral
qa-instructions.md, AC Matrix), with no view reaching PR reviewers and no
defined rework path for FAIL results. The split caused: invisible QA for
reviewers, ambiguous result collection, and stalled deliveries on FAIL.

**Rejected:** Keeping qa-instructions.md as optional (preserves the source split
and leaves the reviewer gap unresolved); defining a fixed response-token
dictionary for human QA input (contradicts language.md's free-form intent model
and fails for unsupported locales); splitting into 4 separate specs (parallel
edits to the same files).

**Consequence:** ship.md Acceptance rewritten (steps 1–3f). PR Feedback Loop
gains router triggers (`{slug} feedback` / `修正依頼` / `PR feedback`) with a
`_done` resolution exception. `pr-description.md` gains `## Testing`.
`spec.standard.md` QA table gains a `Type` column. Three conformance tests
migrated to new sources. MANIFEST and vendored engine updated.
