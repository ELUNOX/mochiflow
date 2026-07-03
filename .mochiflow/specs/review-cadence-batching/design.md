# Batch bounded in-scope fixes: hold locally, review once before push/accept — Design

> Workflow-contract change to the engine source (repo-root `engine/...`).
> Decisions and contracts only; the concrete prose edits are made against the
> live engine files during build, then synced to the vendored copy.

## Design Decisions

- **The in-scope/out-of-scope judgment is generalized, not invented.**
  `build.md`'s existing stop condition ("an out-of-scope change or a new design
  decision") and `update.md`'s existing "unrelated feedback → new spec" rule
  already make this call. Moving the statement into `reference/risk.md` once,
  and having the three verb files reference it, follows the SSOT discipline in
  `reference/authoring.md` ("fix each fact in one place"). No new field, enum,
  or lint rule is added for the judgment itself — it stays an agent judgment
  call, same as today.
- **Hold/finalize reuses the existing explicit-command-vs-hint distinction,
  extended to slug-qualified patterns.** `router.md`'s Decision Flow already
  treats a `mochiflow-<verb>` token as an explicit command and a `{slug}
  <pattern>` match as an equally strong, immediate-activation signal (step 4:
  "declare the verb in one line and activate"), distinct from a bare
  natural-language hint (step 6: propose and wait). This spec draws the
  hold/finalize line along that same existing strength boundary rather than by
  trigger wording: **finalize** = the explicit `mochiflow-update` command, a
  match against any of `update.md`'s own `{slug} <pattern>` trigger patterns
  (`{slug} update`, `{slug} feedback`, `{slug} 修正依頼`, `{slug} PR
  feedback`), or an unambiguous completion statement (following the same
  "illustrative examples, not fixed trigger strings" pattern `router.md`
  already uses for merge-report phrasing); **hold** = the same natural-language
  hints used bare, with no slug qualifier (`修正依頼`, `PR feedback`,
  `PRを直して` said alone). `router.md`'s trigger/routing *decision logic* is
  not edited — only two descriptive spots that summarize `update`'s behavior
  (the "PR Feedback Loop Routing" sentence and the Verb Delegation table's
  `update` row; see the router.md bullet below), because both currently assert
  an unconditional push and would otherwise contradict `update.md`'s own
  contract, creating a dual-contract inconsistency in a standing, always-loaded
  document. `update.md`'s own frontmatter `description` has the same defect
  (it also asserts an unconditional push) and is reworded for the same reason.
- **`Reviewed through: <sha>` is additive, not a schema change, and stays on
  its own line.** It is a second, separate line inside the existing free-text
  `## Review Results` section, directly below `Verdict:` — never appended to
  the same line. `accept.rs::has_passing_review` matches a line starting
  `Verdict:` and requires the remainder to equal exactly `pass` /
  `pass-with-comments` (confirmed by reading `accept.rs` and the
  `lint.rs`/`conformance.rs` fixtures, none of which combine the two facts on
  one line); if `Reviewed through` were appended to the `Verdict:` line
  instead, the value would no longer match either accepted token and the
  accept gate would wrongly report "no passing verdict recorded" for every
  `risk >= elevated` spec. Requiring the separate line is therefore load-bearing
  for the Scope's "no Rust behavior change" boundary (`accept.rs` is never
  touched), not just a style preference.
  Nothing mechanically enforces freshness from this line yet; it exists so an
  agent (or a human reading `git log`) can compute held commits deterministically
  instead of relying on memory. A follow-up could teach `accept.rs` to check it,
  but that is explicitly out of scope here (see `pitch.md ## No-gos`).
