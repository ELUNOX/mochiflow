# Pre-approval review gate and PR-shipped context refresh

## Background and Design Rationale

Two parts of the workflow contract are inconsistent with the model the engine
already commits to elsewhere; this spec corrects both.

1. **Review ordering (Change A).** `commands/plan.md` sets `status: approved` and
   creates the plan commit (steps 8-9), then offers `mochiflow-review` only in
   the step-10 next-step card. Review is a spec/design **quality** review, so a
   `fail` means the spec just locked to `approved` and committed was not sound.
   For `risk >= elevated`, where review is the recommended default, the human
   therefore approves before the recommended quality check can inform that
   decision. The current placement was chosen by decision
   `2026-06-23-plan-to-build-transition-ux` to make review *visible*, not to make
   the ordering correct. Fix: for `risk >= elevated`, offer review *before* the
   approve-to-build gate; keep it optional and skippable; keep `standard`
   unchanged. **Dependency:** `agents/independent-reviewer.md` is framed for
   *post-implementation* review (`phases: [build]`, Stage 2 code quality, "Inputs
   from builder: changed files / full diff"). Running it on a code-less draft
   spec needs an explicit **plan-quality mode** (Stage 1 spec/design/tasks
   conformance + spec-artifact quality, with no diff/changed-files input), so the
   reviewer contract (`independent-reviewer.md`, referenced by `review.md`) is
   updated as part of Change A and backed by a conformance test.

2. **Context refresh timing (Change B).** When `commands/open.md` detects a
   coarse structural shift that staled the `[context]` layer, it records a
   *post-merge* `refresh-context` follow-up (`open.md` step 4;
   `reference/git.md ## Living-spec fold`; `commands/refresh-context.md`). But
   `commands/close.md` writes nothing to the base branch, so a post-merge refresh
   needs its own separate PR cycle and never rides the spec branch — noise. This
   contradicts the principle of decision `2026-06-27-post-build-pr-close-flow`:
   durable, judgment-bearing changes ship inside the open PR, reviewed and merged
   atomically. The deferral reason ("refresh writes files without auto-commit,
   dirtying the tree before PR pre-flight") is a sequencing artifact: pre-flight
   runs at `mochiflow pr`, *after* the accept close-out, so committing the
   regenerated context before that point keeps the tree clean. Fix: run
   `refresh-context` on the feature branch under human confirmation and commit it
   as a separate `docs(context)` commit, placed **after** the fold and
   context-check and **before** the `mochiflow accept` close-out commit, so the
   accept commit stays the final state commit and `mochiflow pr` pre-flight sees
   a clean tree; demote post-merge refresh to the fallback for staleness
   discovered only at or after merge. `refresh-context` itself keeps its
   no-auto-commit contract ("branch/PR handling outside this command") — `open`
   owns the regeneration trigger, human confirm, `git add` of the context paths,
   and the `docs(context)` commit.

Both changes are edits to the **engine source at repo-root `engine/...`** (the
`.mochiflow/engine/` copy is the gitignored, regenerated vendored mirror and is
never edited directly, per `constitution.md ## Dogfood`), plus a conformance-test
rewrite, the ADR fold, and the engine sync sequence (re-freeze + vendored
upgrade + adapter check). No Rust feature code, and no change to
`mochiflow accept` / `mochiflow pr` staging or pre-flight contracts.

## User Story

As a mochiflow user driving an elevated-risk spec, I want the recommended review
to run before I approve the plan, and I want context-layer refreshes to ship
inside the PR, so that I never approve an unreviewed spec and never get a noisy
post-merge context commit that cannot land on the spec branch.

## Scope

- In (all engine edits target repo-root `engine/...`, the tracked source):
  - `engine/commands/plan.md`: pre-approval review option for `risk >= elevated`;
    post-approval step-10 card reduced accordingly; aligned stop conditions.
  - `engine/agents/independent-reviewer.md`: add an explicit plan-quality review
    mode (Stage 1 conformance + spec-artifact quality, no diff/changed-files
    input) alongside the existing post-implementation mode; widen `phases`.
  - `engine/commands/review.md`: note that ad-hoc/pre-approval review on a
    code-less spec uses the reviewer's plan-quality mode.
  - `engine/reference/risk.md` (`## Review transport` / `## Ad-hoc review`): a
    one-line note that a code-less spec uses the no-diff plan-quality mode, so
    the review-inputs SSOT stays consistent with AC-10.
  - `engine/commands/open.md`: step-4 context-refresh check rewritten to an
    in-PR feature-branch refresh with post-merge fallback; the `docs(context)`
    commit placed explicitly after the fold/context-check and before the
    `mochiflow accept` close-out commit; the `(a)-(f)` sequence updated.
  - `engine/commands/refresh-context.md`: "When it runs" + procedure **and the
    frontmatter `description`** updated to the in-branch-before-PR primary path,
    keeping its no-auto-commit contract (the commit is `open`'s responsibility).
  - `engine/reference/git.md ## Living-spec fold` and
    `## Auto-commit and staging`: the context-refresh paragraph updated to the
    in-PR primary path; the `docs(context)` commit documented as a separate
    spec-lane commit preceding the single accept close-out commit.
  - `engine/reference/workflow.md`: clarify that review is a quality assist, not
    a delivery approval gate (the two gates are unchanged).
  - `engine/router.md`: update the `open` one-line summary (Verb Delegation /
    routing) to include the optional `docs(context)` commit before accept.
  - `cli/crates/mochiflow-cli/tests/conformance.rs`: (a) rewrite
    `open_defers_context_refresh_until_after_pr_or_merge` to pin the **new**
    in-PR-primary contract (not loosen/delete it); (b) add a Change A test
    pinning that `risk >= elevated` offers pre-approval review before confirm and
    `standard` keeps the prior order.
  - ADR fold at open: a decision record for each change (Change A supersedes
    `2026-06-23-plan-to-build-transition-ux`; Change B is a new record aligned
    with `2026-06-27-post-build-pr-close-flow`).
  - Engine sync per `constitution.md ## Dogfood` (`mochiflow freeze` →
    `mochiflow upgrade --source engine` → `mochiflow adapter generate --check`)
    and the surface `default` verify + lint pass.
- Out:
  - Making review mandatory; adding a third delivery gate; any new lint rule for
    review ordering.
  - Splitting a separate `agents/spec-reviewer.md` (a mode on the existing
    reviewer is sufficient and avoids a second reviewer contract).
  - Broadening the generated kiro adapter description
    (`engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`): the
    reviewer body is `file://`-referenced, so `adapter generate --check` does not
    drift and the description is non-behavioral. Left out of scope deliberately.
  - Widening `mochiflow accept` staging to include `[context]` files.
  - Editing the vendored `.mochiflow/engine/` copy directly (it is regenerated
    by `mochiflow upgrade --source engine`).
  - Retrofitting specs authored under the old flow or archived under `_done/`.

## Edge Cases

- A `risk = standard` plan: pre-approval review must NOT be forced; the existing
  approve-then-optional-review flow stays.
- The user confirms the plan directly without taking pre-approval review: allowed
  (review is optional); `status: approved` is set as today.
- Pre-approval review returns `fail`: findings reported, spec stays `draft`, no
  approve and no plan commit until the user re-confirms.
- `open` detects no structural shift: no `refresh-context`, no `docs(context)`
  commit, behavior unchanged.
- Context staleness discovered only after merge: routed to a follow-up (`fix`
  spec or backlog seed), not a base-branch edit.
- `refresh-context` triggered by open but the human does not confirm
  current-state accuracy: nothing is committed.

## Acceptance Criteria (EARS)

- AC-01: WHERE the spec `risk` is `elevated` or `critical`, THE SYSTEM SHALL
  offer a pre-approval review action in `plan.md`'s readiness card, presented
  before the confirm-plan (approve-to-build) action that sets `status: approved`.
- AC-02: WHEN the user selects the pre-approval review and the review verdict is
  `fail`, THE SYSTEM SHALL report the findings and SHALL NOT set
  `status: approved` or create the plan commit until the user re-confirms.
- AC-03: WHERE the spec `risk` is `standard`, THE SYSTEM SHALL leave the existing
  approve-then-optional-review flow in `plan.md` unchanged.
- AC-04: THE plan and workflow references SHALL state that review is a quality
  assist and not a delivery approval gate, keeping exactly two delivery gates
  (approve-to-build and approve-PR).
- AC-05: WHEN `open` detects a coarse structural shift that stales the
  `[context]` layer, THE SYSTEM SHALL run `refresh-context` on the feature
  branch and commit the regenerated context files as a separate `docs(context)`
  commit positioned after the fold/context-check and before the
  `mochiflow accept` close-out commit (which remains the final state commit), so
  the update ships in the PR and `mochiflow pr` pre-flight sees a clean tree.
- AC-06: BEFORE committing the `open`-triggered context refresh, THE SYSTEM SHALL
  require human confirmation that the regenerated context matches current code,
  and `open` (not `refresh-context`) SHALL own the `git add` and `docs(context)`
  commit, preserving `refresh-context`'s no-auto-commit contract.
- AC-07: WHERE context staleness is discovered only at or after merge, THE SYSTEM
  SHALL route it to a post-merge follow-up (a `fix` spec or a backlog seed)
  rather than a base-branch edit.
- AC-08: THE engine documents (`open.md`, `refresh-context.md`, `git.md`,
  `router.md`), including each file's frontmatter/metadata, SHALL NOT describe a
  post-merge `refresh-context` as the primary path for staleness detected during
  `open`, and the `open_defers_context_refresh_until_after_pr_or_merge`
  conformance test SHALL be rewritten (and MAY be renamed to reflect the
  in-PR-primary contract) to pin the new contract.
- AC-09: THE edited engine SHALL be synced via the `constitution.md ## Dogfood`
  sequence (`mochiflow freeze` → `mochiflow upgrade --source engine` →
  `mochiflow adapter generate --check`) and SHALL pass the surface `default`
  verification (including `freeze --check`) and `mochiflow lint`.
- AC-10: THE reviewer contract (`agents/independent-reviewer.md`) SHALL define a
  plan-quality review mode applicable to a code-less spec (Stage 1
  spec/design/tasks conformance + spec-artifact quality, with no diff or
  changed-files input required), `plan.md`'s pre-approval review and
  `review.md`'s ad-hoc review SHALL use it when no implementation exists, and
  `reference/risk.md` (`## Review transport` / `## Ad-hoc review`) SHALL note
  that a code-less spec uses this no-diff mode.
- AC-11: A conformance test SHALL assert that `plan.md` offers pre-approval
  review before the confirm-plan action for `risk >= elevated` and leaves the
  `standard` order unchanged.

## QA Scenarios

> Personas P1-P7 per `reference/risk.md ## QA attack coverage`. This is a
> docs/contract change with no runtime data surface, so AC-01..AC-04, AC-08,
> AC-11 are primarily guarded by conformance tests (automated); the QA below
> covers what tests cannot: boundary behavior, real git/file state, no-retrofit,
> regression, and document/frontmatter consistency. Data-integrity checks target
> git/file state, not a database.

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P2 | cli | Human-operated | Complete a plan on an `elevated` spec and drive the readiness card by label, keyword (`review`/`approved`), and number. | Review is offered before confirm-plan; keyword/number inputs route to pre-approval review then approve-to-build. |
| QA-02 | P3 | cli | Human-operated | On an `elevated` spec, take pre-approval review, force a `fail`, then try to proceed as approved; separately, confirm directly without review. | Findings reported and `approved`/commit refused until re-confirm; direct confirm without review still allowed (optional). |
| QA-03 | P4 | cli | Human-operated | After a `fail` pre-approval review inspect `spec.yaml` + `git log`; after an `open`-triggered refresh inspect branch commits, `[context]` files, and the position of the `docs(context)` commit relative to the accept close-out commit. | `status` stays `draft` on review fail (no plan commit); context files are committed in a `docs(context)` commit that precedes the accept close-out commit; `mochiflow pr` pre-flight tree is clean. |
| QA-04 | P5 | cli | Human-operated | Open a spec authored under the old flow and an archived `_done/` spec; trace the superseded ADR `2026-06-23-plan-to-build-transition-ux`. | Old/archived specs are not retrofitted; the superseded ADR keeps its immutable body with reciprocal `supersedes`/`superseded_by` links. |
| QA-05 | P6 | cli | Automated | Run the dogfood sequence (`mochiflow freeze` → `mochiflow upgrade --source engine` → `mochiflow adapter generate --check`) then the surface `default` verify and `mochiflow lint`. | Standard-risk flow, `accept`/`pr` contracts, lint, freeze, and both conformance tests (Change A + Change B) pass. Evidence recorded as QA-05. |
| QA-06 | P7 | cli | Human-operated | Compare the actual edited source wording of `engine/commands/plan.md`, `engine/commands/open.md`, `engine/commands/refresh-context.md` (incl. frontmatter), `engine/reference/git.md`, `engine/reference/workflow.md`, `engine/router.md`, `engine/agents/independent-reviewer.md` against the ACs. | Documented behavior matches the ACs; no residual post-merge-primary instruction in body or frontmatter; reviewer plan-quality mode present. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded; the engine re-freeze leaves
  `freeze --check` passing.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-05 | `engine/commands/plan.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | Pre-approval review before confirm for risk>=elevated (AC-11 test) |
| AC-02 | cli | AI-observed | QA-02, QA-03 | `engine/commands/plan.md` | UNVERIFIED | | Review fail keeps status draft, no plan commit |
| AC-03 | cli | automated | QA-05 | `engine/commands/plan.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | standard order unchanged (AC-11 test) |
| AC-04 | cli | AI-observed | QA-06 | `engine/commands/plan.md`, `engine/reference/workflow.md` | UNVERIFIED | | review documented as non-gate; two gates kept |
| AC-05 | cli | AI-observed | QA-03 | `engine/commands/open.md`, `engine/reference/git.md`, `engine/router.md` | UNVERIFIED | | docs(context) commit precedes accept close-out; (a)-(f) order explicit |
| AC-06 | cli | AI-observed | QA-03 | `engine/commands/refresh-context.md`, `engine/commands/open.md` | UNVERIFIED | | human confirms; open owns commit; refresh no-auto-commit |
| AC-07 | cli | AI-observed | QA-04 | `engine/reference/git.md`, `engine/commands/open.md` | UNVERIFIED | | post-merge fallback routes to follow-up |
| AC-08 | cli | automated | QA-06, QA-05 | `engine/commands/open.md`, `engine/commands/refresh-context.md`, `engine/reference/git.md`, `engine/router.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | no post-merge-primary (incl. frontmatter); Change B test re-pins new contract |
| AC-09 | cli | automated | QA-05 | `engine/MANIFEST.json`, `cli/` | UNVERIFIED | | dogfood sync (freeze → upgrade --source engine → adapter generate --check) + default verify + lint |
| AC-10 | cli | AI-observed | QA-06 | `engine/agents/independent-reviewer.md`, `engine/commands/review.md`, `engine/commands/plan.md`, `engine/reference/risk.md` | UNVERIFIED | | reviewer plan-quality mode for code-less specs; risk.md inputs note |
| AC-11 | cli | automated | QA-05, QA-01 | `cli/crates/mochiflow-cli/tests/conformance.rs`, `engine/commands/plan.md` | UNVERIFIED | | conformance test for Change A ordering |
