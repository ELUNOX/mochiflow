# Post-build PR and Close Flow — Tasks

Implementation Summary: Replace `ship` with open/update/close, derive merged/in_review from VCS, keep specs flat, and make the board a computed/gitignored artifact.
risk: elevated
Critical Stop Conditions:
- Never introduce a base-branch commit/push, a `_done/` move, or a committed `INDEX.md` write.
- Edit engine docs/adapters in the repo-root `engine/` SoT, never only the vendored `.mochiflow/engine/` copy.
- Keep existing `_done/` archived specs read-only and lint-clean; do not migrate them.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Frozen-surface rule: any task that edits a frozen surface (`contracts/*.json` or `engine/**`) runs `mochiflow freeze` as its final step before verification — regenerating `contracts/contracts.lock` + the version gate and/or `engine/MANIFEST.json` — so that task's own `freeze --check` and the `version_gate_*` / `drift_doctor_*` conformance tests pass in isolation. T-015 still performs the final `freeze` + `upgrade --source engine` + `adapter generate`.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-03, AC-04] Add `accepted` across the spec contract surface
  - Depends on: none
  - Files:
    - `contracts/spec.schema.json`
    - `cli/crates/mochiflow-core/src/lint.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `contracts/spec.schema.json` `status` enum includes `accepted` (keeps `done`); `lint.rs` `allowed_statuses` adds `accepted`; the terminal acceptance gate is ADDED at `accepted` — specifically (a) all AC Matrix results done-eligible, (b) reviewer verdict when `risk ≥ elevated`, (c) every task checked, and (d) every AC covered by a task (the four conditions previously gated at `done`) — while the `approved` AC-in-matrix coverage check is RETAINED at `approved`, and the `("done", None)` completed-timestamp WARN stays scoped to legacy `done` reads only (not relocated to `accepted`, which never writes `completed`); conformance `GOOD_YAML`/`status` fixtures updated (incl. the `lint_fails_on_invalid_status` message-string assertion, which now lists `accepted`); tests prove an `accepted` spec passes, an unmet `accepted` fails (unchecked task and untasked AC each rejected), a reviewer verdict is required at `accepted` when `risk ≥ elevated`, and no spurious completed-WARN fires on `accepted`. Per the Defaults frozen-surface rule, this task runs `mochiflow freeze` to regenerate `contracts/contracts.lock` + the version gate.
  - Stop: if the status set is owned outside `lint.rs`/schema, stop and confirm the single owner first.
- [x] T-002 [AC-06] Keep legacy `_done/`+`done` read-only compatible
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/lint.rs`
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `lint` accepts `done` only for specs located under `{specs_dir}/_done/` and rejects `status: done` on an active (non-`_done/`) spec; `index`/board still renders `_done/` specs in Done ordered by `completed`/`updated`; tests include a positive archived-`done` case and a negative case (an active flat spec with `status: done` fails lint).
  - Stop: do not rewrite or move any `_done/` content.
- [x] T-003 [AC-07, AC-08] Add the delivery-state derivation module
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
  - Done: a pure function resolves each spec to one delivery column using the precedence Done > In Review > Ready > Active (Done = provider-merged or `Spec: {slug}` trailer reachable from `origin/{base_branch}`; In Review = open PR, or for `provider = none` a pushed-and-unmerged branch; Ready = accepted-unpushed) with two merged signals only (no persisted human-report fallback); the `provider = none` and provider-unavailable paths work and never error; unit tests cover accepted-unpushed→Ready, accepted-pushed→In Review, merged→Done, a conflicting open-PR+merge-trailer case (Done wins), and provider-unavailable fallback.
  - Stop: do not persist any derived/human-report state; derivation is read-only observation.
- [x] T-004 [AC-09, AC-10] Add `mochiflow status` board command (read-only)
  - Depends on: T-003
  - Files:
    - `cli/crates/mochiflow-core/src/status.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-core/src/doctor.rs`
  - Done: `mochiflow status` prints Backlog/Active/Ready/In Review/Done computed from asserted ∪ derived state and writes no file (read-only); `--fetch` performs one `git fetch` first; exit 0 even with degraded derivation; `status` is added to `doctor.rs` `terminal_cli_command_references` so the exact-match allowlist test (`doctor_terminal_command_allowlist_matches_clap_subcommands`) stays green in this task; a unit test asserts column placement and that no file (incl. `INDEX.md`) is written.
  - Stop: `status` must never write `INDEX.md` or mutate any spec.
