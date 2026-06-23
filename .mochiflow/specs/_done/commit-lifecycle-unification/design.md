# Unify commit timing across discuss/plan/build/ship — Design

## Design Decisions

- **Branch ownership**: discuss owns branch creation. The branch name
  `{prefix}/{slug}` requires `type` from `spec.yaml`, and pitch-only lint
  requires all normal metadata keys, so discuss must write a complete
  lint-valid `spec.yaml` (`version`, `slug`, `title`, `type`, `surfaces`,
  `integration`, `risk`, `status`, `created`, `updated`) before committing.

- **pitch.md structure**: 7 sections derived from Shape Up pitch (Problem,
  Appetite, Solution, Rabbit Holes, No-gos) + RFC tradition (Alternatives
  Considered, Open Questions). No frontmatter — metadata lives in spec.yaml.

- **Handoff abolition**: ready-for-plan (`maturity: ready-for-plan`) is removed.
  Raw seeds (`maturity: seed`) remain as discuss input. The `discuss-handoff.md`
  template is deleted; `pitch.md` template replaces it. Router and conformance
  prose guards must be updated in the same change so the old handoff path is not
  still an accepted lifecycle.

- **Lint dual-mode validation**: lint checks file presence based on status:
  pitch-only `draft` requires only `spec.yaml` + `pitch.md` and skips
  `spec.md` / `design.md` / AC Matrix checks; draft with `spec.md` present runs
  the normal plan-time checks, including required `design.md`. `approved`/`done`
  require `spec.md`. This is a new branch in the existing lint logic.

- **Backlog validator contract**: `_backlog` is seed-only after this change.
  `mochiflow backlog validate` must reject `maturity: ready-for-plan` instead of
  accepting the old handoff shape, so CLI behavior matches router/workflow docs.

- **Commit type for spec-lane pre-build phases**: `docs` (Conventional Commits).
  Scope is `spec` (not the slug). Summary describes the action, not the slug.
  Trailers provide machine traceability.

## Architecture

Affected modules in `mochiflow-core`:

- `lint.rs`: add pitch.md presence check for `draft`; relax spec.md requirement
  for pitch-only `draft`; skip design / AC Matrix checks only while `spec.md` is
  absent; keep spec.md required for `approved`/`done`.
- `backlog.rs`: remove the `ready-for-plan` validation branch and update the
  maturity error text to seed-only.

No new modules. No new CLI commands. No new config keys.

## Integration Contract

- `discuss` is the only phase that creates `{specs_dir}/{slug}/spec.yaml` with
  `status: draft`, creates `pitch.md`, creates/switches to `{prefix}/{slug}`, and
  commits those discuss artifacts after pitch-only lint passes.
- `plan` requires an existing draft spec folder from discuss. It reads
  `pitch.md`, writes `spec.md` plus optional `design.md` / `tasks.md`, sets
  `status: approved` only after the human approval gate, and commits that plan
  artifact set on the existing branch.
- `build` requires the existing `{prefix}/{slug}` branch and fails before source
  edits if the branch cannot be found or switched to.
- `_backlog/{slug}.md` remains only a raw-seed inbox (`maturity: seed`). It is
  not a ready-for-plan persistence format after this change.

## Error Handling

- `mochiflow lint` on `draft` without `pitch.md`: emit error
  "pitch.md required for draft status".
- `mochiflow lint` on pitch-only `draft` with `risk: elevated` or
  `integration: workflow`: pass without `design.md` because plan has not
  expanded the spec yet.
- `mochiflow lint` on draft with `spec.md` and `risk: elevated` or
  `integration: workflow` but no `design.md`: fail with the existing
  design-required error.
- `mochiflow lint` on `approved` without `spec.md`: existing error remains.
- `mochiflow backlog validate` on `maturity: ready-for-plan`: fail with a
  seed-only maturity error.
- `mochiflow ready` on a spec without branch: not applicable (ready checks
  status, not branch existence; branch check is build's runtime responsibility).

## Test Strategy

- Existing Rust conformance tests in `cli/crates/mochiflow-cli/tests/conformance.rs`
  already pin engine prose and lint behavior. Update the ready-for-plan prose
  guards and add draft+pitch-only / draft-without-pitch / approved-without-spec
  / draft-with-spec-missing-design lint cases there.
- Update `backlog.rs` unit tests and CLI conformance coverage so
  `ready-for-plan` no longer validates as a backlog maturity.
- Engine doc/template changes are verified by `mochiflow freeze --check`,
  `mochiflow doctor`, `mochiflow adapter generate --check`, and
  `mochiflow index --check`.

## Review Results

- Reviewer mode: inline
- Verdict: pass
- Notes: Reviewed implementation against AC-01..AC-17 after full verification.
  No spec-conformance or code-quality findings.
