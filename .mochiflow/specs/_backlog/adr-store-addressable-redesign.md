---
slug: "adr-store-addressable-redesign"
title: "Make the ADR store addressable and bounded: per-file records, content index, lint"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "close"
created: "2026-06-27"
updated: "2026-06-27"
---

# Make the ADR store addressable and bounded: per-file records, content index, lint

## Signal

`.mochiflow/adr/decisions.md` is a single append-only monolith that the `open`
fold grows on every spec. It is already long after ~14 entries and keeps
growing. Because the file is read whole, consulting a single past *why* forces
loading the entire history. ADR is `on-demand` (not always-loaded), so the cost
lands at the moments discuss / plan / review consult it — exactly when context
budget matters. The core defect is that retrieval is all-or-nothing: the store
is not addressable or filterable, so reads are `O(total history)` instead of
`O(relevant active records)`.

`pitfalls.md` has the same append-only shape but a different lifecycle: it holds
*active operational guardrails*, so obsolete entries should be retire-able,
unlike decisions which are immutable history.

## Why It Matters

Left as-is, the ADR corpus monotonically inflates the context cost of every
discuss/plan/review that needs prior rationale. This is the same class of
problem `post-build-pr-close-flow` solved for the board (stop committing a
growing artifact; derive a bounded view). Keep the *active/relevant* set bounded
as history grows, without losing any durable *why*.

## Framing: mochiflow already implements most of karpathy's llm-wiki pattern

karpathy's "llm-wiki" pattern (an LLM-maintained, compounding markdown knowledge
base; see Evidence) has three layers + a `index.md`(content-oriented) /
`log.md`(chronological, greppable) split + a recurring `lint` op. Mapped onto
mochiflow:

- **Raw sources (immutable, source of truth)** = the codebase (+ specs / PRs).
- **Synthesis "wiki" (LLM-maintained)** = the `context/` layer
  (product/structure/tech), already refreshed from code.
- **Schema (maintenance rules)** = the constitution + engine docs.
- **`log.md` (append-only chronological)** = `decisions.md` today — it even uses
  a greppable `## YYYY-MM-DD — slug` prefix.

So the pieces that are **missing** are exactly: a content-oriented **index**, and
a periodic **lint** (contradiction / stale-superseded / orphan / missing
cross-ref detection). This redesign fills those two gaps without violating
mochiflow's invariants.

## Scope (one spec, three phases)

1. **Per-file records + generated content index (non-destructive migration).**
   - Split the monolith into one record per decision:
     `adr/decisions/{YYYY-MM-DD}-{slug}.md`. Migration only *cuts out* existing
     entries verbatim — never rewrites their text (ADR immutability).
   - The `open` fold appends a new file instead of growing a monolith.
   - Generate `adr/decisions/INDEX.md` — the content-oriented catalog, one line
     per record (date, title, area, status). This is karpathy's `index.md`; the
     dated records themselves remain the `log`. Index size scales with record
     count, not body size, and is the only thing scanned by default.
   - Same shape for `pitfalls/` (per-file + generated index).
   - This phase alone removes most of the read-time pressure.

