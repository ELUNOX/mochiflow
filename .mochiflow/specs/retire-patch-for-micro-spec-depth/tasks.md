# Retire patch and redefine micro as the fastest spec depth - Tasks

Implementation Summary: Replace the non-spec patch lane and no-PR fast path with a micro spec depth inside the normal spec-to-PR lifecycle.
risk: elevated
Critical Stop Conditions:
- Do not add a `depth` field, schema change, `contracts.lock` change, or `engine/VERSION` bump.
- Do not preserve `patch` as an active compatibility lane or keep a no-PR delivery path.
- Do not edit `.mochiflow/engine/` directly; update it only through the dogfood sync.
- Do not leave the existing `workflow.md` Micro row (`pitch.md + spec.md`) or old micro conformance tests contradicting the new pitchless micro contract.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Engine-editing tasks must run `mochiflow freeze` before their verification commit so `engine/MANIFEST.json` matches repo-root `engine/`.
- Final sync runs `mochiflow upgrade --source engine`, then write-mode `mochiflow adapter generate`, then `mochiflow adapter generate --check`; generated adapter outputs may change through the template regeneration path.
- Conformance substring pitfall: when editing engine Markdown, keep asserted phrases unwrapped or update the assertion; run the full conformance suite because unrelated substring guards can fail after nearby prose rewraps.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-10] Retire active patch routing and command catalog
  - Depends on: none
  - Files:
    - `engine/router.md`
    - deleted: `engine/commands/patch.md`
    - `engine/README.md`
    - `engine/adapters/agents/AGENTS.md.tpl`
    - `engine/adapters/claude-code/CLAUDE.md.tpl`
    - `engine/adapters/copilot/copilot-instructions.md.tpl`
    - `engine/adapters/kiro/steering/mochiflow.md.tpl`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-core/src/adapter.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: Router no longer presents patch as a non-phase lane, load-on-demand command, or eligibility branch; concrete small-fix/no-spec wording without an active spec proposes `plan` and waits; `mochiflow-patch` is documented only as a deprecated token that emits a one-line notice and routes toward plan. Adapter templates and engine README no longer list `commands/patch.md`; `doctor.rs` no longer treats `patch` as an active workflow command; `adapter.rs` still cleans old generated Kiro `spec-patch.md` residue without generating or documenting active patch behavior. Update `router_preserves_named_routing_branches` by removing/replacing the old `commands/patch.md ## Eligibility` asserted substring, and add the new deprecated-token/plan-route assertions. Shared files remain consistent for later tasks: router wording should not define micro details owned by T-002/T-003, and conformance changes should leave room for T-005's final residual checks.
  - Stop: if preserving `engine/commands/patch.md` is required for routing mechanics, stop and redesign it as a non-active deprecation artifact without patch procedure semantics before continuing.
- [ ] T-002 [AC-03, AC-04, AC-05, AC-06, AC-08] Redefine existing micro depth in workflow, plan, authoring, risk, git, and templates
  - Depends on: T-001
  - Files:
    - `engine/reference/workflow.md`
    - `engine/reference/authoring.md`
    - `engine/reference/git.md`
    - `engine/reference/risk.md`
    - `engine/commands/plan.md`
    - `engine/commands/discuss.md`
    - `engine/templates/spec/spec.micro.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: The depth ladder starts at micro, not patch; the existing `workflow.md` Depth scaling Micro row is updated from `pitch.md + spec.md` to `spec.yaml + spec.md`; micro is inferred by file presence and eligible only for standard-risk single-surface `integration: none` work with no design-required impact, human QA, or ADR fold need. `plan.md` can create a micro spec directly from an explicit concrete request without `pitch.md`; standard/design planning still reads `pitch.md` from discuss, so `plan.md` frontmatter prerequisites and procedure are conditional instead of globally requiring `pitch.md`. For direct micro, `plan.md` derives proposed `slug`, `title`, `type`, `surfaces`, `integration`, and `risk` from request/config/code, presents them for user confirmation or correction, checks duplicate active/_done/backlog slugs, writes `spec.yaml` from the template, prepares `{prefix}/{slug}` from `origin/{base_branch}` using the `git.md` branch rules, and creates the initial draft micro `docs(spec): ...` commit with a `Spec:` trailer before presenting approve-to-build; the normal plan approval commit remains separate. `discuss.md` remains optional for micro and the right path for ambiguity. `spec.micro.md` contains EARS AC guidance and a minimal `## Verification Plan / AC Matrix` in `spec.md`; reverse `micro_template_has_no_ac_verification_matrix` so it asserts the Matrix is present. Update `discuss_persists_pitch_draft_spec` so plan's pitch requirement is explicitly standard-or-larger/direct-micro-exempt. Conformance tests pin metadata intake, duplicate checks, branch/draft-commit ownership, micro template, and plan/direct-micro contract. Shared files remain consistent for T-003: lint-facing terms for micro shape must match the exact file-presence rules implemented later.
  - Stop: if micro eligibility needs any stored metadata or schema change, stop and revise the plan because that violates AC-11.
