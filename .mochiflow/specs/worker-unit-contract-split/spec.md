# Name the worker execution units: build-task vs rework

## Background and Design Rationale

The delegated build orchestrator (shipped 2026-06-28,
`build-orchestrator-disposable-workers`) introduced one `worker` role reused by
`build`, `open`, and `update`. The word "worker" silently covers two different
execution units: a `build` task (`T-###`, with a `tasks.md` row, checkbox, and
`Task:` trailer) and a bounded `open`/`update` fix (QA-`FAIL` rework or
PR-feedback, with no task row, no checkbox, no trailer).

`engine/agents/worker.md` already branches on the host phase in prose, but the
two units have **no name**, and the behavioral split is dispatched implicitly by
reading the `unit` id prefix (`T-###` vs `qa-fail:` / `pr-feedback:`). That
prefix is not 1:1 with the two behaviors — `rework` spans two prefixes — so the
contract is honest-by-convention rather than honest-by-construction. This is the
residual ambiguity the original review flagged.

Key decisions (agreed in discuss):

- Keep one shared role, one shared delegation transport, and the single
  generated `spec-worker.json` agent. Do **not** add a second
  `agents/rework-worker.md` role or a second transport — both were already
  rejected by `build-orchestrator-disposable-workers` (duplicate
  selection/fallback logic, SSOT violation).
