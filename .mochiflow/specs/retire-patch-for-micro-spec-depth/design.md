# Retire patch and redefine micro as the fastest spec depth - Design

## Design Decisions

- **This redefines the existing partial micro depth.** Micro is not new in the
  engine: `workflow.md` already has a Micro spec row (`pitch.md` + `spec.md`),
  `plan.md` references `spec.micro.md`, and `risk.md` / `git.md` / `build.md`
  already mention micro. This spec replaces that partial definition with the
  new pitchless micro shape; build must update the existing Micro row rather
  than leave two conflicting definitions.
- **Micro is an artifact shape, not metadata.** Depth is inferred from which
  files exist. A micro spec has `spec.yaml` and `spec.md` only. There is no
  `depth` field, schema update, contract lock update, or version bump. This keeps
  `risk` as the only stored gate axis and avoids file-vs-field drift.
- **Micro eligibility is deliberately narrow.** Because `risk.md` requires
  `design.md` for elevated/critical risk, integration, multiple surfaces, and
  other design-required impacts, micro is eligible only for standard-risk,
  single-surface, `integration: none` changes with no human QA and no durable ADR
  fold need. If those facts stop being true, the same spec escalates in place by
  adding the artifacts required by the higher depth.
- **Plan owns direct micro creation and the first durable commit.** `discuss`
  remains the right entry for ambiguous work, but an explicit concrete request
  can enter `plan` directly and produce a micro spec without `pitch.md`. In that
  direct-micro path, `plan` first owns intake: derive a proposed `slug`,
  `title`, `type`, `surfaces`, `integration`, and `risk` from the concrete
  request, current config, and code context; present that metadata to the user
  for confirmation or correction; check duplicate active/_done/backlog slugs;
  and write `spec.yaml` from the template. Only after that does `plan` take over
  the branch/durability responsibility that `discuss` normally has: create or
  switch `{prefix}/{slug}` from `origin/{base_branch}`, then commit `spec.yaml`
  + `spec.md` as a draft micro spec before presenting the approve-to-build gate.
  This preserves the session-loss protection introduced by commit lifecycle
  unification. With no active spec and only a vague small-fix phrase, the router
  proposes planning and waits, preserving the no-activation-without-explicit-
  intent rule.
- **Pitchless draft ambiguity is accepted by design.** Without a `depth` field,
  lint cannot distinguish an intended standard draft that forgot `pitch.md` from
  a valid pitchless micro draft when both have `spec.md`, no `pitch.md`, and
  standard-risk single-surface `integration: none` metadata. The chosen rule is
  therefore explicit: such a draft is treated as micro. If the author intended a
  larger spec, adding design-required metadata/files or `pitch.md` moves it out
  of the micro branch. This is the cost of preserving no new metadata.
- **`mochiflow-patch` is a deprecation notice, not a compatibility lane.** The
  token is recognized only to tell the user patch was retired and to route
  toward `plan`. There is no patch procedure, no patch commit rule, no patch
  verification section, and no active command catalog entry.
- **Every spec delivers through PR.** Removing the no-PR fast path keeps the two
  delivery gates consistent for every depth. Micro reduces planning overhead,
  not delivery traceability.
- **ADR fold is an escalation signal for micro.** A micro spec carries no fold by
  definition. If the work produces decision rationale or a pitfall worth folding,
  that fact proves the work is no longer micro; add the required artifacts and
  fold normally during `open`.
- **Dogfood edits target repo-root `engine/`.** Per constitution, repo-root
  `engine/...` is the source. The vendored `.mochiflow/engine/` copy is updated
  only via `mochiflow upgrade --source engine` after `mochiflow freeze`.

## Architecture

- `engine/router.md`
  - Remove `patch` from the normal routing vocabulary, command catalog, verb
    delegation, transition discipline, and patch eligibility branch.
  - Add the deprecated-token branch for `mochiflow-patch`.
  - Route concrete small-fix/no-spec phrasings without an active spec as plan
    intent that asks before activation.
- `engine/commands/`
  - Delete or retire `patch.md` as an active command source.
  - Update `plan.md` prerequisites and procedure so micro can be created without
    `pitch.md` when the user request is explicit and concrete. For that
    direct-micro path, `plan.md` derives/confirms required metadata, checks slug
    duplicates, writes `spec.yaml`, prepares the feature branch from
    `origin/{base_branch}`, and creates the initial `docs(spec): ...` draft
    micro commit before approval. Its frontmatter prerequisites become
    conditional: `pitch.md` is required for standard-or-larger plan, not for
    direct micro.
  - Update `discuss.md` so it remains optional for micro and required for
    ambiguous/standard-or-larger handoff.
  - Update `build.md`, `open.md`, and `update.md` to remove patch/no-PR
    references and keep micro on the normal spec path.