- [x] T-005 [AC-11, AC-12] Gitignore INDEX, derive its columns, auto-regenerate on state-changing commands
  - Depends on: T-004
  - Files:
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-core/src/config.rs`
    - `.mochiflow/.gitignore`
  - Done: `INDEX.md` columns come from the same board computation as `status`, and `index.rs` rendering (`status_emoji` + pipeline counts) gains `accepted`; `init.rs` `write_install_gitignore` generates an install `.gitignore` that ignores the configured index filename (so new `mochiflow init` projects never track `INDEX.md`); this repo's `.mochiflow/.gitignore` ignores `INDEX.md` and its already-tracked `.mochiflow/INDEX.md` is untracked once with a manual `git rm --cached`; existing user repos perform the same one-time manual untrack (called out in the engine docs) — automated `join`/`upgrade` migration is explicitly out of scope for this change; a shared post-command step regenerates `INDEX.md` after state-changing commands (`accept`, `pr`, `index`) — never on `status`; no command stages/commits `INDEX.md`; the `ship`-referencing generated text is updated off `ship` (init.rs `ADR_STUB_BODY` and the config.rs fold-target comment point to `open`/the fold step); tests assert the generated install gitignore ignores the index and that `INDEX.md` is untracked after a state-changing command.
  - Stop: if `INDEX.md` is referenced as a committed artifact in freeze/doctor/manifest, reconcile before flipping it to gitignored.
- [x] T-006 [AC-05] Repurpose ship close-out into `accept` (no done/_done/INDEX)
  - Depends on: T-001, T-005
  - Files:
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: the deterministic close-out sets `status: accepted` (+ `updated`), settles automated AC rows, lints, stages `{specs_dir}/{slug}/**` + ADR paths and creates the single close-out commit — with the `done`/`completed` write, the `_done/` `fs::rename`, and the in-close-out `index::generate` removed; the `Accept` clap subcommand replaces `Ship` and `doctor.rs` `terminal_cli_command_references` swaps `ship`→`accept` in the same task so the exact-match allowlist test stays green; the `behavioral_ship_*` conformance family (incl. `behavioral_ship_commits_active_spec_archive_with_safe_paths` and the `materialize_ship_repo`/`write_active_ship_spec` helpers) is rewritten for the `accept` semantics. `accept` commits whatever ADR fold the caller (`open`) has already written and does NOT author the fold itself; tests prove no `done`, no `_done/` move, no `INDEX` staged.
  - Stop: do not delete the module's clean-tree/readiness pre-flight; only change its target state and staged paths.
- [ ] T-007 [AC-17] Update `mochiflow pr` pre-flight to `accepted`+trailer
  - Depends on: T-006
  - Files:
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-core/src/ship.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/tests/pr.rs`
  - Done: pre-flight requires the active `{specs_dir}/{slug}/` committed with status `accepted` and a `Spec: {slug}` trailer present (rewriting `validate_pr_spec_closeout_committed` in `ship.rs`, which `pr.rs` calls, to drop the `_done/`+`done` check); the `behavioral_pr_*` conformance tests (`behavioral_pr_slug_guard_requires_committed_ship_closeout`, `behavioral_pr_path_like_spec_preserves_request_dir_behavior`) and `tests/pr.rs` (`mark_shipped` helper + the pre-flight expectations at its call sites) are reworked for the `accepted`+trailer pre-flight; pr tests updated and green.
  - Stop: keep the `pr` exit-code contract (`0`/`10`/`3`/`1`/`2`) unchanged.
- [ ] T-008 [AC-18] Update doctor workflow vocabulary + conformance command set
  - Depends on: T-004, T-006
  - Files:
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
  - Done: `workflow_command_references` reflects the open/update/close verb vocabulary (the `terminal_cli_command_references` allowlist is already updated in T-004/T-006); the conformance command-set/vocabulary assertions match the new set (`status` in, `ship` out); the allowlist test and `mochiflow doctor` pass.
  - Stop: do not loosen the allowlist test; keep it exact.
- [ ] T-009 [AC-01] Remove user-facing `ship` across `engine/` SoT docs, adapters, and CLI-surfaced text/tests
  - Depends on: T-001, T-003, T-004, T-005, T-006, T-007
  - Files:
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/commands/close.md`
    - deleted: `engine/commands/ship.md`
    - `engine/router.md`
    - `engine/reference/workflow.md`
    - `engine/reference/git.md`
    - `engine/reference/authoring.md`
    - `engine/templates/delivery/pr-description.md`
    - `engine/templates/handoff/build-session-prompt.md`
    - `engine/README.md`
    - `engine/commands/refresh-context.md`
    - `engine/commands/onboard.md`
    - `engine/reference/language.md`
    - `engine/adapters/kiro/steering/mochiflow.md.tpl`
    - `engine/adapters/agents/AGENTS.md.tpl`
    - `engine/adapters/claude-code/CLAUDE.md.tpl`
    - `engine/adapters/copilot/copilot-instructions.md.tpl`
    - `cli/crates/mochiflow-core/src/present.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/tests/first_run.rs`
    - any other file surfaced by the `rg` sweep (see Done)
  - Done: a repo-wide `rg` over `engine/**/*.md`, all adapter templates, and CLI-surfaced verb text finds no residual user-facing `ship` verb/trigger and no stale `done`/`_done`/committed-`INDEX` lifecycle wording; `open`/`update`/`close` are defined with triggers/prerequisites/procedure; router routes them; the state table is `draft → approved → accepted` + derived `in_review`/`merged`; the docs state that every spec-lane procedure commit step (discuss/plan/build/open/update/close) regenerates the board via `mochiflow index` so `INDEX.md` stays fresh (AC-11); `present.rs` `render_guide` replaces the `ship`/`mochiflow-ship` verb line (both `en` and `ja`), reworks the "Four verbs" framing to open/update/close, and its co-located unit test (`guide_card_is_static_with_verbs_commands_and_gates`) is updated; `tests/first_run.rs` (the `mochiflow-ship` guide-card assertion, ~line 255) is updated to the new verb set; and the conformance tests coupled to the deleted `ship.md`/`_done` model are removed/rewritten — `pr_feedback_restore_*`, `ship_defers_context_refresh_*`, `ship_guidance_*`, the router merged-event `_done` exception tests, the build.md/git.md `ship.md ## PR Feedback Loop` assertions, and the two tests that `read_repo_file("engine/commands/ship.md")` (`ac_matrix_pending_human_is_canonical_provisional_token`, `no_pr_fast_path_skips_pr_gate_but_still_ships`) — with new AC-01 assertions added (no user-facing `ship` verb; `open`/`update`/`close` defined).
  - Stop: edit the repo-root `engine/` SoT, not the vendored `.mochiflow/engine/` copy.
