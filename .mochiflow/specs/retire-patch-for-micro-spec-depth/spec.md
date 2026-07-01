# Retire patch and redefine micro as the fastest spec depth

## Background and Design Rationale

MochiFlow currently has two ways to make small changes. The spec lane creates
durable artifacts, an AC Matrix, acceptance state, and PR delivery. The `patch`
lane skips all of that: no `spec.yaml`, no AC Matrix, no branch/PR contract, and
no accepted state. The no-PR fast path creates the same kind of fork from inside
the spec lane by skipping the approve-PR gate and PR handoff.

This spec retires that split and corrects the partial micro definition that
already exists in the engine. Today `workflow.md` already has a "Micro spec" row
defined as `pitch.md` + `spec.md`; `plan.md` references `spec.micro.md`;
`risk.md`, `git.md`, and `build.md` also already mention micro. The work is
therefore a redefinition, not a brand-new depth: micro becomes the smallest
tracked spec shape with `spec.yaml` and `spec.md` only, while the old
`pitch.md + spec.md` micro row is replaced. `risk` continues to be the stored
review/gate axis. Because current risk rules require `design.md` for elevated or
critical work, micro is eligible only when the work stays standard-risk,
single-surface, integration-free, and does not need durable ADR rationale or
human QA.

The design deliberately avoids a `depth` field, a new `micro` command, schema
changes, and a permanent `mochiflow-patch` alias. `mochiflow-patch` is handled
only as a deprecated trigger with a short notice that routes the user toward
planning a spec. Existing specs and historical patch commits are not rewritten.

Current-state checks against code confirm the main affected surfaces:
`engine/router.md`, `engine/commands/patch.md`, `engine/reference/workflow.md`,
`engine/reference/git.md`, `engine/commands/plan.md`,
`engine/commands/build.md`, `engine/commands/open.md`,
`engine/commands/update.md`, adapter templates, `README.md`, `doctor.rs`,
`lint.rs`, and conformance tests still encode patch and/or no-PR behavior.
Decision `2026-06-23-commit-lifecycle-unification` and pitfall
`2026-06-23-draft-two-valid-shapes-lint-branch` explain the existing
pitch-only/expanded draft split; this spec extends that model to include a
micro draft shape.

## User Story

As a MochiFlow user making a small concrete fix, I want the fastest path to keep
the same traceability and PR delivery model as every other spec, so that small
work remains auditable without carrying the full standard/design spec overhead.

## Scope

- In:
  - Retire `patch` as a non-spec lane in the router, engine command catalog,
    adapter entrypoints, workflow/git references, public docs, doctor checks,
    and conformance tests.
  - Recognize `mochiflow-patch` only as a deprecated trigger that gives a
    one-line notice and routes to `plan`; do not keep a full compatibility lane.
  - Redefine the existing `micro` depth as the fastest spec depth with
    `spec.yaml` and `spec.md` only, inferred from file presence and eligible only
    for standard-risk single-surface work with no integration, design-required
    impact, human QA, or ADR fold need.
  - Let `plan` create micro specs directly from an explicit concrete request;
    `discuss` and `pitch.md` remain optional for micro and required for
    standard-or-larger planning.
  - Define direct-micro intake: when no draft spec already exists, `plan`
    derives a proposed slug/title/type/surface/risk/integration from the
    concrete request and current config/code, asks the user to confirm or edit
    that metadata, checks duplicate active/_done/backlog slugs, then writes
    `spec.yaml` from the template.
  - When `plan` creates a micro spec directly, make `plan` responsible for the
    same branch durability normally established by `discuss`: create/switch the
    `{prefix}/{slug}` feature branch from `origin/{base_branch}` and commit the
    draft micro artifacts before asking for approve-to-build.
  - Update lint so draft validation supports three shapes: pitch-only draft,
    expanded draft with pitch, and pitchless micro draft. Because no `depth`
    metadata is added, a standard-risk single-surface pitchless draft with
    `spec.md` is treated as micro; lint cannot prove the author intended a
    larger spec and forgot `pitch.md`.
  - Remove the no-PR fast path from workflow/git/build/open guidance and
    conformance tests; every spec depth delivers through the normal feature
    branch plus PR path.
  - Update `spec.micro.md` so micro carries the minimal spec contract and an AC
    Matrix placeholder in `spec.md`.
  - Re-freeze and dogfood-sync engine source after editing repo-root `engine/`.
- Out:
  - Adding a `depth` field to `spec.yaml`, changing JSON schemas, changing
    `contracts.lock`, or bumping `engine/VERSION`.
  - Adding a new `micro` command or new public CLI subcommand.
  - Retrofitting already-authored specs or rewriting historical patch commits.
  - Changing `mochiflow pr` backend behavior or the PR request schema.
  - Editing the vendored `.mochiflow/engine/` copy directly; it is regenerated
    by `mochiflow upgrade --source engine`.

## Edge Cases

- A bare "fix this" / "直して" with no active spec and no explicit plan intent:
  the router proposes planning a spec and waits; it does not auto-activate a
  micro spec.