- `engine/reference/`
  - `workflow.md`: replace the patch-vs-spec lane with a single spec lane whose
    first depth is micro; update the existing Depth scaling Micro row from
    `pitch.md + spec.md` to `spec.yaml + spec.md`; remove patch verification and
    no-PR gate exception.
  - `git.md`: remove patch commit and no-PR rules; keep taskless/micro build
    commit language; document that direct-micro `plan` performs branch
    creation/switch and the first draft commit when `discuss` was skipped.
  - `risk.md`: document micro escalation where the existing design-required
    conditions make micro impossible.
  - `authoring.md`: document micro as `spec.yaml` + `spec.md` and update
    pitch/plan wording.
- `engine/templates/spec/spec.micro.md`
  - Expand the template enough to include problem/goal, scope, EARS AC, QA
    scenarios when needed, completion conditions, and a minimal
    `## Verification Plan / AC Matrix` table in `spec.md`. Reverse the existing
    `micro_template_has_no_ac_verification_matrix` conformance test so it asserts
    the Matrix is now present.
- Adapter/public docs
  - Update agents, Claude, Copilot, and Kiro adapter templates so generated
    entrypoints no longer mention patch as a lane or command.
  - Preserve Kiro self-heal cleanup for older generated `spec-patch.md` files.
  - Update `README.md` and `engine/README.md` wording.
- Rust code/tests
  - `lint.rs`: branch draft validation into pitch-only, expanded-with-pitch, and
    micro-without-pitch. A draft with `spec.md`, no `pitch.md`, and eligible
    micro metadata is treated as micro; there is no mechanical way to reject a
    forgotten-pitch standard draft with the same signature without adding
    metadata. Approved/accepted still require `spec.md` and Matrix coverage.
  - `doctor.rs`: remove `patch` from active workflow command references.
  - `adapter.rs`: keep deprecated Kiro path cleanup but do not imply active
    patch generation.
  - `conformance.rs`: replace old patch/no-PR tests with tests for retired patch,
    micro draft shape, Matrix template, and no residual active no-PR behavior.
    Specific known updates include removing the patch-eligibility substring from
    `router_preserves_named_routing_branches`, conditioning
    `discuss_persists_pitch_draft_spec` so direct micro does not require
    `pitch.md`, reversing `micro_template_has_no_ac_verification_matrix`, and
    replacing no-PR assertions.
- Public docs / release notes:
  - Update `README.md` and `README.ja.md` together when user-facing workflow
    wording changes.
  - Add an Unreleased `CHANGELOG.md` entry for retiring patch/no-PR and
    redefining micro, unless implementation intentionally documents why this
    user-visible workflow change is excluded.

## Data Model / Interfaces

- No new metadata fields. `spec.yaml` remains version 1 with existing
  `type`, `surfaces`, `integration`, `risk`, and `status`.
- No JSON schema, `contracts.lock`, PR request schema, or `engine/VERSION`
  change.
- Micro shape is detected by file presence:
  - `spec.yaml` + `spec.md`
  - no `pitch.md`, `design.md`, or `tasks.md`
  - metadata/design-required checks prove it is standard-risk, single-surface,
    and integration-free.
- Direct micro commits use the existing branch prefix mapping from `git.md`
  (`feature` -> `feat`, other types as-is) and include the normal `Spec: {slug}`
  trailer. The first commit is a draft-spec commit owned by `plan`; the approval
  commit remains the existing plan approval commit.
- Direct micro metadata uses existing `spec.yaml` fields only. `plan` proposes
  `type` conservatively (`fix` for concrete bug fixes, `feature` only when the
  request adds behavior), chooses `surfaces` from configured surfaces and code
  context, and defaults `integration: none` / `risk: standard` only when micro
  eligibility is satisfied. If any metadata is ambiguous, `plan` asks for
  confirmation or routes to `discuss`.
- Existing pitch-only draft remains valid before expansion by discuss-created
  specs. Existing standard/design expanded draft remains valid with `pitch.md`.
- Pitchless micro ambiguity is intentional: `spec.md` + no `pitch.md` +
  standard-risk single-surface `integration: none` metadata is the micro signal.
  This can accept a malformed intended-standard draft, but rejecting it would
  require a new metadata field, which this spec explicitly avoids.

## Error Handling

- Deprecated patch trigger: report deprecation and stop at plan routing; do not
  edit files under patch semantics.
- Micro eligibility fails during plan: keep the same spec and add the required
  standard/design artifacts before approval.