- [ ] T-010 [AC-02] Update build.md end-state (approved, no PR/terminal/move)
  - Depends on: T-009
  - Files:
    - `engine/commands/build.md`
  - Done: build ends at `approved`, points to `open` for PR prep, explicitly does not set a terminal state / create a PR / move the spec; the completion card offers "Create the PR" (open) / resume.
  - Stop: do not change build's verification/AC-Matrix responsibilities.
- [ ] T-011 [AC-13] Author `open` procedure (acceptance → accepted → fold → PR → gate)
  - Depends on: T-009
  - Files:
    - `engine/commands/open.md`
    - `engine/templates/delivery/pr-description.md`
  - Done: `open.md` specifies ordered steps a–f — (a) acceptance incl. QA round-trip; (b) finalize the fold by writing durable ADR knowledge; (c) run `accept`, which sets `accepted` and creates the single close-out commit bundling the spec and the already-written ADR fold; (d) generate PR title/body; (e) approve-PR gate; (f) push + `mochiflow pr` — and states the PR is never created before the gate and that `open` (not `accept`) owns authoring the fold.
  - Stop: do not introduce a second human gate beyond approve-PR.
- [ ] T-012 [AC-14] Author `update` procedure (delegate build, no move/revert)
  - Depends on: T-009
  - Files:
    - `engine/commands/update.md`
  - Done: `update.md` delegates code changes to the build loop, re-verifies, pushes, updates PR metadata and (when changed) the fold, and explicitly does not move the spec or revert asserted state.
  - Stop: do not reimplement build logic inside update.
- [ ] T-013 [AC-15] Author `close` procedure (local hygiene only)
  - Depends on: T-009
  - Files:
    - `engine/commands/close.md`
    - `engine/reference/git.md`
  - Done: `close.md` performs only local hygiene (switch base, ff pull, delete branch, clear `state/{slug}/`, regenerate board) and states it writes nothing to the base branch; git.md post-merge section matches; the human merge report triggers `close` but persists no merged flag.
  - Stop: do not add any base-branch commit/push to close.
- [ ] T-014 [AC-16] Stale-base guard at spec start
  - Depends on: T-003, T-009
  - Files:
    - `engine/commands/discuss.md`
    - `engine/reference/git.md`
  - Done: discuss/branch-creation fetches and branches from `origin/{base_branch}` and warns when the local base is behind; the behavior is provider-independent and documented.
  - Stop: if this requires a new CLI subcommand, stop and confirm scope before adding one.
- [ ] T-015 [chore: derived-file integrity + full verification] Freeze, re-vendor, and finalize verification
  - Depends on: T-001, T-002, T-003, T-004, T-005, T-006, T-007, T-008, T-009, T-010, T-011, T-012, T-013, T-014
  - Files:
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
    - regenerated adapter outputs (`AGENTS.md`, `.kiro/steering/*`, `CLAUDE.md`, copilot instructions)
  - Done: `mochiflow freeze` regenerates `engine/MANIFEST.json`, `contracts/contracts.lock`, and the version gate; `mochiflow upgrade --source engine` re-vendors into `.mochiflow/engine/`; `mochiflow adapter generate` regenerates the tracked adapter outputs (root `AGENTS.md`, `.kiro/steering/*`, `CLAUDE.md`, copilot instructions) and `mochiflow adapter generate --check` then confirms they are in sync; the full `default` verification (test + fmt + clippy + freeze --check) is green.
  - Stop: do not hand-edit generated files; regenerate via `freeze`/`upgrade`/`adapter generate`.

## Verification Plan / AC Matrix

The AC Verification Matrix is maintained in `spec.md ## Verification Plan / AC
Matrix` (the canonical location per `reference/workflow.md ## AC Matrix`). Build
records results there.
