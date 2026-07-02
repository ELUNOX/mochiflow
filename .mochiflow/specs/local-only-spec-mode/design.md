# Support local-only MochiFlow specs when .mochiflow is gitignored — Design

## Design Decisions

- Use Git ignore behavior as the v1 persistence-mode source of truth. This
  keeps mode selection aligned with the repository's existing policy and avoids
  a config value that can drift from `.gitignore`.
- Detect mode through one shared core helper, not separately in `accept` and
  `pr`. The helper should inspect the configured spec artifact path for the
  active slug and classify it as `tracked` or `local`.
- Treat local mode as a persistence change only. Acceptance quality gates still
  run: final verification, lint, AC Matrix completeness, and required review
  result remain mandatory before `status: accepted`.
- Keep tracked mode strict. The accepted close-out commit and `Spec:` trailer
  preflight remain the audit contract for repositories that track specs.
- Keep PR-body ownership in `open`. `mochiflow pr` remains a transport command
  that reads `--body-file`; engine guidance and templates ensure local-mode PR
  bodies carry the evidence that reviewers cannot inspect through committed spec
  files.
- For branch safety in local-mode PR preflight, require head to be ahead of
  base. This prevents local-only accepted specs from creating empty PR handoffs.
- Preserve delivery-state derivation in local mode by adding a non-trailer
  fallback: when provider state is unavailable and the local-mode spec branch
  tip is reachable from `origin/{base}`, the spec is considered delivered for
  status/index/close cleanup. Tracked mode keeps the existing provider-or-trailer
  signal.

## Architecture

- Add a core module for spec persistence classification, for example
  `spec_mode.rs`, exported from `lib.rs`.
- `accept.rs` resolves the target spec once, then asks the shared classifier for
  persistence mode before computing blockers and performing staging.
- In tracked mode, `accept.rs` uses the existing staging and close-out commit
  path.
- In local mode, `accept.rs` runs the same readiness and mutation steps, then
  returns success after lint without calling git staging or commit routines.
- `pr.rs` determines the slug-aware mode before accepted-spec preflight. Tracked
  mode calls the existing committed-closeout validation. Local mode calls a new
  local accepted-evidence validation and skips trailer validation.
- `pr.rs` extends agnostic branch checks so both modes reject `head == base` and
  head-not-ahead-of-base before push/dispatch. This is especially important in
  local mode because local spec artifacts are not committed evidence.
- `delivery.rs` / status / index logic learns the local-mode merge signal. For
  `provider = none`, an accepted local-mode spec with no `Spec:` trailer can
  still become Done/local-cleanup-pending when the source branch tip is an
  ancestor of `origin/{base_branch}`.
- Engine docs and templates describe how `open` branches behavior after the
  fold/context step: tracked mode commits the close-out; local mode records the
  local fold/evidence and makes the PR body carry the reviewable summary.

## Data Model / Interfaces

- Internal enum:
  - `SpecPersistenceMode::Tracked`
  - `SpecPersistenceMode::Local`
- Shared helper inputs:
  - repository root;
  - configured install/specs paths;
  - active spec directory or slug.
- Shared helper output:
  - mode enum;
  - human-readable reason for CLI output, such as the ignore rule/path that made
    the spec local-only when available.
- Local-mode accepted validation should reuse existing acceptance checks where
  possible:
  - `spec.yaml` status is `accepted`;
  - lint passes for the spec;
  - Matrix rows use done-eligible result tokens;
  - review result exists for `risk >= elevated`.
- Local-mode delivery derivation should use the canonical spec branch name
  derived from `spec.yaml` type and slug. If the branch tip is absent locally and
  remotely, provider state remains the only non-trailer merge signal and status
  should report the limitation rather than inventing Done.
- `mochiflow pr` command-line shape remains unchanged. The existing `--spec`,
  `--title`, `--body-file`, `--draft`, and `--dry-run` flags are enough.

## Error Handling

- If Git mode detection fails, fail closed with a clear CLI error rather than
  guessing tracked or local behavior.
- If a repository has mixed ignore state, prefer classifying by the concrete
  spec artifact path. Documentation should explain that selectively ignored
  specs are local mode even if other `.mochiflow/` files are tracked.
- In local mode, unrelated tracked working-tree dirt remains a hard failure.
  Ignored local spec artifacts are not a reason to force-add them.
- In tracked mode, missing committed accepted spec/trailer remains a preflight
  failure.
- In local mode, missing accepted state, incomplete evidence, missing review
  result, base/head equality, detached HEAD, or head-not-ahead-of-base are
  preflight failures before push or backend dispatch.
- If local-mode branch-tip reachability cannot be evaluated because the source
  branch no longer exists and the provider cannot report merge state, do not
  derive Done from absence. Keep the state conservative and document that
  cleanup should happen before deleting the branch, or that provider-backed
  projects should rely on provider merge state.

## Test Strategy

- Unit-test mode detection with tracked, root-ignored `.mochiflow/`, and
  specs-only ignored fixtures.
- Extend accept tests with a local-mode fixture proving final verification and
  local metadata update succeed without staging/committing ignored specs.
- Preserve tracked-mode accept regression tests that inspect staged paths and
  close-out commit trailers.
- Extend PR integration tests with local-mode accepted specs that are ignored
  and uncommitted, covering real run and `--dry-run`.
- Preserve tracked-mode PR regression tests for committed spec/trailer
  requirements.
- Add delivery/status/index regression coverage for provider-none local mode:
  branch pushed, branch tip merged into `origin/main`, no `Spec:` trailer, and
  status/index/close eligibility derive delivered/local-cleanup-pending.
- Add output assertions that local mode does not recommend `git add -f`.
- Add docs/template content assertions when practical; otherwise record
  reviewer-visible grep/manual evidence in the AC Matrix.
- Run full CLI verification and the dogfood engine sync checks after engine
  source changes.

## Integration Contract

- Owner: MochiFlow CLI core.
- Request: `accept` and `pr` ask for the persistence mode of the active spec
  before deciding whether to require committed spec artifacts.
- Response: a mode enum plus reason text.
- Error: detector errors stop the command before mutation, staging, push, or
  backend dispatch.
- Compatibility: tracked mode remains backward compatible; local mode is
  additive for repositories whose spec paths are ignored.
- Failure handling: local mode must never recover from ignored staging failures
  by suggesting forced tracking; tracked mode must never silently skip trailer
  validation.
- Verification: fixture repositories vary only their ignore/tracking policy so
  tracked and local assertions exercise the same command paths.
- Delivery signal: local mode adds branch-tip reachability as a manual-handoff
  fallback; tracked mode continues to use provider state or `Spec:` trailer
  reachability.

## Review Results

- Mandatory `change-reviewer` result will be recorded after implementation,
  because this spec is `risk: elevated`.
