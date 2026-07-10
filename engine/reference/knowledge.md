# Knowledge Reference

ADR lookup, the living-spec fold, supersession, and foundational-context refresh.
Commit / staging mechanics live in `reference/git.md`; the accept close-out
commit that carries the fold lives in `reference/git.md ## Accept close-out
commit`; persistence modes live in `reference/delivery.md`.

## ADR lookup (why, not current state)

`[adr]` (directory-rooted `decisions` / `pitfalls` stores) is consulted only for
*why*, never as the source of current state — code is always the source of truth
for current state. When consulting ADR, load each store's generated `INDEX.md`
first, then open only the records whose `area` intersects the spec's `surfaces`
and whose `status` is active (`mochiflow adr list` / `search`). Open superseded /
deprecated records only when explicitly tracing supersession lineage. Each store
keeps a generated, gitignored `INDEX.md` content catalog; never stage it.

## Living-spec fold (on the feature branch, before `mochiflow pr`)

The fold happens **on the feature branch as part of the single close-out commit**
(`reference/git.md ## Accept close-out commit`), created before `mochiflow pr` —
never as a post-merge push to the base branch. This keeps the judgment-bearing
durable record inside the PR, under review. Fold only knowledge that **code
cannot reproduce**, as dated historical records — never as a "current state"
description (current state is always derived from code):

- The *why* behind design decisions / contracts (why a new type, schema shape,
  ownership, registry rule, or persistence model was chosen, and which
  alternatives were rejected) → add a new per-file record under `[adr].decisions`
  named `{YYYY-MM-DD}-{slug}.md` with front-matter (`id`, `date`, `area`
  defaulting to the spec's `surfaces`, `spec: {slug}`, `status: active`). Write it as a
  fact *as of that date*; never rewrite an existing record. When a decision
  overrides an earlier one, add the new record with `supersedes: <id>` and flip
  the old record to `status: superseded` with the reciprocal `superseded_by:
  <id>` (status/link change only — its body stays immutable).
- Operational pitfalls found during implementation (to prevent recurrence) →
  a new per-file record under `[adr].pitfalls`, using `Applies to`, `Signal`,
  `Cause`, `Guardrail`, `Check`, and `Status`. Resolved pitfalls flip to
  `status: resolved` rather than being deleted.

Each store keeps a generated, gitignored `INDEX.md` content catalog; regenerate
it after adding a record and **never stage it** (consistent with the board
`INDEX.md`). `mochiflow accept {slug}` stages only ADR record files linked to
that slug, plus reciprocal supersession records; the per-store `INDEX.md` files
are not staged.

Fold is skipped when the change yields no new rationale or pitfall (e.g. a trivial
display fix). Do not create the close-out commit until the fold (or the decision
that none is needed) is done.

## Foundational context is refreshed, not folded

Do not fold prose that describes current state ("how the system is put together
now", "where things live"). The context layer (`[context].product` /
`[context].structure` / `[context].tech`) is **not** a fold target — it is a
current-state orientation map regenerated from code via onboard /
`refresh-context`, never appended to during fold. For coarse code-layout changes
(new module, responsibility move, technology/verification change) detected during
`open`, run `refresh-context` (`commands/refresh-context.md`) on the feature
branch under human confirmation and ship the regenerated context **inside the PR** as a separate `docs(context)` commit placed after the fold/context-check and
before the accept close-out commit (`reference/git.md ## Auto-commit and
staging`); code remains the source of truth and the refresh is never folded.
Running the refresh in-branch before the PR is the primary path — never a post-merge base-branch edit.

## Knowledge discovered at or after merge

Knowledge discovered **at or after merge** is not appended to the merged spec
(that would re-introduce an unreviewed base-branch edit). Route it to a
follow-up: a small `fix` spec when it carries a code change, or a backlog seed
when it is pure rationale/pitfall for a later `discuss`. Context staleness
discovered only at or after merge follows the same rule (a `fix` spec when it
carries a code change, or a backlog seed for later `discuss`).