- Direct-micro branch creation fails: stop before asking for approve-to-build;
  do not leave an uncommitted micro spec as the durable handoff.
- Direct-micro metadata cannot be derived safely or slug duplicates exist: ask
  for correction or route to `discuss`; do not create branch/spec files on
  guessed metadata.
- Micro eligibility fails during build/open: stop and route back to plan for
  re-approval rather than improvising missing durable artifacts.
- Dogfood sync omitted: `freeze --check`, adapter check, or doctor catches drift
  before completion.
- Engine doc rewrap breaks unrelated conformance substring tests: treat this as
  a verification failure and repair the test/doc wording deliberately. The active
  pitfall `2026-06-28-conformance-substring-line-wrap` applies here because this
  spec edits many engine Markdown files with literal substring assertions.

## Test Strategy

- Add or rewrite conformance tests for:
  - router/adapter guidance no longer exposing an active patch lane;
  - `mochiflow-patch` deprecation routing;
  - direct micro planning without pitch;
  - direct micro metadata/spec.yaml intake, duplicate slug checks, and branch
    prefix derivation;
  - direct micro branch creation/switch and initial draft commit responsibility;
  - lint acceptance/rejection for the three draft shapes;
  - micro template includes an AC Matrix by reversing the old
    `micro_template_has_no_ac_verification_matrix` assertion;
  - no active no-PR fast path text remains;
  - changed pitch prerequisites in `plan.md` and
    `discuss_persists_pitch_draft_spec`;
  - replacement of the patch branch assertion in
    `router_preserves_named_routing_branches`;
  - doctor command references no longer include `patch`;
  - no schema/lock/version change is part of the implementation.
- Because of pitfall `2026-06-28-conformance-substring-line-wrap`, run the full
  conformance suite after engine doc edits and keep any test-asserted phrase on
  a single unwrapped line or rewrite the assertion to match shorter stable
  substrings. Do not assume a nearby prose rewrap is harmless.
- Run the dogfood sequence after engine edits:
  - `mochiflow freeze`
  - `mochiflow upgrade --source engine`
  - `mochiflow adapter generate`
  - `mochiflow adapter generate --check`
- Run the surface `default` verification:
  - `cargo test --manifest-path cli/Cargo.toml`
  - `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`
  - `cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Run `mochiflow doctor`.
- Run `mochiflow lint --spec retire-patch-for-micro-spec-depth`.

## Review Results

### Pre-approval review 1

- Reviewer mode: delegated
- Verdict: fail

Finding: direct micro skipped `discuss`/`pitch.md` but did not define branch
creation/switch or first durable commit ownership. Fixed by adding branch and
draft-commit coverage.

### Pre-approval review 2

- Reviewer mode: delegated
- Verdict: pass

No findings after the branch/draft-commit fix.

### Pre-approval review 3

- Reviewer mode: delegated
- Verdict: fail

Findings: direct-micro intake still lacked metadata/slug/spec.yaml ownership,
and this Review Results section had stale structured state. Fixed by adding
metadata confirmation, duplicate checks, `spec.yaml` creation, and chronological
review entries. This draft should be re-reviewed before approval.

### Pre-approval review 4

- Reviewer mode: delegated
- Verdict: fail

Findings: final sync used only `mochiflow adapter generate --check` even though
adapter templates may change, and final verification omitted `mochiflow doctor`.
Fixed by adding write-mode `mochiflow adapter generate` before the check and
adding `mochiflow doctor` to AC-13, Test Strategy, and T-005. This draft should
be re-reviewed before approval.

### Build review 1

- Reviewer mode: delegated
- Verdict: pass-with-comments

Finding: direct micro draft commit guidance in `engine/commands/plan.md` appeared
before `spec.md` creation/refinement. Fixed by moving the direct micro draft
commit instruction after draft lint and before the approve-to-build readiness
card, then updating the conformance guard to pin that order.

### Build review 2

- Reviewer mode: delegated
- Verdict: pass-with-comments

Findings: AC Matrix evidence had not yet been recorded for AI-observed QA rows,
and `main...HEAD` still showed an unrelated backlog seed deletion. Fixed by
recording concrete AC/QA evidence in `spec.md` and restoring
`.mochiflow/specs/_backlog/ship-handoff-recovery-and-cleanup.md` from `main`.

### Build review 3

- Reviewer mode: delegated
- Verdict: pass-with-comments

Finding: T-005 was still unchecked and nested under T-004 in `tasks.md`. Fixed by
unindenting T-005 to the top-level task list and marking it complete after
confirming final verification remained current.
