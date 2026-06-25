# Add a QA attack matrix to plan and reviewer flows — Tasks

Implementation Summary: Add a risk-scaled persona attack dimension to QA Scenarios, plan, and reviewer Stage 1, reusing QA-XX traceability.
risk: standard
Critical Stop Conditions:
- A change starts requiring Rust/CLI lint code, a new reviewer stage, or an AC Matrix schema change (out of scope -> stop and route to plan).
- An existing or archived spec would stop passing `mochiflow lint` (retroactive enforcement -> stop).

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01] Add the persona dimension (P1-P7) to QA Scenarios in the spec templates
  - Depends on: none
  - Files: `engine/templates/spec/spec.standard.md`, `engine/templates/spec/spec.md`
  - Done: both templates' `## QA Scenarios` tables carry a persona column/guidance covering P1-P7, with an example that uses a reasoned `N/A: <reason>`; micro template left untouched.
  - Stop: a template edit would force persona coverage on micro specs.
- [x] T-002 [AC-03] Document risk-scaled attack-evidence strength in risk.md
  - Depends on: none
  - Files: `engine/reference/risk.md`
  - Done: risk.md states a single risk->persona/evidence mapping that owns both required persona coverage and evidence strength per risk level (standard exercises at least P1/P3/P6/P7 with reasoned `N/A` allowed; elevated requires evidence for relevant personas; critical requires strong evidence and rejects casual `N/A`), without duplicating the reviewer-cadence table.
  - Stop: the wording would introduce a new enforced gate beyond the existing risk consequences.
- [x] T-003 [AC-02, AC-05] Add persona-coverage and QA-XX traceability guidance to plan.md
  - Depends on: T-001
  - Files: `engine/commands/plan.md`
  - Done: plan.md instructs capturing risk-appropriate persona coverage in QA Scenarios by referencing the risk->persona/evidence mapping in risk.md (no restated thresholds), and references attacks via `QA-XX` from the AC Matrix `Planned test/QA` / `Evidence` columns, with no new AC column or attack-ID scheme.
  - Stop: guidance would require promoting attacks to formal ACs.
- [x] T-004 [AC-04] Extend reviewer Stage 1 with attack-evidence verification
  - Depends on: T-002
  - Files: `engine/agents/independent-reviewer.md`
  - Done: Stage 1 verifies, against the risk.md mapping, risk-appropriate persona-row presence, concrete `N/A` reasons, and that exercised rows carry evidence backing the attack; the reviewer references risk.md for this rule; no new stage and the Completion output format is unchanged.
  - Stop: a change would alter the reviewer output contract (Stage headings, Finding shape, or Completion output).
- [ ] T-005 [AC-06] Sync vendored engine + adapters and run full verification
  - Depends on: T-001, T-002, T-003, T-004
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`, generated adapter outputs (`AGENTS.md`, `.kiro/steering/mochiflow.md`)
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, and the `cli` `default` verification all pass with no drift.
  - Stop: `adapter generate --check` or `freeze --check` reports drift that cannot be resolved by regeneration alone.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | AI-observed | QA-01 | `engine/templates/spec/spec.standard.md`, `engine/templates/spec/spec.md` | UNVERIFIED | | persona dimension present in QA Scenarios |
| AC-02 | cli | AI-observed | QA-01, QA-02 | `engine/commands/plan.md` | UNVERIFIED | | references risk.md persona/evidence mapping, no restated thresholds |
| AC-03 | cli | AI-observed | QA-01 | `engine/reference/risk.md` | UNVERIFIED | | single owner of risk->persona/evidence mapping incl. standard set P1/P3/P6/P7 |
| AC-04 | cli | AI-observed | QA-02 | `engine/agents/independent-reviewer.md` | UNVERIFIED | | Stage 1 extended, output format unchanged |
| AC-05 | cli | AI-observed | QA-03 | `engine/commands/plan.md`, `engine/templates/spec/spec.standard.md` | UNVERIFIED | | QA-XX trace only, no new AC column/token |
| AC-06 | cli | automated | QA-03 | `engine/MANIFEST.json`, `.mochiflow/engine/**` | UNVERIFIED | | freeze/upgrade/adapter-check/default verify pass |
