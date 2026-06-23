# Unify commit timing across discuss/plan/build/ship — Tasks

Implementation Summary: Update engine docs (router/discuss/plan/build/authoring/git/workflow/handoff), add pitch.md template, remove discuss-handoff template, update lint/backlog logic and conformance guards, freeze and sync.
risk: elevated
Critical Stop Conditions:
- Design decision not covered by discuss agreement surfaces
- Lint change breaks existing approved/done specs
- Engine doc change contradicts another doc not in scope

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && mochiflow lint --spec commit-lifecycle-unification && mochiflow freeze --check && mochiflow doctor && mochiflow adapter generate --check && mochiflow index --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-02] Create `engine/templates/spec/pitch.md` template
  - Depends on: none
  - Files: `engine/templates/spec/pitch.md`
  - Done: File exists with sections: Problem, Appetite, Solution, Rabbit Holes, No-gos, Alternatives Considered, Open Questions
  - Stop: Template structure conflicts with existing templates

- [x] T-002 [AC-08] Remove `engine/templates/backlog/discuss-handoff.md`
  - Depends on: none
  - Files: `engine/templates/backlog/discuss-handoff.md`
  - Done: File deleted
  - Stop: Other engine docs still reference this file (update those first)

- [x] T-003 [AC-09] Update `commands/discuss.md` — branch creation, commit, pitch.md output
  - Depends on: T-001
  - Files: `engine/commands/discuss.md`
  - Done: discuss.md `allowed_writes` includes `{specs_dir}/{slug}/**`; procedure writes lint-valid draft metadata + `pitch.md`, runs pitch-only `mochiflow lint --spec {slug}`, has branch creation + commit steps, removes `_backlog` handoff output, and references `templates/spec/pitch.md`
  - Stop: Procedure step ordering creates circular dependency with plan

- [x] T-004 [AC-10] Update `commands/plan.md` — read pitch.md, commit at approval, remove _backlog deletion
  - Depends on: T-003
  - Files: `engine/commands/plan.md`
  - Done: plan.md reads `pitch.md` as input alongside spec.yaml; has commit procedure at approval; no `_backlog` deletion step; prerequisites reference branch existence
  - Stop: plan procedure conflicts with existing downstream build expectations

- [x] T-005 [AC-11] Update `commands/build.md` — branch existence check replaces branch creation
  - Depends on: T-003
  - Files: `engine/commands/build.md`
  - Done: Step 2 verifies `{prefix}/{slug}` branch exists and switches to it; no branch creation; error-stop if branch missing
  - Stop: Standard-risk no-PR fast path logic conflicts with existence check

- [x] T-006 [P] [AC-12] Update `engine/router.md` — remove ready-for-plan plan routing
  - Depends on: none
  - Files: `engine/router.md`
  - Done: Router no longer lets `{slug} plan` resolve a `_backlog/{slug}.md` `maturity: ready-for-plan` handoff; raw seeds still route to discuss; existing spec folders still resolve normally
  - Stop: Routing rules become ambiguous for existing draft spec folders

- [x] T-007 [P] [AC-05] Update `reference/workflow.md` — draft set by discuss, backlog is raw seed only
  - Depends on: none
  - Files: `engine/reference/workflow.md`
  - Done: workflow.md states `status: draft` is set by discuss (not plan); Backlog seeds section removes ready-for-plan handoff lifecycle and documents `pitch.md` as the durable discuss artifact
  - Stop: Workflow text would contradict router behavior

- [x] T-008 [P] [AC-06] Update `reference/git.md` — commit lifecycle table
  - Depends on: none
  - Files: `engine/reference/git.md`
  - Done: git.md contains a commit lifecycle table showing discuss → plan → build → ship commit sequence on a single branch
  - Stop: none (additive text change)

- [x] T-009 [P] [AC-13] Update authoring and build handoff docs for `pitch.md`
  - Depends on: none
  - Files: `engine/reference/authoring.md`, `engine/templates/handoff/build-session-prompt.md`
  - Done: Durable artifacts table includes `pitch.md`; build handoff prompt tells new sessions to read `pitch.md` with the other spec artifacts
  - Stop: Handoff prompt becomes too adapter-specific

- [x] T-010 [AC-07] Update lint logic — pitch-only draft requires pitch.md, not spec.md
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/lint.rs`
  - Done: `status: draft` lint passes with only `spec.yaml` + `pitch.md` (no `spec.md` / `design.md`); draft with `spec.md` present still enforces required `design.md`; `status: approved`/`done` still requires `spec.md`; `draft` without `pitch.md` emits error
  - Stop: Change breaks existing conformance tests for approved/done specs

- [x] T-011 [AC-14] Update backlog validation to seed-only
  - Depends on: T-002
  - Files: `cli/crates/mochiflow-core/src/backlog.rs`, `cli/crates/mochiflow-core/src/index.rs`
  - Done: `validate_seed_text` accepts `maturity: seed`; rejects `maturity: ready-for-plan`; error text no longer lists ready-for-plan as an allowed maturity
  - Stop: Backlog index/list behavior depends on ready-for-plan as an active status

- [x] T-012 [AC-15] Update conformance guards for pitch/draft lifecycle and seed-only backlog
  - Depends on: T-001, T-002, T-006, T-010, T-011
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Ready-for-plan prose tests are replaced with assertions for discuss-created draft spec + `pitch.md`; lint tests cover draft+pitch-only pass, draft without `pitch.md` fail, draft with `spec.md` missing required `design.md` fail, and approved without `spec.md` fail; backlog validate rejects ready-for-plan
  - Stop: Existing helper forces `spec.md` creation and needs a larger test harness change

- [x] T-013 [AC-16] Regenerate MANIFEST.json and sync vendored engine
  - Depends on: T-001, T-002, T-003, T-004, T-005, T-006, T-007, T-008, T-009
  - Files: `engine/MANIFEST.json`, vendored engine output
  - Done: `mochiflow freeze --check` exits 0; `mochiflow doctor` exits 0
  - Stop: freeze fails due to missing or invalid engine file

- [x] T-014 [AC-17] Final verification — cargo test + lint + freeze + doctor + generated checks
  - Depends on: T-010, T-011, T-012, T-013
  - Files: none (verification only)
  - Done: `cargo test --manifest-path cli/Cargo.toml`, `mochiflow lint --spec commit-lifecycle-unification`, `mochiflow freeze --check`, `mochiflow doctor`, `mochiflow adapter generate --check`, and `mochiflow index --check` all exit 0
  - Stop: Test failures unrelated to this spec

- [x] T-015 [AC-01, AC-03, AC-04] Content verification of discuss/plan/build docs
  - Depends on: T-003, T-004, T-005
  - Files: none (verification only)
  - Done: discuss.md contains branch creation + pitch-only lint + commit; plan.md contains commit at approval; build.md has existence check without creation
  - Stop: none (read-only check)
