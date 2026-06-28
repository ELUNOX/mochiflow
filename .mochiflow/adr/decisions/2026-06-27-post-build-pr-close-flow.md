---
id: 2026-06-27-post-build-pr-close-flow
date: 2026-06-27
area: [cli]
spec: post-build-pr-close-flow
status: active
---
## 2026-06-27 — post-build-pr-close-flow: ship → open/update/close, asserted vs derived state

**Decision:** Replace the single `ship` verb with three state-driven actions
`open` / `update` / `close`. Separate **asserted** spec states stored in
`spec.yaml` (`draft → approved → accepted`) from **derived** delivery facts
(`in_review`, `merged`) observed from VCS/provider and never stored. `done` is no
longer a written state — it survives only as a read-only legacy value for
archived `_done/` specs. Specs stay flat at `{specs_dir}/{slug}/` for life (no
`_done/` move). The kanban is computed: `mochiflow status` renders it on demand
and `INDEX.md` becomes a gitignored, regenerated cache. The living-spec fold
lands in the `open` PR (reviewed and merged atomically); `close` is local hygiene
only and writes nothing to the base branch.

**Why:** `ship` wrote a terminal state (`done` + `_done/` move + committed
`INDEX.md`) into tracked files *before* the PR opened. Because the base branch
cannot be committed to directly, that terminal write was forced pre-merge, and PR
feedback then had to resurrect completed work (`_done/` → active, `done` →
`approved`). Treating delivery state as observed rather than written removes every
post-merge base-branch write and eliminates the archive-before-PR and
resurrection problems at the root.

**Rejected:** A smarter `ship` (keeps the pre-merge terminal write);
archive-before-PR and a post-merge base commit at close (both require writing to
the base branch); a follow-up archive PR and an optimistic in-PR archive (extra
PR or premature state); a symlink projection board and a committed live
`INDEX.md` (merge churn / staleness); git hooks for freshness (hidden,
environment-coupled). Full list in the spec's `pitch.md ## Alternatives
Considered`.

**Consequence:** New status `accepted` added across the contract surface
(`contracts/spec.schema.json` enum, version gate / `contracts.lock`, `lint.rs`
allowed statuses + the matrix/reviewer gate, conformance fixtures), with `done`
kept valid only under `_done/`. New `delivery.rs` derives one column per spec
(Done > In Review > Ready > Active) and new `status.rs` + `mochiflow status`
render the board; `index.rs` columns come from the same derivation. `ship.rs` is
repurposed to `accept` (sets `accepted`, stages spec + ADR fold, single close-out
commit — no `done`/`_done`/INDEX). `pr` pre-flight now requires a committed flat
`accepted` spec with a `Spec:` trailer. New specs branch from
`origin/{base_branch}` with a stale-base warning. Engine docs/adapters were
rewritten to the open/update/close vocabulary. A reviewer verdict is required at
`accepted` for risk ≥ elevated.