- Introduce `unit_kind` as the **primary behavioral discriminator**:
  `unit_kind ∈ {build-task, rework}`. Behavior (checkbox + `Task:` trailer vs the
  host verb's commit convention, and the STOP routing destination) is selected by
  `unit_kind`, not by parsing the `unit` id prefix. The `unit` id stays as the
  human-readable identifier (`T-###` / `qa-fail:<id>` / `pr-feedback:<id>`).
- The compact report keeps a **single uniform schema** across both kinds, adding
  `unit_kind` as the only new field; no unit-specific fields (`feedback_id` /
  `qa_item` / `task`) — they collapse into `unit`.
- Naming style is tokens + plain prose: values `build-task` / `rework`, referred
  to as "build-task worker" / "rework worker"; not CamelCase type names, because
  the contract is agent-to-agent prose with no corresponding Rust type.
- `unit_kind` is set at dispatch time by the orchestrator (build) or the host
  verb (open/update) in the context pack; the generated `spec-worker.json` agent,
  its tools, and its model are unchanged.

This is a documentation/contract clarification with no runtime or Rust code
change. The compact report and context pack are agent-to-agent prose contracts,
not parsed Rust types, so `unit_kind` is a documented field, not a serde change.

Origin: backlog seed `worker-unit-contract-split` (source: conversation, from the
`build-orchestrator-subagent-execution` review).

## User Story

As a mochiflow maintainer or an agent executing build/open/update, I want the
two worker execution units to be explicitly named and dispatched by an explicit
`unit_kind` discriminator, so that the worker contract is unambiguous without
inferring the unit from an id-prefix convention.

## Scope

- In: `engine/agents/worker.md` (the worker contract); the worker-unit
  references in `engine/commands/build.md`, `open.md`, and `update.md`;
  conformance assertions for the worker contract; the dogfood regeneration of
  derived engine artifacts (`engine/MANIFEST.json`, `contracts/contracts.lock`,
  the vendored `.mochiflow/engine` copy).
- Out: any second worker role or agent file; any change to `spec-worker.json`
  tools / model / prompt path; the delegation threshold (≥ 2 open tasks) and its
  decoupling from `risk`; any Rust struct / serialization for the report; any
  runtime/CLI behavior change.

## Edge Cases

- `rework` has two host sub-forms (`qa-fail:` from open, `pr-feedback:` from
  update) that differ only in id prefix and host verb, not in worker behavior.
  They MUST remain one `unit_kind` (`rework`), not a third kind.
- A worker that hits a STOP condition still routes by host phase: `build-task` →
  orchestrator stops the loop and routes to `plan`; `rework` → host verb pauses
  (a genuine new design decision routes to `plan`; an ambiguity in the fix
  contract routes back to the host verb's interpretation step). The `unit_kind`
  selects this routing.
- The build per-task checkbox tick and `Task:` trailer are exclusive to
  `unit_kind: build-task`; `rework` never ticks a checkbox or writes a `Task:`
  trailer.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL define, in `engine/agents/worker.md`, `unit_kind` with
  exactly the two values `build-task` and `rework` as the primary discriminator
  that selects worker behavior (the checkbox + `Task:` trailer cadence for
  `build-task` vs the host verb's commit convention for `rework`), rather than
  selecting behavior by parsing the `unit` id prefix.
- AC-02: THE SYSTEM SHALL specify a single uniform compact-report schema in
  `engine/agents/worker.md` that includes the `unit_kind` field alongside `unit`,
  `status`, `files_changed`, `verify`, `commit`, and `reason`, and SHALL NOT
  introduce per-unit report fields such as `feedback_id`, `qa_item`, or `task`.
- AC-03: THE SYSTEM SHALL describe the worker execution unit in
  `engine/commands/build.md`, `engine/commands/open.md`, and
  `engine/commands/update.md` using the `unit_kind` vocabulary (`build-task` /
  `rework`) consistently with `engine/agents/worker.md`, with no remaining
  prefix-only behavioral language that contradicts the discriminator.
- AC-04: WHERE the engine source is edited, THE SYSTEM SHALL keep the generated
  `.kiro/agents/spec-worker.json` (tools, model, prompt path) unchanged and keep
  `mochiflow adapter generate --check`, `mochiflow freeze --check`, and the
  worker conformance tests green after the dogfood regeneration.

## QA Scenarios

> Standard-risk docs/contract change with no end-user or runtime surface. The
> applicable personas are P6 (regression) and P7 (spec skeptic); user-facing and
> data personas are reasoned `N/A` per `reference/risk.md ## QA attack coverage`.

| QA | Persona | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- | --- |
| QA-01 | P1, P2 | cli | Automated | N/A check: there is no end-user-facing operation; this is an engine-internal agent-to-agent contract. | N/A: no end-user surface in a docs/contract change. |
| QA-02 | P3 | cli | Automated | N/A check: no input, auth, or boundary surface is added. | N/A: no executable input/permission surface; prose contract only. |
| QA-03 | P4 | cli | Automated | N/A check: no persisted data or state is read or written. | N/A: no data-integrity surface. |
| QA-04 | P5 | cli | Automated | N/A check: no data/format migration; archived specs are untouched. | N/A: no migration surface. |
| QA-05 | P6 | cli | Automated | Run the full `default` profile after the change: `cargo test`, `fmt --check`, `clippy -D warnings`, `freeze --check`; confirm the existing worker/adapter conformance tests and build/open/update behavior are unaffected. | All existing tests pass; `freeze --check` and `adapter generate --check` are green; no behavioral regression. |
| QA-06 | P7 | cli | Automated | Compare `engine/agents/worker.md` and the verb references against the agreed contract: assert both `unit_kind` values are named, behavior is keyed on `unit_kind`, and the uniform report includes `unit_kind` with no per-unit id fields. | Conformance assertions confirm the documents match the agreed `unit_kind` contract. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | QA-06 conformance assertion (worker.md names `build-task`/`rework`, behavior keyed on `unit_kind`) | `engine/agents/worker.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-02 | cli | automated | QA-06 conformance assertion (uniform report includes `unit_kind`, no `feedback_id`/`qa_item`/`task`) | `engine/agents/worker.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-03 | cli | automated | QA-06 conformance assertion (verb references use `unit_kind` vocabulary) | `engine/commands/build.md`, `engine/commands/open.md`, `engine/commands/update.md`, `cli/crates/mochiflow-cli/tests/conformance.rs` | UNVERIFIED | | |
| AC-04 | cli | automated | QA-05 regression (`freeze --check`, `adapter generate --check`, worker conformance green; agent unchanged) | `engine/MANIFEST.json`, `contracts/contracts.lock`, `.mochiflow/engine/agents/worker.md` | UNVERIFIED | | |
