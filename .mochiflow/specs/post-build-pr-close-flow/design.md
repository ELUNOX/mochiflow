# Post-build PR and Close Flow — Design

## Design Decisions

- **Terminal state is observed, not written.** `merged` (and `in_review`) are
  derived from VCS/provider at read time, never persisted. This removes every
  post-merge write to the base branch and eliminates the archive-before-PR and
  resurrection problems at the root. Rationale and rejected alternatives live in
  `pitch.md`.
- **`spec.yaml` stores only asserted states** `draft → approved → accepted`. The
  status enum gains `accepted` and drops `done` as a value the engine *writes*.
  `done` remains a *readable* legacy value so archived `_done/` specs stay valid.
- **`accepted` is a quality state, not a new human gate.** It is set by the
  deterministic close-out (renamed from `ship` to `accept`) once acceptance
  conditions hold (AC Matrix all done-eligible; reviewer verdict when
  `risk ≥ elevated`). The two delivery approval gates are unchanged
  (approve-to-build, approve-PR).
- **Specs never move.** The `_done/` move, the `done` write, and the committed
  `INDEX.md` regeneration are removed from the close-out. A spec stays at
  `{specs_dir}/{slug}/` for life.
- **The board is computed.** `mochiflow status` is the source of truth for the
  kanban; `INDEX.md` is a regenerable, gitignored cache. No git hooks; freshness
  comes from per-command regeneration plus on-demand `status`.
- **`open`/`update`/`close` are engine command procedures** (agent-run inline,
  like discuss/plan/build), backed by deterministic CLI helpers (`accept`, `pr`,
  `status`). They are not new clap verbs beyond `status`.
- **Derivation degrades gracefully** so `provider = none` and offline never hard
  fail; the board reflects last-known local state and `--fetch` refreshes.

## Architecture

CLI (`cli/crates/mochiflow-core`, dispatched from `mochiflow-cli/src/main.rs`):

- New module `delivery.rs` (working name): pure derivation of delivery state for
  a slug — `in_review` and `merged` — resolving the signal priority
  (provider API → `Spec:` trailer ancestry in `origin/{base_branch}`; two signals
  only, no human-report signal) and the `provider = none` local-git path. Reuses
  git-invocation patterns from `ship.rs`/`pr.rs` (commit-trailer grep, `git
  merge-base --is-ancestor`, and branch/push state).
- Delivery-state precedence (highest first): **Done > In Review > Ready >
  Active > Backlog**, each spec resolving to exactly one column:
  - Done: provider reports merged, or (any provider, incl. `none`) a
    `Spec: {slug}` trailer is reachable from `origin/{base_branch}`.
  - In Review: not Done, and either the provider reports an open PR, or
    (`provider = none`) the spec branch is pushed to `origin` and unmerged.
  - Ready: `status: accepted`, not Done, not In Review (e.g. accepted-unpushed).
  - Active: `status` `draft` or `approved`. Backlog: a `_backlog/` seed.
  Conflicting signals resolve by precedence (merged outranks in_review). When the
  provider is unavailable, fall back to the local-git signals and never error
  (a row may read In Review/Ready until git/provider confirms).
- New module `status.rs` (working name) + `Status` clap subcommand: computes the
  board (Backlog from `_backlog/` seeds; Active = `draft`/`approved`; Ready =
  `accepted` with no open PR; In Review = derived `in_review`; Done = derived
  `merged` ∪ legacy `_done/`) and renders text; `--fetch` triggers a pre-compute
  `git fetch`.
- `index.rs`: `render_index`/`collect` reworked to render columns from the same
  board computation (asserted ∪ derived), not from directory location, and
  `status_emoji`/pipeline counts gain `accepted`. `INDEX.md` stays a generated
  artifact but is gitignored; the per-command regeneration is wired through a
  shared post-command step in `main.rs` (state-changing commands only, never
  `status`). `init.rs` `write_install_gitignore` must also ignore the configured
  index filename so new installs never track `INDEX.md`, and the already-tracked
  `INDEX.md` is untracked once via a manual `git rm --cached` (existing user
  repos do the same one-time manual untrack; automated `join`/`upgrade` migration
  is out of scope). Spec-lane procedure commit steps (discuss/plan/build/open/update/
  close) invoke `mochiflow index` to keep the board fresh between CLI commands.