2. **Front-matter (area / status) + supersession + ADR lint (long-term bounding).**
   - Each record gains front-matter: `id`, `date`, `area` (module/surface tags),
     `spec` (source), `status: active | superseded | deprecated`, `superseded_by`.
   - Supersession instead of edit: a new decision that overturns an old one
     declares `supersedes:`; the old record flips to `status: superseded`.
     History is preserved; the active set shrinks. Add a "does this supersede a
     prior decision?" step to the `open` fold.
   - Pitfalls gain `status: active | resolved`; resolved guardrails drop out of
     the default read set (still retained as history).
   - **ADR lint** (karpathy's lint op): a recurring health-check that flags
     contradictions, stale/superseded entries, orphans (never referenced),
     missing cross-references, and dangling `superseded_by`. Surface as
     `mochiflow adr lint` and/or a `doctor adr` target.

3. **Retrieval command + selective load in discuss/plan (finish).**
   - Add a small read-only `mochiflow adr <list | show | search>` that filters by
     `area` / `status` / `slug`, returning headers by default and bodies on
     demand. No new runtime — reuse the existing metadata-reading / derivation
     patterns; index file over embedding search (single-binary fit).
   - discuss / plan load the index (cheap), then open only records whose `area`
     matches the spec's surfaces/modules and whose `status` is active.
     Superseded/deprecated records are read only when explicitly tracing lineage.

## Why It Matters (principle)

> The monolith forces "load everything." Make ADR addressable and filterable
> (per-file + front-matter + status + generated content index) and keep the
> active set bounded via supersession/retirement + a lint op — so reads become
> `O(relevant active records)`, not `O(total history)`.

## Evidence

- `.mochiflow/adr/decisions.md` — single append-only file, ~14 dated entries and
  growing one per delivered spec; `open.md` step (b) fold appends here. Uses a
  greppable `## YYYY-MM-DD — slug` prefix (already a `log.md`).
- `.mochiflow/adr/pitfalls.md` — same append-only shape, but entries are "active
  operational guardrails" (some already obsolete as code changes).
- karpathy, "llm-wiki" pattern:
  https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f — three
  layers (immutable raw sources / LLM-maintained synthesis wiki / schema),
  `index.md`(content) vs `log.md`(chronological) split, and an `ingest / query /
  lint` operation set. Validates the index/log split and contributes the lint op.
- Engine touch points: `engine/commands/open.md` (fold step), `reference/git.md`
  `## Living-spec fold`, the steering/router lines that describe ADR as on-demand
  `*why*`, and `mochiflow config show` `[adr]` paths.
- Precedent: `post-build-pr-close-flow` replaced a committed growing board with a
  derived, bounded view (same motivation).

## Decisions (tentative)

- One record per decision file; non-destructive split of the existing monolith
  (verbatim move, no rewrite).
- Generated content index (`INDEX.md`) as the default navigation surface; the
  dated records stay the chronological log.
- `status` + `supersedes`/`superseded_by` lifecycle for decisions; `active |
  resolved` for pitfalls.
- An ADR lint health-check (contradiction / stale / orphan / missing cross-ref).
- A read-only `mochiflow adr` retrieval command; discuss/plan selective load by
  area + active status.
- Explicitly NOT doing: lossy summarization/compaction of decision bodies (the
  *why* and rejected alternatives must stay intact; only the index is a digest);
  turning decisions into a rewritten synthesis wiki (breaks immutability);
  letting prose compete with code as truth (context stays code-derived);
  embedding search; high-density rewrite-many-pages-per-ingest (PR churn).

## Open Questions

- **Living rationale synthesis layer?** karpathy's wiki maintains a *current*
  synthesis. mochiflow synthesizes current *state* (`context/`) but not current
  *why*. Add a maintained "current rationale" page (supersession resolved) beside
  the immutable decision log, or stay with immutable-log + index + supersession
  only? Trade-off: synthesis value vs immutability/churn.
- Index: gitignored-and-regenerated (like `INDEX.md`) or committed-and-generated?
- Where do `[adr]` config paths point after the split — a directory plus an index
  file? Backward-compat for repos that still have the monolith?
- How is `area` assigned at fold time — derived from the spec's `surfaces` +
  touched modules, or prompted? How granular should tags be?
- Do existing `_done`/archived specs and their already-folded entries need any
  re-tagging, or only newly-folded records going forward?
- Risk level: likely `elevated` (ADR storage contract change + one-time
  migration + new command + fold-step change). Confirm at plan time and decide
  whether the migration is a separate reversible step.
- Should `doctor` own the ADR lint (index freshness, dangling `superseded_by`),
  or is a standalone `mochiflow adr lint` the right home?
