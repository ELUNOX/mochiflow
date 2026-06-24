# Kiro adapter: drop dedicated agent, single always-on steering, delegate permissions — Tasks

Implementation Summary: Replace the Kiro dedicated agent + verb steering with one always-on steering file plus the read-only reviewer, delegate permissions to permissions.yaml, and self-heal deprecated outputs.
risk: elevated
Critical Stop Conditions:
- A `contracts/*.json` schema change becomes necessary (would require VERSION + lock bump) — stop and re-plan.
- Self-heal would delete a marker-less / user-authored `.kiro` file — stop; only marker-bearing files may be removed.
- `mochiflow freeze` cannot restore engine integrity by re-running — stop.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01] Rewrite Kiro manifest + templates to the two-file surface
  - Depends on: none
  - Files: `engine/adapters/kiro/manifest.toml`, `engine/adapters/kiro/steering/mochiflow.md.tpl`, `engine/adapters/kiro/agents/spec-builder.json.tpl`, `engine/adapters/kiro/steering/spec.md.tpl`, `engine/adapters/kiro/steering/spec-discuss.md.tpl`, `engine/adapters/kiro/steering/spec-plan.md.tpl`, `engine/adapters/kiro/steering/spec-build.md.tpl`, `engine/adapters/kiro/steering/spec-ship.md.tpl`, `engine/adapters/kiro/steering/spec-patch.md.tpl`, `engine/adapters/kiro/steering/spec-review.md.tpl`, `engine/adapters/kiro/steering/spec-refresh-context.md.tpl`
  - Done: manifest `[files]` maps only `mochiflow.md` and the reviewer agent; `mochiflow.md.tpl` created; `spec-builder.json.tpl` and all `spec*.md.tpl` deleted
  - Stop: if a verb procedure would lose its load path (confirm router lazy-load covers it before deleting verb steering)
- [ ] T-002 [AC-02] Author `mochiflow.md.tpl` content (always-on + pointers + Rules)
  - Depends on: T-001
  - Files: `engine/adapters/kiro/steering/mochiflow.md.tpl`
  - Done: `inclusion: always` frontmatter; marker comment; `#[[file:]]` includes for `{{engine}}/router.md`, `{{constitution.project}}`, `{{constitution.local}}`, `{{context.product}}`, `{{context.structure}}`, `{{context.tech}}`; Rules block mirroring `AGENTS.md.tpl` (push/PR via `mochiflow pr` only)
  - Stop: if any `{{token}}` lacks a substitution in `subs()`
- [ ] T-003 [AC-03] Confirm reviewer agent is policy-free
  - Depends on: T-001
  - Files: `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`
  - Done: no `toolsSettings` key; `tools` exactly `["read","grep","glob"]`; no `subagent` trust block required
  - Stop: if the reviewer needs a tool beyond read/grep/glob to function
- [ ] T-004 [AC-04] Implement marker-gated self-heal of deprecated Kiro outputs
  - Depends on: T-001
  - Files: `cli/crates/mochiflow-core/src/adapter.rs`
  - Done: `is_kiro_agent_json` lists only the reviewer; `generate` removes deprecated kiro paths only when they contain `MARKER_PREFIX`, records them in `AdapterResult.removed`, records marker-less deprecated files in `AdapterResult.preserved`, and `cmd_adapter_generate` prints `removed:` / `preserved:` lines; marker-less files are never deleted
  - Stop: if removal logic would run for non-kiro adapters
- [ ] T-005 [AC-05] Make `--check` and `doctor` consistent with the new layout
  - Depends on: T-004
  - Files: `cli/crates/mochiflow-core/src/adapter.rs`, `cli/crates/mochiflow-core/src/doctor.rs`
  - Done: one shared deprecated-path list drives both branches; `doctor` and `adapter generate --check` (doctor reuses `generate(check)`) FAIL while un-healed marker-bearing deprecated residue is present, and clear once write-mode `mochiflow adapter generate` removes it
  - Stop: if doctor and generate could disagree on what counts as residue (they must share one list)
- [ ] T-006 [AC-06] Update tests to the new Kiro file set
  - Depends on: T-004
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`, `cli/crates/mochiflow-cli/tests/cli.rs`, `cli/crates/mochiflow-core/src/present.rs`
  - Done: assertions expect `mochiflow.md` + reviewer agent; `spec-builder.json` cases replaced; new self-heal unit test added; `cargo test --manifest-path cli/Cargo.toml` passes
  - Stop: if a test encodes behavior that contradicts the agreed design
- [ ] T-007 [AC-07] Dogfood re-freeze + vendored sync + regenerate
  - Depends on: T-001, T-002, T-003, T-004, T-005, T-006
  - Files: `engine/MANIFEST.json`, `.mochiflow/engine/**`, `.kiro/**`, `AGENTS.md`
  - Done: `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate` run; `mochiflow doctor` and `mochiflow adapter generate --check` pass; this repo's stale `.kiro` outputs healed, `release.md` untouched; no VERSION bump unless freeze/doctor require it
  - Stop: if `freeze --check`/`doctor` demand a contracts schema change (Critical Stop)
- [ ] T-008 [AC-08] Update README + Kiro docs
  - Depends on: T-007
  - Files: `README.md`, `README.ja.md`
  - Done: Kiro integration row (en + ja) describes always-on steering + read-only reviewer with permissions delegated to `permissions.yaml`; no claim of a generated dedicated agent with baked policy
  - Stop: if docs would describe behavior not implemented

<!-- AC Verification Matrix lives in spec.md ## Verification Plan / AC Matrix -->