- [ ] T-003 [AC-07] Implement lint support for micro and preserve expanded-draft guards
  - Depends on: T-002
  - Files:
    - `cli/crates/mochiflow-core/src/lint.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: Lint accepts pitch-only draft (`spec.yaml` + `pitch.md`), expanded draft with `spec.md` and `pitch.md`, and pitchless micro draft with `spec.md` and no `pitch.md` when metadata and file presence satisfy micro eligibility. Lint rejects pitchless drafts that fail micro eligibility, approved specs without `spec.md`, approved specs without Matrix coverage, and accepted specs with non-done-eligible Matrix results. The implementation and tests explicitly document the trade-off that a standard-risk single-surface pitchless draft with `spec.md` is treated as micro because no file/metadata signal can distinguish a forgotten `pitch.md` without adding `depth`.
  - Stop: if lint cannot distinguish micro from a malformed expanded draft using file presence plus existing metadata, stop and revisit the no-`depth` decision before coding around it.
- [ ] T-004 [AC-06, AC-09] Remove no-PR delivery and align build/open/update/git guidance
  - Depends on: T-002
  - Files:
    - `engine/reference/workflow.md`
    - `engine/reference/git.md`
    - `engine/commands/build.md`
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `README.md`
    - `README.ja.md`
    - `CHANGELOG.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
  - Done: Workflow/git/build/open/update no longer describe a no-PR fast path, patch verification, patch commits, or routing unrelated PR feedback to patch. Every spec depth, including micro, ends build at `approved`, runs acceptance/open, presents approve-PR, and uses `mochiflow pr`. `git.md` retains taskless/micro build commit rules and normal feature-branch PR mechanics. README.md and README.ja.md describe micro/small specs instead of small patches, and CHANGELOG.md has an Unreleased note for the user-visible workflow change. Conformance tests that previously pinned no-PR are replaced with negative and positive assertions for the normal PR path. Shared files remain consistent with T-001's router removal and T-005's residual grep checks.
  - Stop: if a user-requested no-PR exception appears necessary for micro, stop because that contradicts the accepted pitch and AC-09.
- [ ] T-005 [AC-10, AC-11, AC-12, AC-13] Final conformance, dogfood sync, and verification
  - Depends on: T-001, T-002, T-003, T-004
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/MANIFEST.json`
    - `.mochiflow/engine/`
    - generated adapter outputs
  - Done: Conformance includes residual guards that no active patch lane, active `commands/patch.md` reference, patch command catalog entry, or no-PR fast path remains, while allowing deprecation wording and Kiro cleanup references where intentional. Run the full conformance test suite specifically to catch unrelated literal-substring failures from engine Markdown rewraps. Confirm `contracts/`, `contracts/contracts.lock`, and `engine/VERSION` are unchanged by this spec. Run `mochiflow freeze`, `mochiflow upgrade --source engine`, write-mode `mochiflow adapter generate`, `mochiflow adapter generate --check`, the surface `default` verification, `mochiflow doctor`, and `mochiflow lint --spec retire-patch-for-micro-spec-depth`; record AC Matrix evidence after build. If adapter generation updates tracked entrypoints, include those generated outputs in the appropriate verification commit and keep `.mochiflow/engine/` changes generated only.
  - Stop: if residual search finds active patch/no-PR wording that is not a deliberate deprecation or cleanup reference, fix the contract before final verification.