- `mochiflow-patch` is entered explicitly: the router reports that patch is
  deprecated and routes toward `plan`; it does not run a non-spec edit lane.
- A micro candidate discovers durable rationale, a pitfall, integration,
  elevated/critical risk, public contract impact, or human QA need: the spec
  escalates in place to standard/design depth before approval.
- A draft spec has `spec.md` but no `pitch.md`: lint accepts it only when it
  satisfies the micro shape; expanded standard/design drafts still require
  `pitch.md`.
- A micro spec reaches delivery: `open` still uses acceptance, approve-PR, and
  `mochiflow pr`; there is no no-PR exception.
- A micro spec is created directly by `plan`: `plan` performs the branch
  creation/switch and initial draft commit that `discuss` normally performs, so
  the artifacts are durable before the approval gate.
- Direct micro intake cannot derive safe metadata: `plan` asks one concise
  clarification or routes to `discuss`; it does not guess slug/type/surface/risk
  when that would affect branch naming or eligibility.
- Existing active or archived specs authored under the old model remain valid;
  only new routing and lint behavior changes.
- A standard-risk single-surface draft has `spec.md` but no `pitch.md`: lint
  treats it as micro when the metadata and file set satisfy micro eligibility.
  This is the explicit trade-off for avoiding a `depth` field; an intended
  standard/design spec missing `pitch.md` is not mechanically distinguishable
  until it adds design-required files or metadata.

## Acceptance Criteria (EARS)

- AC-01: THE router and generated adapter guidance SHALL remove `patch` as a
  non-phase lane and SHALL route concrete small-fix intent with no active spec
  toward `plan` rather than `patch`.
- AC-02: WHEN the explicit deprecated token `mochiflow-patch` is used, THE
  router SHALL produce a one-line deprecation notice and route toward `plan`
  without running a non-spec patch procedure or preserving a permanent alias.
- AC-03: THE plan workflow SHALL define `micro` as the smallest spec depth,
  inferred from artifacts, with `spec.yaml` plus `spec.md` only; it SHALL update
  the existing `workflow.md` Depth scaling Micro row from `pitch.md + spec.md`
  to `spec.yaml + spec.md`, and SHALL allow plan to create a micro spec directly
  from an explicit concrete request without requiring `pitch.md` or a prior
  discuss phase.
- AC-04: WHEN plan creates a micro spec directly without a prior discuss phase,
  THE SYSTEM SHALL derive or collect the required `spec.yaml` metadata
  (`slug`, `title`, `type`, `surfaces`, `integration`, `risk`) from the explicit
  concrete request, current config, and code context; SHALL present the proposed
  metadata for user confirmation or correction; SHALL reject duplicate
  active/_done/backlog slugs; and SHALL write `spec.yaml` from the template
  before branch creation.
- AC-05: WHEN plan creates a micro spec directly after metadata confirmation,
  THE SYSTEM SHALL create or switch to the `{prefix}/{slug}` feature branch from
  `origin/{base_branch}` and commit the draft micro artifacts (`spec.yaml` and
  `spec.md`) before presenting the approve-to-build gate.
- AC-06: WHERE work is standard-risk, single-surface, `integration: none`, and
  has no design-required impact, human QA, or ADR fold need, THE SYSTEM SHALL
  allow the micro shape; IF implementation discovers durable rationale, a
  pitfall, integration, elevated/critical risk, public contract impact, or human
  QA need, THEN THE SYSTEM SHALL escalate the spec in place before approval or
  delivery.
- AC-07: THE lint rules SHALL accept exactly the intended draft shapes:
  pitch-only draft, expanded draft with `pitch.md`, and pitchless micro draft
  when metadata/file presence satisfies micro eligibility; they SHALL reject
  pitchless expanded drafts that fail micro eligibility and approved specs
  missing `spec.md` or required AC Matrix rows. The lint contract SHALL document
  that without a `depth` field it cannot distinguish an intended standard draft
  that forgot `pitch.md` from a valid pitchless micro draft when both have the
  same eligible metadata and file set.
- AC-08: THE micro spec template and micro plan guidance SHALL keep the
  traceability model by including acceptance criteria and a `spec.md` AC Matrix
  suitable for build/open results, and the existing
  `micro_template_has_no_ac_verification_matrix` conformance test SHALL be
  reversed to assert that the micro template includes the Matrix.
- AC-09: THE workflow, git, build, open, update, README, README.ja.md,
  CHANGELOG.md, and conformance
  contracts SHALL remove the no-PR fast path; every spec depth SHALL deliver by
  the normal feature branch plus PR path and the approve-PR gate SHALL always
  apply.
- AC-10: THE engine command catalog, doctor command-reference checks, manifest,
  and adapter self-heal lists SHALL stop treating `patch` as an active command
  while still cleaning up deprecated generated Kiro patch files.
- AC-11: THE implementation SHALL NOT change `spec.yaml` schema, JSON schema
  files, `contracts.lock`, or `engine/VERSION`.
- AC-12: THE conformance suite SHALL pin the new routing/deprecation, micro
  draft lint, micro template Matrix, removed no-PR path, and removed patch lane
  behavior, including replacement of the old
  `router_preserves_named_routing_branches` patch-eligibility assertion and the
  pitch-prerequisite assertions in `discuss_persists_pitch_draft_spec`.