- **`open`'s freshness trigger lives at the accept gate (step 6), not inside the
  QA round-trip.** The QA round-trip (step 3, including 3e's QA-`FAIL` rework)
  is skipped entirely when a spec has no human-operated/visual QA item
  (step (a).2: "If no such items exist, skip to (b)"). Placing the generalized
  check only in 3e would silently drop freshness enforcement for the common
  case of an elevated spec with only automated QA plus a held post-build
  bounded fix — exactly the scenario `spec.md`'s Edge Cases calls out. The
  check therefore belongs in step 6, immediately before `mochiflow accept`
  runs, as an unconditional precondition; 3e's own QA-`FAIL` rework becomes one
  path that can leave a commit beyond `Reviewed through`, caught by the same
  step-6 gate, rather than a second, independent re-review mechanism.
- **The trigger is scoped to code-changing commits, matching `update`'s
  existing carve-out.** `update.md`'s AC-05 already excludes a PR-body-only
  correction (no commit at all) from re-triggering review. `open` has a real
  analogue that *does* produce a commit: the optional `docs(context)` commit
  in step (c), which lands after the fold/context-check and before the step-6
  accept gate. Scoping the trigger to "a commit that changes code" (not "any
  commit") avoids a spurious re-review on that docs-only commit while keeping
  the invariant intact for the case that matters.
- **Reviewer diff scope is unchanged.** The mandatory reviewer keeps
  reconstructing the full diff from git (never conversation history or compact
  reports), per the existing fidelity principle in `risk.md ## Review
  transport`. Only the trigger frequency changes; this spec does not touch
  `agents/change-reviewer.md`.
- **Engine source is repo-root `engine/...`; the vendored copy is regenerated.**
  Per `constitution.md ## Dogfood`, edits target `engine/...` (tracked);
  `.mochiflow/engine/` is the gitignored vendored mirror, never edited directly.
  After editing, run `mochiflow freeze` → `mochiflow upgrade --source engine` →
  `mochiflow adapter generate --check` (pitfall
  `2026-06-25-editing-engine-requires-freeze`).

## Architecture

Edited engine documents (repo-root `engine/...`, the tracked source) and the
contract each carries:

- `engine/reference/risk.md` (`## Consequences`, "Verdict freshness"
  paragraph):
  - States the shared judgment once: an in-scope change has no task-structure
    change, no new AC, and no new design decision; an out-of-scope change keeps
    routing to `plan` (pre-`open`) or a new spec (post-`open`), unchanged.
  - Redefines the boundary: for an in-scope change, apply and commit locally
    (hold); the mandatory reviewer re-runs, at most once, at the next
    push/accept boundary — a `git push` that updates an open PR, or
    `mochiflow accept` — covering every code-changing commit held since the
    last recorded review. A non-code commit (docs/context/PR-body-only) does
    not by itself make a verdict stale.
  - Adds the `Reviewed through: <sha>` recording convention: every mandatory
    reviewer run (build's own elevated/critical run, and any later re-review)
    records `Reviewed through: <sha>` on its own line, directly below
    `Verdict:`, in `design.md ## Review Results`.
  - States explicitly that the reviewer's diff input stays the full diff from
    git — this paragraph changes trigger frequency only.
- `engine/templates/spec/design.md`: the `## Review Results` HTML comment gains
  a mention of `Reviewed through: <sha>` as a separate line below `Verdict:`.
- `engine/commands/update.md`:
  - Step 2 (apply requested code changes) becomes the hold path for the
    existing bare natural-language triggers (no slug qualifier): apply, commit
    with build discipline, re-verify, update AC Matrix rows touched — then stop
    and report a status line. No push, no `mochiflow accept`, no reviewer run.
  - A new explicit finalize path — the `mochiflow-update` command, a match
    against any of `update.md`'s own `{slug} <pattern>` trigger patterns
    (`{slug} update`, `{slug} feedback`, `{slug} 修正依頼`, `{slug} PR
    feedback`), or an unambiguous completion statement — runs steps 4-6 as
    today (review-if-stale once, on the full diff, when a code-changing commit
    is held; push; PR metadata) over every commit held since the last
    `Reviewed through` sha, not just the most recent one.
  - Every phrase pinned by `pr_feedback_routes_to_update_without_restore` stays
    verbatim: "do not move it", "do not revert", "bounded inline PR-feedback
    fix", "Build's eligibility gate", "flat".
  - A new stop condition: do not push automatically on a hold-only message; do
    not guess finalize from an ambiguous reply (ask once instead).
  - The frontmatter `description` ("update re-verifies, pushes, updates PR
    metadata...") is reworded to the same hold-by-default / finalize-pushes
    contract, so the summary that describes the procedure does not contradict
    it. The Presentation section's "Report what changed, what was re-verified,
    and what was pushed" line is made conditional on finalize (a hold-only
    turn reports what changed and was held; "what was pushed" only applies
    when finalize actually ran) for the same reason.
- `engine/router.md` (two spots only): (1) the "PR Feedback Loop Routing"
  sentence — "`update` applies bounded inline fixes, re-verifies, pushes, and
  updates the PR body when needed" — and (2) the Verb Delegation table's
  `update` row — "applies a bounded inline code fix, then re-verifies,
  pushes, and updates PR metadata" — are both reworded so neither asserts an
  unconditional, immediate push, matching `update.md`'s hold-by-default
  contract. The routing decision itself (PR feedback → `update.md`) and the
  substring "applies bounded inline fixes" (pinned by
  `pr_feedback_routes_to_update_without_restore`) are unchanged; no other line
  in `router.md`, and no trigger/routing decision logic, is touched.
- `engine/commands/open.md`:
  - Step 6 (immediately before `mochiflow accept {slug}` runs) gains an
    unconditional precondition: for `risk >= elevated`, when any code-changing
    commit exists beyond the recorded `Reviewed through` sha — whether it came
    from 3e's QA-`FAIL` rework or from a held post-build bounded fix carried
    into `open` — re-run `change-reviewer` once on the full diff, record the
    fresh verdict and updated `Reviewed through: <sha>`, then proceed to
    `mochiflow accept`. This fires independent of whether the QA round-trip
    (step 3) ran at all, so an elevated spec with only automated QA is still
    covered.
  - Step 3e's own wording changes from "re-run the reviewer" to reference the
    step-6 gate as the single place this happens, so the round-based batching
    already in 3e (multiple failed items in one round share one bounded fix)
    is unchanged and there is exactly one mechanism, not two.
  - The optional `docs(context)` commit in step (c) is explicitly noted as not
    triggering the step-6 check by itself (it is not a code-changing commit).
- `engine/commands/build.md`: a new subsection after the existing task loop
  (step 4 / "Stop conditions" area) documenting the post-all-tasks/pre-`open`
  window: an in-scope request is applied and committed with no `Task:` trailer
  (mirroring the existing convention for `open`'s QA-`FAIL` rework and
  `update`'s PR-feedback fix commits) and held; an out-of-scope request keeps
  routing to `plan`, reusing the exact existing stop-condition sentence ("Stop
  when an out-of-scope change or a new design decision is needed.") verbatim.
- `cli/crates/mochiflow-cli/tests/conformance.rs`: new tests (see Test
  Strategy) alongside the existing suite, which must keep passing unmodified.

## Data Model / Interfaces

- No schema, CLI flag, or `config.toml` change.
- No change to `spec.yaml` status values (`draft → approved → accepted`).
- New textual convention only: a `Reviewed through: <sha>` line inside the
  existing free-text `## Review Results` section of `design.md`, next to each
  `Reviewer mode:` / `Verdict:` pair. Not schema-enforced; not read by any Rust
  code in this spec.

## Error Handling

- Finalize triggered, fresh review returns `fail`: do not push; report
  findings; fix, re-verify, and re-commit before re-attempting finalize — the
  same discipline `build.md` already applies to a failing risk-cadence review.
- Ambiguous hold-or-finalize reply: do not guess finalize (which risks an
  unreviewed push); ask one clarifying question, per the new `update.md` stop
  condition.
- Reviewer dispatch unavailable at finalize time: falls back to inline reviewer
  role per the existing, unchanged `risk.md ## Review transport` selection —
  this spec does not change that fallback.
- Session ends mid-hold (commits exist, not yet finalized): recoverable purely
  from git (`Reviewed through: <sha>` vs. `HEAD`) and `design.md`; a resumed
  session re-derives "N held commits" the same way, with no dependency on
  conversation memory.

## Test Strategy

- Primary verification is the surface `default` profile (`cargo test` + `fmt
  --check` + `clippy -D warnings` + `freeze --check`) plus `mochiflow lint` and
  `mochiflow doctor`, run after editing `engine/...` and running the dogfood
  sync (AC-10 / QA-07).
- New conformance tests guard the contract (AC-09), each added alongside — never
  replacing — the existing suite:
  - `risk_defines_shared_bounded_fix_judgment_and_reviewed_through`: asserts
    `risk.md` states the in-scope/out-of-scope judgment once and documents the
    `Reviewed through: <sha>` convention on its own line below `Verdict:`;
    asserts the `design.md` template comment mentions it too (AC-01, AC-03).
  - `update_holds_fixes_until_explicit_finalize_signal`: asserts `update.md`
    documents hold-only handling for its bare natural-language triggers, and a
    distinct explicit finalize signal — including a match against its own
    `{slug} <pattern>` trigger patterns, not just `mochiflow-update` — before
    push; asserts the exact phrases pinned by
    `pr_feedback_routes_to_update_without_restore` are still present (AC-04,
    AC-05, AC-09).
  - `open_generalizes_freshness_trigger_to_reviewed_through`: asserts
    `open.md` step 6 states the accept-gate freshness check ("any code-changing
    commit beyond `Reviewed through`"), reachable independent of whether the
    QA round-trip (step 3) ran, and that 3e references the same step-6 gate
    rather than a second mechanism (AC-06).
  - `build_documents_post_completion_bounded_fix_window`: asserts `build.md`
    documents the post-all-tasks/pre-`open` bounded-fix+hold path and still
    contains the existing out-of-scope→`plan` stop-condition sentence verbatim
    (AC-02, AC-07).
  - `router_update_summary_matches_hold_by_default`: asserts both router.md
    spots (the PR-feedback-routing sentence and the Verb Delegation table's
    `update` row) no longer claim an unconditional immediate push, and that
    `update.md`'s own frontmatter `description` matches the same
    hold-by-default / finalize-pushes contract, while the routing decision and
    the pinned substring "applies bounded inline fixes" are unchanged (AC-04,
    AC-11).
- Per active pitfall `2026-06-28-conformance-substring-line-wrap`, every phrase
  a new test asserts must live on a single unwrapped line in the engine doc;
  prefer short, unlikely-to-wrap substrings (or two short contiguous tokens
  over one long phrase), and re-run the full `conformance` suite — not just the
  new test — after any rewrap, since a rewrap can break a pre-existing
  assertion on a neighboring phrase.
- Regression guard: `pr_feedback_routes_to_update_without_restore`,
  `build_ends_at_approved_without_pr_or_move`,
  `open_orders_acceptance_fold_accept_pr_gate`,
  `engine_open_update_close_defined_no_ship_verb`, and
  `branch_placeholders_use_prefix_slug` must all continue to pass unmodified
  (AC-09).
- Remaining behavioral ACs (AC-02, AC-06, AC-08, AC-11) are AI-observed by
  reading the edited engine docs and confirming the documented flow matches
  each AC, since the engine's behavior is defined by these documents (QA-01,
  QA-02, QA-03, QA-06, QA-08).

## Review Results

Review profile: change-reviewer
Reviewer mode: delegated
Verdict: pass
Reviewed through: 8365f03

Notes: Mandatory elevated-risk build review. The delegated reviewer reported no
findings and independently checked the full diff, conformance suite, freeze
check, spec lint, readiness, and doctor.
