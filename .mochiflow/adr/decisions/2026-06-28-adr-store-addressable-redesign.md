---
id: 2026-06-28-adr-store-addressable-redesign
date: 2026-06-28
area: [cli]
spec: adr-store-addressable-redesign
status: active
---
## 2026-06-28 — ADR store: per-file addressable records with supersession

**Decision:** The ADR store (`[adr].decisions` / `[adr].pitfalls`) is
directory-rooted with one immutable record per decision/pitfall, a generated
gitignored `INDEX.md` per store, and a supersession lifecycle
(`supersedes`/`superseded_by`). `mochiflow adr lint` is deterministic-structural
only and wired into `doctor`; `mochiflow adr list | show | search` defaults to
the bounded active set. There is no legacy monolith read path and no permanent
migration command (hard cut).

**Why:** The monolith files forced `O(total history)` reads on every ADR
consultation during discuss / plan / review — exactly when context budget
matters. A per-file store with front-matter makes reads
`O(relevant active records)` by filtering on `area ∩ surfaces` + `status`.

**Rejected:** Keeping the monolith with section anchors (still whole-file reads,
no per-record metadata); a committed INDEX (merge conflicts on concurrent
folds); a maintained "current rationale" synthesis page (drifts, violates
immutability); embedding-based search (exceeds single-binary scope);
backward-compatibility / legacy-monolith fallback code (no external users, debt
for zero benefit); module-path-granular `area` tags (go stale on refactors).

**Consequence:** `init` scaffolds empty record directories (no monolith stub).
The `open` fold appends a per-file record and regenerates the gitignored INDEX.
`accept` stages the record directories but never `INDEX.md`. `id` and `area` are
required front-matter keys (gating on lint); `--status active` resolves to the
effective active set (supersession-aware). A `refresh-context` follow-up is
recommended post-merge (new `adr` module + CLI subcommand + doctor target).
