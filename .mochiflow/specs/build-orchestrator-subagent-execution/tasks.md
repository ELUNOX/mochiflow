# Run build as an orchestrator dispatching disposable per-task workers — Tasks

Implementation Summary: make build an orchestrator that dispatches sequential disposable per-task workers over one shared delegation transport, with a new worker role, a refined principle 5, git-reconstructed full-diff review, a plan-time recoverability rule, and a generated kiro worker agent.
risk: elevated
Critical Stop Conditions:
- Edit the repo-root `engine/` SoT, never the vendored `.mochiflow/engine/` copy.
- Keep this change additive: the inline build path stays as the fallback and the `risk.md` reviewer cadence is unchanged — no behavior change when running inline.
- Do not introduce model downgrade, parallel workers, a git worktree, a new deterministic lint, or any new CLI subcommand / contract change.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Frozen-surface rule: any task that edits a frozen surface (`engine/**` or `contracts/*.json`) runs `mochiflow freeze` as its final step before verification — regenerating `engine/MANIFEST.json` and/or `contracts/contracts.lock` + the version gate — so that task's own `freeze --check` and the `version_gate_*` / `drift_doctor_*` conformance tests pass in isolation. T-008 still performs the final `freeze` + `upgrade --source engine` + `adapter generate`.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing.

## Tasks

- [x] T-001 [AC-02, AC-08] Generalize the delegation transport in risk.md
  - Depends on: none
  - Files:
    - `engine/reference/risk.md`
  - Done: the `## Review transport` section is generalized into a single shared delegation transport (prefer `delegated` subagent when the runtime supports it, else `inline`) described as reused by both the read-only `independent-reviewer` and the new write-capable `worker` role, with no second transport introduced; **the `## Review transport` heading is preserved** (only its body is generalized) so the by-name citations in `router.md` principle 5 and `build.md` step 3f stay resolvable — if a rename is unavoidable, update every by-name citation in lockstep; a statement is added that the mandatory risk-cadence review reconstructs the diff from git and never uses compact reports (or conversation history) as evidence; the reviewer cadence table in `## Consequences` is unchanged (standard=none / elevated=once after all tasks / critical=after each task). Per the Defaults frozen-surface rule this task runs `mochiflow freeze`.
  - Stop: do not change the reviewer cadence or the QA attack-coverage mapping; only generalize transport and add the evidence/diff rule.
- [x] T-002 [AC-03, AC-05, AC-06, AC-09] Author the worker role doc
  - Depends on: T-001
  - Files:
    - `engine/agents/worker.md`
  - Done: a new `engine/agents/worker.md` defines a write + verify + commit worker role distinct from the read-only reviewer; it specifies (a) the context pack it consumes (relevant `design.md` slice, the single `tasks.md` row with `Files`/`Done`/`Stop`/AC refs, the `default` verify command, and constitution/standards/pitfalls pointers), (b) repo-wide read but contract-bounded write — an out-of-scope edit returns `blocked` instead of widening scope, (c) the compact report it returns (`task`, `status` done|blocked, `files_changed`, `verify` profile+result+evidence pointer, `commit` ref, `reason` when blocked) excluding any implementation narrative, (d) that it runs on the top model with no downgrade, and (e) that it performs the build per-task commit cadence. The doc is project-agnostic English consistent with `engine/agents/independent-reviewer.md` style. Runs `mochiflow freeze` per Defaults.
  - Stop: do not give the worker authority over acceptance, fold, PR, or human gates; if the role would need to make a design decision, that is a `blocked` return by contract.