- AC-13: THE edited engine SHALL be synced through the dogfood sequence
  (`mochiflow freeze` -> `mochiflow upgrade --source engine` ->
  `mochiflow adapter generate` -> `mochiflow adapter generate --check`) and
  SHALL pass the surface `default` verification, `mochiflow doctor`, and
  `mochiflow lint --spec retire-patch-for-micro-spec-depth`.

## QA Scenarios

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P7 | cli | Human-operated | Ask for a concrete small fix with no active spec, then use the explicit `mochiflow-patch` token. | Small-fix wording proposes planning; `mochiflow-patch` reports deprecation and routes toward plan. |
| QA-02 | P2 | cli | Human-operated | Create a micro spec from an explicit concrete request and continue through plan approval by label, keyword, and number. | The micro spec has `spec.yaml` + `spec.md`, no `pitch.md`/`design.md`/`tasks.md`; approval still follows the approve-to-build gate. |
| QA-03 | P3 | cli | Automated | Run conformance tests that assert `patch` is not an active command and no no-PR fast path text remains. | Deprecated token behavior is narrow; no active patch/no-PR route can be selected. |
| QA-04 | P4 | cli | Automated | Run lint fixtures for pitch-only draft, micro draft without pitch, non-micro expanded draft without pitch, approved micro with Matrix, and accepted result tokens. | Only the intended file shapes pass; traceability checks remain enforced. |
| QA-05 | P5 | cli | Human-operated | Inspect old active specs and archived `_done/` specs after the change. | Existing specs are not rewritten or migrated; historical artifacts remain readable. |
| QA-06 | P6 | cli | Automated | Run the dogfood sequence, adapter check, surface `default` verification, and lint for this spec. | Engine manifest, vendored engine, adapters, conformance tests, fmt, clippy, freeze check, and spec lint all pass. |
| QA-07 | P7 | cli | Human-operated | Compare edited engine docs, adapter outputs, README, lint behavior, and conformance tests against AC-01 through AC-13. | The documented workflow matches the spec; there are no residual active patch or no-PR instructions. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- Engine source edits are frozen and synced; generated adapter outputs are
  checked.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-01, QA-03, QA-07 | `engine/router.md`, adapter templates, generated adapter outputs, conformance tests | UNVERIFIED |  | Small fixes route toward plan, not patch |
| AC-02 | cli | automated | QA-01, QA-03 | `engine/router.md`, conformance tests | UNVERIFIED |  | Deprecated token notice only |
| AC-03 | cli | automated | QA-02, QA-04 | `engine/commands/plan.md`, `engine/reference/workflow.md`, `engine/templates/spec/spec.micro.md`, lint tests | UNVERIFIED |  | Direct micro planning without pitch |
| AC-04 | cli | automated | QA-02, QA-06 | `engine/commands/plan.md`, `engine/reference/git.md`, conformance tests | UNVERIFIED |  | Direct micro intake creates confirmed metadata/spec.yaml |
| AC-05 | cli | automated | QA-02, QA-06 | `engine/commands/plan.md`, `engine/reference/git.md`, conformance tests | UNVERIFIED |  | Direct micro plan creates/switches branch and commits draft artifacts |
| AC-06 | cli | AI-observed | QA-02, QA-07 | `engine/commands/plan.md`, `engine/reference/risk.md`, `engine/reference/git.md` | UNVERIFIED |  | Escalation criteria documented |
| AC-07 | cli | automated | QA-04 | `cli/crates/mochiflow-core/src/lint.rs`, conformance tests | UNVERIFIED |  | Three draft shapes plus documented ambiguity trade-off |
| AC-08 | cli | automated | QA-04, QA-07 | `engine/templates/spec/spec.micro.md`, `engine/commands/plan.md`, conformance tests | UNVERIFIED |  | Micro keeps AC Matrix traceability |
| AC-09 | cli | automated | QA-03, QA-06, QA-07 | `engine/reference/workflow.md`, `engine/reference/git.md`, `engine/commands/build.md`, `engine/commands/open.md`, `README.md`, `README.ja.md`, `CHANGELOG.md`, conformance tests | UNVERIFIED |  | No no-PR fast path remains |
| AC-10 | cli | automated | QA-03, QA-06 | `cli/crates/mochiflow-core/src/doctor.rs`, `cli/crates/mochiflow-core/src/adapter.rs`, `engine/MANIFEST.json`, conformance tests | UNVERIFIED |  | Patch removed from active command catalog |
| AC-11 | cli | automated | QA-06 | `contracts/`, `engine/VERSION`, implementation diff | UNVERIFIED |  | No schema, lock, or version change |
| AC-12 | cli | automated | QA-03, QA-04, QA-06 | `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED |  | New contract is pinned |
| AC-13 | cli | automated | QA-06 | `engine/MANIFEST.json`, `.mochiflow/engine/`, generated adapters, `cli/` | UNVERIFIED |  | Dogfood sync, doctor, and default verification |
