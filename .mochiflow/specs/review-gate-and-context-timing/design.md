# Pre-approval review gate and PR-shipped context refresh — Design

> Workflow-contract change to the engine source (repo-root `engine/...`).
> Decisions and contracts only; the concrete prose edits are made against the
> live engine files during build, then synced to the vendored copy.

## Design Decisions

- **Pre-approval review is gated on `risk >= elevated`, not universal.** The
  recommended-review nudge already keys on risk (decision
  `2026-06-23-plan-to-build-transition-ux`). Moving it before the approve gate
  only where it is recommended preserves `standard`'s fast path and avoids
  forcing an optional step on low-risk specs. Rationale: review is most valuable
  exactly where the spec is risky, and that is the only place the old ordering
  was actually backwards.
- **Review stays optional and is explicitly not a delivery gate.** The two
  delivery gates in `reference/workflow.md` (approve-to-build, approve-PR) are
  unchanged in count and meaning. The readiness card simply lets the user run
  review before exercising gate 1. A new lint rule is intentionally NOT added —
  ordering is agent procedure — but a **conformance test** pins the documented
  ordering (elevated offers review before confirm; standard unchanged) so the
  contract has a regression guard (AC-11).
- **Reviewer gets a plan-quality mode rather than a new agent.**
  `agents/independent-reviewer.md` is framed for post-implementation review
  (`phases: [build]`, Stage 2 code quality, "Inputs from builder: changed files /
  full diff"). Running it pre-approval on a code-less spec needs an explicit
  plan-quality mode (Stage 1 spec/design/tasks conformance + spec-artifact
  quality; no diff/changed-files input). A mode on the existing reviewer is
  chosen over a separate `agents/spec-reviewer.md` to avoid a second reviewer
  contract and keep one transport. `review.md` notes the ad-hoc path uses this
  mode when no implementation exists.
- **Context ships via a separate `docs(context)` commit ordered before accept.**
  Keeping the regenerated `[context]` files in their own feature-branch commit
  ships them in the PR (aligning with `2026-06-27-post-build-pr-close-flow`)
  without widening `mochiflow accept`'s deterministic staging contract
  (`spec dir + ADR paths` only). Because `mochiflow accept` stages only the spec
  dir + ADR, the context commit must be made by `open` and ordered **after** the
  fold/context-check and **before** the accept close-out commit. This keeps the
  accept commit the single final state commit and leaves `mochiflow pr`
  pre-flight a clean tree. `refresh-context` keeps its no-auto-commit contract
  ("branch/PR handling outside this command"); `open` owns the regeneration
  trigger, human confirm, `git add`, and the `docs(context)` commit.
- **Post-merge refresh is demoted to fallback.** It remains the route only for
  staleness discovered at/after merge, reusing the existing
  `git.md ## Living-spec fold` rule that post-merge knowledge goes to a follow-up
  `fix` spec or backlog seed — never a base-branch edit.
- **Engine source is repo-root `engine/...`; the vendored copy is regenerated.**
  Per `constitution.md ## Dogfood`, all edits target `engine/...` (tracked);
  `.mochiflow/engine/` is the gitignored vendored mirror and is never edited
  directly. After editing, run `mochiflow freeze` → `mochiflow upgrade --source
  engine` → `mochiflow adapter generate --check` so the manifest, the vendored
  copy, and the generated adapters stay in sync (pitfall
  `2026-06-25-editing-engine-requires-freeze`).

## Architecture

Edited engine documents (repo-root `engine/...`, the tracked source) and the
contract each carries:

- `engine/commands/plan.md`
  - Step 7: readiness card. For `risk >= elevated`, present Review (recommended)
    / Confirm the plan / revise. For `risk = standard`, unchanged (Confirm the
    plan / revise; review remains available post-approval at step 10).
  - Pre-approval Review: runs `mochiflow-review` on the draft spec; on
    `pass`/`pass-with-comments` re-present the approval action; on `fail` report
    findings and stop (spec stays `draft`).
  - Steps 8-9 (set `approved`, re-lint, commit) run only after the confirm-plan
    action.
  - Step 10: post-approval next-step card offers Start implementation / Create a
    resume prompt. When pre-approval review was already consumed (elevated),
    review is not re-offered; for `standard` the existing review option may
    remain.
  - Stop conditions updated to match.
- `engine/agents/independent-reviewer.md`: add the plan-quality mode (Stage 1 +
  spec-artifact quality, no diff input) alongside the post-implementation mode;
  widen `phases` so plan/ad-hoc review is in contract.
- `engine/commands/review.md`: ad-hoc review on a code-less spec uses the
  plan-quality mode.
- `engine/reference/risk.md` (`## Review transport` / `## Ad-hoc review`): a
  one-line note that a code-less spec uses the no-diff plan-quality mode (keeps
  the review-inputs SSOT consistent with the new mode).
- `engine/commands/open.md` step 4 (Foundational context refresh check): when a
  coarse structural shift is detected, run `refresh-context` on the feature
  branch (human confirm); `open` makes the `docs(context)` commit after the
  fold/context-check and before the `mochiflow accept` close-out commit. The
  `(a)-(f)` sequence becomes acceptance → fold + context-check → optional
  `docs(context)` commit → accept close-out → PR body → approve-PR → `mochiflow
  pr`. Post-merge follow-up is the fallback for at/after-merge discovery.
- `engine/commands/refresh-context.md`: "When it runs", procedure step 3, and the
  frontmatter `description` updated so the open-triggered path is
  in-branch-before-PR; keeps no-auto-commit (the commit is `open`'s).
- `engine/reference/git.md ## Living-spec fold` + `## Auto-commit and staging`:
  the context paragraph updated to the in-PR primary path with the preceding
  `docs(context)` commit; the single accept close-out commit framing kept; the
  at/after-merge fallback retained.
- `engine/reference/workflow.md`: a sentence in the gates / choice-cards area
  stating review is a quality assist, not a gate (no new gate).
- `engine/router.md`: the `open` one-line summary updated to mention the optional
  `docs(context)` commit before accept.
- `cli/crates/mochiflow-cli/tests/conformance.rs`: the
  `open_defers_context_refresh_until_after_pr_or_merge` test currently pins the
  OLD post-merge wording; it is rewritten to pin the new in-PR-primary contract,
  and a new test pins Change A's pre-approval ordering.

## Data Model / Interfaces

- No schema, CLI flag, or `config.toml` change. `spec.yaml` status values
  unchanged (`draft → approved → accepted`).
- New commit convention: a `docs(context): ...` Conventional Commit on the
  feature branch carrying only `[context]` files, with the `Spec: {slug}`
  trailer (it is a spec-lane commit). Documented in `git.md`.

## Error Handling

- Pre-approval review `fail`: no state change; spec stays `draft`; user decides
  to revise+re-review or confirm directly.
- `refresh-context` human non-confirmation: no commit; `open` may proceed with
  the context noted stale and a fallback follow-up recorded.
- Re-freeze omitted: `freeze --check` fails the surface `default` verify, caught
  by AC-09 before accept.

## Test Strategy

- Primary verification is the surface `default` profile
  (`cargo test` + `fmt --check` + `clippy -D warnings` +
  `freeze --check`) plus `mochiflow lint`, run after editing `engine/...` and
  running the dogfood sync (`mochiflow freeze` → `mochiflow upgrade --source
  engine` → `mochiflow adapter generate --check`) (AC-09 / QA-05).
- Two conformance tests guard the contract: (1) the rewritten
  `open_defers_context_refresh_until_after_pr_or_merge` re-pinned to the new
  in-PR-primary wording (AC-08), and (2) a new Change A test asserting
  `risk >= elevated` offers pre-approval review before confirm while `standard`
  keeps the prior order (AC-11). Both are rewritten/added to assert the new
  contract, never loosened.
- Remaining behavioral ACs (AC-02, AC-04..AC-07, AC-10) are AI-observed by
  reading the edited engine docs and confirming the documented flow matches each
  AC (QA-02..QA-04, QA-06), since the engine's behavior is defined by these
  documents.

## Review Results

The mandatory `risk: elevated` reviewer run is recorded here during build as
`Reviewer mode: delegated | inline` and `Verdict: pass | pass-with-comments | fail`.
Not yet run at plan time.