- `ship.rs` → repurposed to `accept` mechanics: run final verification, settle
  automated AC Matrix rows, set `status: accepted` (+ `updated`), `lint`, stage
  `{specs_dir}/{slug}/**` and ADR paths (NOT `INDEX.md`, NOT a `_done/` move),
  and create the feature-branch close-out commit. Remove `update_spec_yaml`'s
  `done`/`completed` write, the `_done/` `fs::rename`, and the
  `index::generate_index_quiet` call from the close-out path.
- `pr.rs`: replace `validate_pr_spec_closeout_committed` (which checks `_done/` +
  `done`) with a check that the active `{specs_dir}/{slug}/` is committed with
  status `accepted` and a `Spec: {slug}` trailer present.
- `lint.rs`: accept `accepted`; keep `done` valid for archived specs; move the
  matrix/coverage gate to apply at `accepted` (the conditions previously gated by
  `done`).
- `doctor.rs`: update `terminal_cli_command_references` (add `status`, the
  command list now lacks `ship` and gains `accept`) and
  `workflow_command_references` (verb vocabulary) so the allowlist test and
  `doctor` pass.

Engine docs — **the source of truth is the repo-root `engine/`, not the vendored
`.mochiflow/engine/` dogfood copy** (the CLI embeds `engine/` at build, and
`.mochiflow/engine/` is regenerated from it). Rewrite `engine/commands/ship.md`
into `engine/commands/open.md` + `update.md` + `close.md`; update
`engine/router.md`, `engine/commands/build.md`, `engine/commands/discuss.md`,
`engine/reference/{workflow,git,authoring}.md`, templates
(`engine/templates/delivery/pr-description.md`,
`engine/templates/handoff/build-session-prompt.md`), and **all** adapter
templates (`engine/adapters/{kiro,agents,claude-code,copilot}/...`). After
editing, run `mochiflow freeze` (regenerate `engine/MANIFEST.json`,
`contracts/contracts.lock`, and the version gate), `mochiflow upgrade --source
engine` (re-vendor into `.mochiflow/engine/`), and `mochiflow adapter generate
--check` (confirm adapter outputs are in sync).

## Data Model / Interfaces

- `spec.yaml` `status`: allowed *writable* values `draft | approved | accepted`;
  `done` accepted on read for legacy archived specs only. `completed` is no
  longer written by the engine (legacy reads still honored for Done ordering).
- Contract surface for the new status (all must move together): the
  `contracts/spec.schema.json` `status` enum gains `accepted` and keeps `done`
  for legacy archived specs; the contracts version gate is bumped and
  `contracts/contracts.lock` regenerated via `mochiflow freeze`; the `lint.rs`
  `allowed_statuses` list and the gate-at-`done` branches move to `accepted`;
  the conformance fixtures (`tests/conformance.rs` `GOOD_YAML` and the
  `status: done` cases) are updated, retaining one legacy-`done` acceptance case.
- Config: no new required keys. Derivation reads existing `[git].provider` and
  `[git].base_branch`. (`provider = none` is already the default.)
- `mochiflow status` exit code: `0` on success regardless of derivation
  freshness; non-zero only on a genuine internal error.
- `INDEX.md`: gitignored; content remains a generated dashboard but columns are
  derived. Format details are an implementation concern, not a contract.

## Error Handling

- Derivation failures (no remote, provider CLI missing, detached HEAD) are
  non-fatal: log a concise note and fall back to local/last-known state; never
  abort `status`/`index`/state-changing commands on derivation error.
- `accept` retains today's strict pre-flight (clean tree except the spec's own
  paths; readiness blockers) but the readiness now targets `approved → accepted`
  and the acceptance conditions, not `done`.
- `pr` pre-flight failure (spec not `accepted`/not committed/no trailer) returns
  the existing pre-flight FAIL exit code (`3`) with an actionable message.

## Test Strategy

- Unit: derivation module (trailer ancestry, provider-none path, in_review vs
  merged), `status` board placement, `accept` close-out (no done/_done/INDEX
  writes), `pr` pre-flight against `accepted`.
- Conformance (`tests/conformance.rs`): update the spec.yaml status fixtures
  (`GOOD_YAML`, `status: done` cases) for `accepted` while keeping one legacy
  `done` acceptance case; assert engine docs/adapters contain no user-facing
  `ship` verb and do define `open`/`update`/`close`; assert `INDEX.md` is
  gitignored; assert the command allowlist (`status` in, `ship` out); and run
  `freeze --check` + `adapter generate --check` for derived-file integrity.
