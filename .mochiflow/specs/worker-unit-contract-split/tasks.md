# Name the worker execution units: build-task vs rework — Tasks

Implementation Summary: Name the two worker execution units via an explicit `unit_kind` discriminator in the worker contract, align the verb references, and add conformance assertions.
risk: standard
Critical Stop Conditions:
- A second worker role/agent or a second delegation transport is needed (rejected scope — route back to plan).
- The change would require a Rust struct / serialization for the compact report (the report is a prose contract — route back to plan).

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03, AC-04] Revise the worker contract and verb references; sync the vendored engine
  - Depends on: none
  - Files:
    - `engine/agents/worker.md`
    - `engine/commands/build.md`
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
  - Done: `worker.md` defines `unit_kind ∈ {build-task, rework}` as the primary behavioral discriminator (checkbox + `Task:` trailer for `build-task`; host verb's commit convention for `rework`), documents `unit_kind` as a dispatch-time field of `## Context pack`, keys STOP routing on `unit_kind`, and specifies a single uniform compact report including `unit_kind` with no `feedback_id`/`qa_item`/`task` fields; `build.md`/`open.md`/`update.md` reference the unit using the `unit_kind` vocabulary consistently with `worker.md` and carry no contradictory prefix-only behavioral language; the dogfood sync (`mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`) is run so the gitignored vendored `.mochiflow/engine` copy is regenerated wholesale and the generated `spec-worker.json` is unchanged; the `default` profile (including `freeze --check`) is green.
  - Stop: a second worker role/agent or transport, or any change to `spec-worker.json` tools/model/prompt, appears necessary (route back to plan).
- [ ] T-002 [AC-01, AC-02, AC-03] Add conformance assertions for the worker unit_kind contract
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: a conformance test asserts that `engine/agents/worker.md` names both `unit_kind` values (`build-task` and `rework`), keys behavior on `unit_kind` rather than the id prefix, documents `unit_kind` as a `## Context pack` dispatch-time input, and lists a uniform compact report containing `unit_kind` without `feedback_id`/`qa_item`/`task`; and that `build.md`/`open.md`/`update.md` reference the `unit_kind` vocabulary; the `default` profile is green.
  - Stop: asserting the contract mechanically requires changing engine wording beyond T-001's authored contract (route back to T-001 / plan).
