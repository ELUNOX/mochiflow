# Agent Context API — Tasks

Implementation Summary: Add a versioned read-only agent context contract, consolidate repository observation and lifecycle eligibility, and migrate existing consumers without breaking their outputs.
risk: elevated
Critical Stop Conditions:
- Stop if lifecycle eligibility cannot remain separate from natural-language routing or would create a competing route authority.
- Stop if an existing `status`, `index`, `ready`, config/spec, or `state/index.json` contract must break rather than remain an explicit compatibility projection.
- Stop if unknown Git/provider state must be treated as known false or if batch collection cannot avoid per-spec external probes.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-05, AC-11] Freeze the Agent Context contract and release baseline
  - Depends on: none
  - Files:
    - `contracts/agent-context.schema.json`
    - `tests/conformance/fixtures/schema/agent-context-repository-good.json`
    - `tests/conformance/fixtures/schema/agent-context-spec-good.json`
    - `tests/conformance/fixtures/schema/agent-context-degraded-good.json`
    - `tests/conformance/fixtures/schema/agent-context-partial-good.json`
    - `tests/conformance/fixtures/schema/agent-context-error-good.json`
    - `tests/conformance/fixtures/schema/agent-context-invalid.json`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/Cargo.toml`
    - `cli/Cargo.lock`
    - `CHANGELOG.md`
    - `README.md`
    - `README.ja.md`
    - `engine/VERSION`
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
    - `AGENTS.md`
    - `.kiro/steering/mochiflow.md`
    - `.kiro/agents/spec-plan-auditor.json`
    - `.kiro/agents/spec-change-reviewer.json`
  - Done: Draft 2020-12 fixtures pin repository, spec, degraded, partial, and error envelopes plus negative validation; the additive feature release is coherently `1.3.0` across public references and frozen artifacts; config `schema_version` is unchanged; the default verification passes with the new frozen hash.
    The configured `agents` and `kiro` targets are regenerated immediately after the version bump and `mochiflow adapter generate --check` passes, so every task boundary is version-coherent; T-006 intentionally regenerates the same shared targets again after engine guidance changes and verifies that no additional adapter target changed.
  - Stop: If the response variants cannot be expressed as one closed additive schema without changing the existing index JSON or consumer config/spec schemas, stop and return to plan.

- [x] T-002 [AC-01, AC-02, AC-04, AC-09] Build the shared snapshot and batched observation core
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/inspect.rs`
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-core/src/spec_mode.rs`
    - `cli/crates/mochiflow-core/src/lib.rs`
  - Done: The shared immutable snapshot discovers valid and malformed specs, obtains current branch/worktree/ref/trailer/ignore/provider facts through an injected constant-bounded collector, preserves known/unknown/not-applicable quality, detects provider truncation, and leaves `inspect.rs` compiling with focused unit tests; `delivery.rs` keeps one pure precedence rule and no new per-spec I/O path is introduced.
  - Stop: If provider batching cannot distinguish unavailable/truncated results from a complete negative result, or if local-only persistence requires one process per spec rather than one batch, stop and report the unsupported observation.

- [x] T-003 [AC-02, AC-03, AC-10] Centralize structured health and lifecycle eligibility
  - Depends on: T-002
  - Files:
    - `cli/crates/mochiflow-core/src/inspect.rs`
    - `cli/crates/mochiflow-core/src/lint.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: Lint exposes a pure structured report while retaining its presenter; table-driven tests enumerate every row and blocker in `design.md`'s closed six-action eligibility table, including discuss state/intent confirmation, both plan input shapes, unknown propagation, stable blocker order, and suggestion precedence; `ready` and detailed build eligibility share the lint/status/verification readiness core, while branch/worktree entry blockers remain outside the legacy `ready` projection; dirty-worktree and missing-expected-branch regressions prove `ready` retains its current output/exit behavior while inspect reports the full build action result; shared `inspect.rs` remains a complete compiling boundary after the task.
  - Stop: If a prerequisite depends on user intent, hidden conversation state, or non-deterministic prose judgment, represent it as unknown/not auto-suggested rather than inventing a deterministic rule; stop if this prevents the agreed six-action contract.

- [ ] T-004 [AC-01, AC-02, AC-05, AC-06, AC-07, AC-10] Expose the inspect CLI and safe presenters
  - Depends on: T-003
  - Files:
    - `cli/crates/mochiflow-core/src/inspect.rs`
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: `inspect [slug] [--json] [--fetch]` emits concise localized human output or exactly one schema-valid JSON document; result/exit semantics match the contract even for pre-dispatch config errors; ordinary execution is proven read-only; explicit fetch is bounded and fail-soft; paths and diagnostics are sanitized; repository and detail payloads pass every positive/negative behavior scenario; shared `inspect.rs` finishes with no presentation side effect inside snapshot construction.
  - Stop: If JSON error output requires raw stderr, absolute paths, command bodies, or a second config parser, stop and redesign the safe error boundary instead of weakening the contract.

- [ ] T-005 [AC-08, AC-09] Migrate board consumers to the shared snapshot without drift
  - Depends on: T-004
  - Files:
    - `cli/crates/mochiflow-core/src/inspect.rs`
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-core/src/status.rs`
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `tests/conformance/golden/INDEX.md`
  - Done: `status` and `index` project their existing board and next-action contracts from one snapshot; the committed index golden remains unchanged unless a separately justified compatibility correction is approved; `state/index.json` fields and ready behavior remain compatible; counted many-spec tests prove board and inspection paths use constant-bounded external probes; shared `inspect.rs` no longer has a parallel legacy collector left beside it.
  - Stop: If migration changes existing human output, next-action precedence, archived-spec behavior, or index JSON semantics, stop and preserve the compatibility adapter before proceeding.

- [ ] T-006 [AC-03, AC-08, AC-11] Integrate engine guidance, document the API, and close verification
  - Depends on: T-005
  - Files:
    - `engine/reference/agent-context.md`
    - `engine/router.md`
    - `engine/commands/discuss.md`
    - `engine/commands/plan.md`
    - `engine/commands/build.md`
    - `engine/commands/open.md`
    - `engine/commands/update.md`
    - `engine/commands/close.md`
    - `docs/configuration.md`
    - `README.md`
    - `README.ja.md`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/VERSION`
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
    - `AGENTS.md`
    - `.kiro/steering/mochiflow.md`
    - `.kiro/agents/spec-plan-auditor.json`
    - `.kiro/agents/spec-change-reviewer.json`
  - Done: One engine reference owns the intent-versus-eligibility boundary and inspect contract usage; router and all six procedures consult deterministic eligibility without duplicating natural-language routing; public docs explain repository/detail examples, read-only/fetch behavior, result/exit semantics, and schema location; doctor recognizes `inspect` as a terminal CLI command and a regression proves context that references it produces no unknown-command warning while genuinely unknown commands still warn; engine edits are frozen, dogfood-upgraded, and adapter-checked; spec lint, targeted doctor tests, default verification, cargo-deny, schema/version gates, and a final full-diff `change-reviewer` pass all succeed through the final code-changing commit.
  - Stop: If engine integration makes `inspect` standing context, embeds API output into generated adapters, or replaces command execution procedures with CLI routing, stop and restore the agreed load-on-demand boundary.
