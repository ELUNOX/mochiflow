# Make the ADR store addressable and bounded

## Problem

`.mochiflow/adr/decisions.md` is a single append-only monolith (currently ~424
lines / 15 dated entries) that the `open` fold grows by one entry per delivered
spec. Because the file is read whole, consulting a single past *why* forces
loading the entire history. ADR is on-demand (not always-loaded), so the cost
lands exactly when discuss / plan / review consult it — the moments context
budget matters most. `pitfalls.md` (~152 lines / 8 entries) has the same
append-only shape but a different lifecycle: it holds *active operational
guardrails*, some already obsolete.

The core defect is that retrieval is all-or-nothing: the store is not
addressable or filterable, so reads are `O(total history)` instead of
`O(relevant active records)`. This is the same class of problem
`post-build-pr-close-flow` solved for the spec board (stop committing a growing
artifact; derive a bounded view).

## Appetite

A single `elevated` spec covering all three phases (storage migration +
front-matter/supersession lifecycle + retrieval command). The project has no
external users yet, so we take the long-term-best design with a hard cut and
**no backward-compatibility code**, rather than a cautious incremental rollout.

## Solution

Make the ADR store per-file, addressable, and bounded, in three phases shipped
together.

**Phase 1 — per-file records + generated content index (non-destructive).**
- Split each monolith into one record per decision:
  `adr/decisions/{YYYY-MM-DD}-{slug}.md`; same shape for `adr/pitfalls/`.
  Migration only *cuts out* existing entries verbatim — never rewrites their
  text (ADR immutability).
- The `open` fold appends a new record file instead of growing a monolith.
- Generate a content-oriented `INDEX.md` per store (date, title, area, status),
  scaling with record count, not body size.

**Phase 2 — front-matter + supersession + ADR lint.**
- Each record gains front-matter: `id`, `date`, `area`, `spec`,
  `status: active | superseded | deprecated`, `superseded_by`. Pitfalls use
  `status: active | resolved`.
- Supersession instead of edit: a new decision declaring `supersedes:` flips the
  old record to `superseded`. History is preserved; the active set shrinks.
- `mochiflow adr lint` runs **deterministic structural checks only**: dangling
  `superseded_by`, orphans, stale (`superseded` still in the active set /
  unset `superseded_by`), missing bidirectional cross-refs, front-matter schema
  violations, INDEX freshness. A deterministic subset is wired into `doctor` as
  a quality gate; orphan/stale stay non-blocking warnings.

**Phase 3 — retrieval command + selective load.**
- A read-only `mochiflow adr <list | show | search>` covering both stores via
  `--kind decisions|pitfalls`, filterable by `--area` / `--status` / `--spec`,
  defaulting to `status: active`. `list`/`search` return headers; `show` returns
  full bodies and supersession lineage. No embedding — index-file / front-matter
  derivation, single-binary fit.
- discuss / plan load the cheap INDEX, then open only records whose `area`
  intersects the spec's `surfaces` and whose `status` is active.
  Superseded/deprecated records open only when explicitly tracing lineage.

**Cross-cutting decisions.**
- INDEX is **gitignored and regenerated** (consistent with the existing
  `INDEX.md` board precedent); retrieval derives its view from record
  front-matter at call time, so the INDEX is a convenience cache, never the only
  read path.
- `[adr]` config resolves to directories; there is **no legacy monolith read
  path**. The existing monolith is split once, in-spec, by a non-destructive
  verbatim migration that deletes the monolith in the same (separate, reversible)
  commit. No permanent `mochiflow adr migrate` command is shipped.
- `area` defaults to the spec's `surfaces`, assigned automatically while writing
  the fold (no dedicated confirmation prompt); shallow surface-based tags, with
  an extra tag only when a decision genuinely spans surfaces. PR review and the
  lint "unknown area" check catch mistakes.

## Rabbit Holes

- Semantic contradiction detection across decision bodies: needs content
  understanding (LLM), not a deterministic CLI check — excluded from lint.
- A maintained "current rationale" synthesis page: a third prose artifact that
  drifts and competes with code/immutable history — not built.
- Committed/generated INDEX: causes merge conflicts across concurrent folds and
  invites stale hand-edits — gitignored + regenerated instead.
- Module-path-granular `area` tags: go stale on refactors while decisions stay
  immutable — keep tags at surface granularity.

## No-gos

- Lossy summarization/compaction of decision bodies (the *why* and rejected
  alternatives stay intact; only the index is a digest).
- Rewriting decisions into a synthesis wiki (breaks immutability).
- Letting prose compete with code as the source of current state (context stays
  code-derived).
- Embedding-based search.
- Backward-compatibility / legacy monolith fallback code.
- A permanent migration command.

## Alternatives Considered

- **Ship Phase 1 only, defer 2–3:** lower risk, but the user chose one combined
  spec to land the full addressable design at once.
- **Warn-and-fallback backward compatibility:** unnecessary with no external
  users; a hard cut yields a cleaner long-term engine with no legacy branch.
- **Maintained current-rationale synthesis page:** rejected for drift, churn,
  and immutability conflict; `status: active` filtering over the immutable log
  delivers the same "current why" view without a second maintained artifact.
- **All checks (incl. contradictions) blocking in `doctor`:** over-sensitive
  gate; deterministic checks block, heuristic/semantic ones are out of scope.

## Open Questions

- Exact front-matter schema and `id` format (`{date}-{slug}` vs separate `id`),
  and how `superseded_by` references resolve — plan-time detail.
- Whether ADR INDEX regeneration reuses `mochiflow index` or a dedicated path,
  and the precise `[adr]` config keys after the directory switch — plan-time
  detail.
- Confirm the one-time migration is implemented as its own reversible commit
  separate from the engine/contract changes (assumed yes).