- Regression: existing `cli.rs`/`pr.rs`/`first_run.rs` suites stay green;
  archived `_done/` specs still lint/render.
- The full `default` verification (`cargo test`, `fmt`, `clippy -D warnings`,
  `freeze --check`) is the build-completion gate.

## Workstreams

| Workstream | Surface | Responsibility | Depends on | Verification |
| --- | --- | --- | --- | --- |
| WS1 state+contract | cli | `accepted` across the contract surface (schema enum, version gate, `contracts.lock`, `lint` gate move, conformance fixtures); legacy `done` read compat | none | `cargo test` (lint, conformance), `freeze --check` |
| WS2 derivation | cli | derive `in_review`/`merged` incl. `provider=none` | WS1 | `cargo test` (delivery) |
| WS3 board | cli | `mochiflow status`, INDEX gitignore + derived columns + auto-regen | WS2 | `cargo test` (status, index) |
| WS4 close-out+pr | cli | repurpose `ship`→`accept` (no done/_done/INDEX), `pr` pre-flight | WS1 | `cargo test` (ship/accept, pr) |
| WS5 engine docs (SoT) | cli | open/update/close docs + router/build/discuss/workflow/git/authoring + all `engine/adapters/*` + templates in repo-root `engine/`, then `freeze` + `upgrade --source engine` + `adapter generate --check` | WS1-WS4 | `freeze --check`, `adapter generate --check`, conformance |
| WS6 guard+doctor | cli | stale-base guard doc, doctor allowlist + conformance | WS3, WS5 | `cargo test` (doctor allowlist), conformance |

## Integration Contract

- **Contract owner:** the MochiFlow lifecycle (engine docs) + the CLI command
  surface. This is a `workflow` integration: the contract is the verb vocabulary,
  the asserted-vs-derived state model, and CLI exit codes.
- **Request/Response:** `mochiflow status` prints the board (exit 0). The accept
  close-out sets `accepted` (exit 0) or returns a readiness/pre-flight FAIL
  (exit 1). `mochiflow pr` keeps its exit contract (`0`/`10`/`3`/`1`/`2`) with
  the new `accepted`-based pre-flight.
- **Merged-derivation signal:** provider API → `Spec: {slug}` trailer reachable
  from `origin/{base_branch}` (two signals only; there is no persisted
  human-report fallback). Squash merges MUST carry the `Spec: {slug}` trailer
  into the squash commit; merge/rebase preserve it. The human merge report only
  triggers `close` locally and persists nothing.
- **Auth:** provider API path uses the existing `provider`/`pr_driver` resolution
  in `pr.rs` (e.g. `gh`); `provider = none` uses local git only.
- **Compatibility:** existing `_done/` specs and their stored `done`/`completed`
  remain valid and read-only; no migration. `ship` is removed (no alias).
- **Failure handling:** derivation errors are non-fatal and fall back to local/
  last-known state; lifecycle-state writes remain strict and fail loudly.
- **Verification:** conformance tests for vocabulary/INDEX/allowlist; unit tests
  for derivation and close-out; `freeze --check` for derived engine files.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Date: 2026-06-27

Independent reviewer (delegated, read-only) reviewed spec.md + AC Matrix,
design.md, tasks.md, pitch.md, the full branch diff, the core CLI sources, the
contract schema, and the SoT engine docs against the reported PASS of the full
`default` verification profile. All 18 ACs satisfied; global invariants AC-05
(no `status: done` / no `_done/` move for active specs) and AC-12 (`INDEX.md`
never staged/committed) hold. No Critical/High/Medium findings. Two Low findings,
both addressed in build:

1. router.md still presented the status enum as `draft|approved|done` and resolved
   "non-done" specs — updated to `draft|approved|accepted` (`done` legacy/derived)
   and "active (non-merged)" resolution.
2. `delivery::derive_column` probed git/provider even for already-`done` legacy
   specs — now short-circuits to Done without any probe.

The reviewer's optional suggestion to gate the `gh pr view` provider call behind
`--fetch` is left as a follow-up: the default provider is `none` (pure local
git), all probes degrade gracefully to `false` on failure (AC-08 holds), and the
change would alter the derivation interface; tracked for a future refinement
rather than this PR. AC-13 remains `PENDING_HUMAN` by design (human QA-08
round-trip is exercised in `open`, not build).