- [x] T-003 [AC-01] Refine router principle 5 and the Verb Delegation table
  - Depends on: T-001, T-002
  - Files:
    - `engine/router.md`
  - Done: principle 5 is reworded to separate the invariant (judgment / gates / integration / fold stay single-threaded on the top model) from the execution transport (a verified code-change task's execution may fan out to disposable workers via the shared transport when available, else inline), and "Review is the only separated procedure" becomes "review and per-task build execution are the delegated procedures over one shared transport"; the Verb Delegation table is realigned so the `build` row reads inline-or-delegated per-task workers (reviewer transport reused), and the `open` / `update` rows reflect that they reuse the build worker only where code changes happen (open's QA-`FAIL` rework, update's PR-feedback change) while their judgment stays inline — keeping the table consistent with the AC-11 boundaries; no other principle is weakened. Runs `mochiflow freeze` per Defaults.
  - Stop: do not relax the judgment-single-threaded invariant; this is a wording refinement, not a behavioral relaxation. `engine/router.md` is the single owner of the Verb Delegation table — do not duplicate that table into open/update docs (T-005).
- [ ] T-004 [AC-04, AC-07] Rewrite the build task loop as orchestrator + workers
  - Depends on: T-002, T-003
  - Files:
    - `engine/commands/build.md`
  - Done: `build.md` adds an orchestrator mode that triggers WHEN `tasks.md` has `>= 2` open tasks AND a subagent mechanism exists, dispatching sequential disposable per-task workers (one at a time on the single working tree, dependency order, no `[P]` parallelism) via the shared transport; otherwise build runs the existing inline task loop unchanged (explicit fallback); the orchestrator holds only the plan/contract, assembles the context pack per task, receives the compact report, and advances; the worker performs the existing 3e per-task commit cadence (mark the `tasks.md` checkbox, then one `Task:`-trailer commit per task) so commit granularity stays one task per commit; the `risk.md` reviewer cadence is preserved EXACTLY — for `critical`, the orchestrator runs `independent-reviewer` on each task's own git commit before advancing; for `elevated`, once after all tasks; for `standard`, none — and any review reconstructs the diff from git, never from the report; **write ownership is explicit**: the worker owns the checkbox tick + per-task code commit, the orchestrator owns the AC Matrix rows recorded once at build completion (step 7), so there is no per-task double-write to `tasks.md` and the resume reconciliation source (checkboxes + `Task:` trailers) is unchanged; `build.md` step 6 / 3d matrix-location wording is aligned to the `spec.md ## Verification Plan / AC Matrix` canonical location (per `reference/workflow.md ## AC Matrix`), replacing the "at the end of tasks.md if present" phrasing; `delegate_to` frontmatter adds `agents/worker.md`; final verification still runs after all tasks. Runs `mochiflow freeze` per Defaults.
  - Stop: do not change build's verification responsibilities, the reviewer cadence, the resume reconciliation, or the AC Matrix token rules; if orchestrator commit handling would alter the commit cadence or cause a per-task AC-Matrix write, stop and confirm.
- [ ] T-005 [AC-11] State phase boundaries in open / update / close
  - Depends on: T-004
  - Files:
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/commands/close.md`
  - Done: `open.md` states its QA-`FAIL` rework loop reuses the build worker mechanism and that acceptance / fold / PR-body / approve-PR gate stay inline; `update.md` states the PR-feedback code change reuses the build worker mechanism while feedback interpretation and PR-metadata updates stay inline; `close.md` states it delegates nothing (deterministic local hygiene); no verb defines its own separate delegation path. Runs `mochiflow freeze` per Defaults.
  - Stop: do not duplicate the worker mechanism into open/update; they only reference build's. Do not add a base-branch write to close.
- [ ] T-006 [AC-10] Add the worker-recoverability authoring rule
  - Depends on: T-004
  - Files:
    - `engine/commands/plan.md`
    - `engine/reference/authoring.md`
  - Done: `plan.md` and `authoring.md` document the worker-recoverability invariant — every fact needed to implement a task must be recoverable from `design.md` + the task row + reading committed code, so cross-task reasoning that inline build would carry implicitly is written into `design.md` at plan time, and a file appearing in more than one task's `Files` documents its shared-state handling in each such task's `Done`; the rule is stated as plan authoring discipline enforced by reviewer Stage 1 judgment, explicitly NOT a new deterministic lint. Runs `mochiflow freeze` per Defaults.
  - Stop: do not add a lint check for recoverability; if a mechanical check seems necessary, stop and confirm scope.
- [ ] T-007 [AC-12] Generate the kiro spec-worker agent
  - Depends on: T-002
  - Files:
    - `engine/adapters/kiro/agents/spec-worker.json.tpl`
    - `engine/adapters/kiro/manifest.toml`
    - `cli/crates/mochiflow-core/src/adapter.rs`
  - Done: a new `spec-worker.json.tpl` modeled on `spec-independent-reviewer.json.tpl` is added with write-capable tools (`read`, `grep`, `glob`, `edit`, `write`, `bash`), the same top model, `prompt` pointing at `engine/agents/worker.md`, and resources including `worker.md` + the references it needs; `manifest.toml` maps `.kiro/agents/spec-worker.json` to it; `adapter.rs is_kiro_agent_json` is extended to also match `.kiro/agents/spec-worker.json` (full-file managed) and its co-located unit tests assert the new agent is generated as a full-file managed target while the reviewer agent still generates; `adapter generate --check` is green for the regenerated set. Edits `engine/**` so runs `mochiflow freeze` per Defaults; the vendored re-install + `adapter generate` happen in T-008.
  - Stop: do not hand-edit generated adapter outputs; if the worker agent needs a deny/trust wiring beyond the reviewer's per-call model, stop and confirm rather than reintroducing a baked tool policy.
- [ ] T-008 [AC-13] Freeze, re-vendor, regenerate adapters, finalize verification
  - Depends on: T-001, T-002, T-003, T-004, T-005, T-006, T-007
  - Files:
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
    - regenerated adapter outputs (`AGENTS.md`, `.kiro/steering/*`, `.kiro/agents/*`, `CLAUDE.md`, copilot instructions)
  - Done: `mochiflow freeze` regenerates `engine/MANIFEST.json` + `contracts/contracts.lock` + the version gate; `mochiflow upgrade --source engine` re-vendors into `.mochiflow/engine/`; `mochiflow adapter generate` regenerates the tracked adapter outputs (including the new `.kiro/agents/spec-worker.json`) and `mochiflow adapter generate --check` confirms they are in sync; the full `default` verification (test + fmt + clippy + freeze --check) is green.
  - Stop: do not hand-edit generated files; regenerate via `freeze` / `upgrade` / `adapter generate`.

## Verification Plan / AC Matrix

The AC Verification Matrix is maintained in `spec.md ## Verification Plan / AC
Matrix` (the canonical location per `reference/workflow.md ## AC Matrix`). Build
records results there. This file is the executable checklist only.
