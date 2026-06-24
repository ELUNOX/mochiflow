# Kiro adapter: drop dedicated agent, single always-on steering, delegate permissions

## Background and Design Rationale

The Kiro adapter is the only adapter that ships a dedicated runtime agent
(`spec-builder.json`) with tool policy baked into `toolsSettings`
(`shell.allowedCommands`/`deniedCommands`, `write.allowedPaths`/`deniedPaths`
rendered from `config.toml [write]`, a `web_fetch.blocked` rule), a ~30-entry
`resources` list, and a `subagent` trust declaration. The `AGENTS.md` and
`CLAUDE.md` adapters instead inject a single managed instruction file into the
host's default agent and rely on prose guardrails — no baked agent, no baked
permissions.

That baked policy is duplicated and drift-prone: it must track engine reality
each release and is hashed into `engine/MANIFEST.json`, and Kiro CLI 3.0's
capability-based `permissions.yaml` (`deny > ask > allow`, per-user and outside
the repo) now overlaps it without being able to be loosened by it. Aligning
Kiro with the prose-enforcement model removes the asymmetry, shrinks the
generated surface, and lets `permissions.yaml` own permissions as the platform
intends.

Key decisions (agreed in discuss, `pitch.md`):

- Generated Kiro surface shrinks to two files: one always-on steering file plus
  the existing read-only reviewer agent. `spec-builder.json` and all per-verb
  steering are removed.
- The context layer is referenced via Kiro-native `#[[file:...]]` pointers, not
  inlined, to avoid coupling the generated steering file to `context/*.md` edits
  (which would make every `refresh-context` trip `adapter generate --check`).
- Verb procedures are not mirrored as steering files; the router's existing
  lazy `fs_read` of `commands/{verb}.md` loads them on activation.
- Permissions are fully delegated to the user's `permissions.yaml`; mochiflow
  does not write, scaffold, or doctor-nudge it. The `git push` / provider-PR
  invariant downgrades from a baked machine deny to `mochiflow.md` Rules prose,
  matching the Claude/AGENTS trust model.
- Alpha / breaking-changes-OK: no migration ceremony. Regeneration self-heals by
  removing deprecated mochiflow-managed Kiro outputs (marker-gated).

This contract surface lives in `engine/` (hashed into `engine/MANIFEST.json`),
not in `contracts/*.json`; it therefore requires `mochiflow freeze` and a
vendored-engine sync, but no `contracts.lock` / engine `VERSION` bump unless a
contracts schema changes (it does not here).

Origin: backlog seed `kiro-adapter-default-agent` (source: conversation). The
overlapping `router-standing-load-weight` seed is absorbed and removed.

## User Story

As a developer using mochiflow with Kiro, I want the generated Kiro integration
to be a thin always-on steering file plus a read-only reviewer agent with
permissions owned by my `permissions.yaml`, so that the integration matches the
Claude/AGENTS trust model and carries no duplicated, drift-prone tool policy.

## Scope

- In: the `kiro` adapter manifest and templates under `engine/adapters/kiro/**`;
  Kiro generation/detection/self-heal in `cli/crates/mochiflow-core/src/adapter.rs`;
  Kiro residue handling in `doctor`; affected tests in `cli.rs`,
  `conformance.rs`, `present.rs`; dogfood re-freeze + vendored sync + adapter
  regeneration; README/docs Kiro rows.
- Out: changes to the `agents` / `claude-code` / `copilot` adapters; changes to
  `contracts/*.json` schemas; any write to a user `permissions.yaml`; engine
  router/command/reference content (only their inclusion wiring changes).

## Edge Cases

- A project carrying a hand-authored, unmanaged steering file (e.g.
  `.kiro/steering/release.md`, no mochiflow marker): must be left untouched by
  self-heal.
- A deprecated Kiro output a user has manually edited so it no longer carries
  the marker: not removed; reported/skipped, not silently deleted.
- `mochiflow.md` already present from a prior run: regeneration updates it in
  place via the existing `.md` marker/managed handling without duplicating.
- A project that only enabled `kiro` (not `agents`): still gets a complete
  standing layer from `mochiflow.md` alone.

## Acceptance Criteria (EARS)

- AC-01: WHEN `mochiflow adapter generate` runs for the `kiro` adapter, THE
  SYSTEM SHALL produce exactly `.kiro/steering/mochiflow.md` and
  `.kiro/agents/spec-independent-reviewer.json`, and SHALL NOT produce
  `.kiro/agents/spec-builder.json` or any `.kiro/steering/spec-*.md` verb
  steering file.
- AC-02: THE SYSTEM SHALL generate `.kiro/steering/mochiflow.md` with
  `inclusion: always` frontmatter, `#[[file:...]]` pointer includes for the
  router, both constitution files, and the three context files, and a Rules
  block stating push/PR go only through `mochiflow pr`.
- AC-03: THE SYSTEM SHALL generate Kiro agent JSON with no `toolsSettings` key,
  and the reviewer agent SHALL retain `tools` of exactly `read`, `grep`, `glob`.
- AC-04: WHEN `mochiflow adapter generate` runs in a project that still contains
  deprecated mochiflow-managed Kiro outputs (`spec-builder.json`, `spec*.md`
  steering), THE SYSTEM SHALL remove only those files that carry the mochiflow
  marker, report each removal, and leave marker-less files unchanged and
  reported as preserved.
- AC-05: WHEN `mochiflow adapter generate --check` and `mochiflow doctor` run
  after generation on the new layout, THE SYSTEM SHALL report no drift and no
  Kiro residue failure.
- AC-06: THE SYSTEM SHALL update Kiro detection (`is_kiro_agent_json`) and the
  tests referencing the old Kiro file set so that `cargo test` passes.
- AC-07: WHEN `engine/adapters/kiro/**` is edited, THE SYSTEM SHALL keep engine
  integrity such that `mochiflow freeze` (MANIFEST regenerated) and the vendored
  sync leave `mochiflow doctor` and `mochiflow adapter generate --check` passing.
- AC-08: THE SYSTEM SHALL update the README Kiro integration row (en and ja) and
  Kiro integration docs to describe the always-on steering + read-only reviewer
  model with permissions delegated to `permissions.yaml`.

## QA Scenarios

| QA | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | cli | Automated | Run `cargo test --manifest-path cli/Cargo.toml` | Pass, incl. updated Kiro file-set assertions |
| QA-02 | cli | Automated | In a materialized kiro project, run `mochiflow adapter generate` then `adapter generate --check` | Two files generated; `--check` reports no drift |
| QA-03 | cli | AI-observed | Inspect generated `.kiro/steering/mochiflow.md` | `inclusion: always`, `#[[file:]]` pointers for router/constitution/context, Rules block present; no inlined context |
| QA-04 | cli | AI-observed | Inspect generated Kiro agent JSON | No `toolsSettings`; reviewer `tools` exactly read/grep/glob |
| QA-05 | cli | Automated | Seed a project with old `spec-builder.json` + `spec*.md` (markered) and an unmanaged `release.md`, run generate | Markered deprecated files removed and reported (`removed:`); `release.md` untouched and reported as preserved (`preserved:`) |
| QA-06 | cli | AI-observed | After dogfood `freeze` + vendored sync + regenerate, run `mochiflow doctor` | Doctor passes; no MANIFEST drift, no residue |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.
- An independent-reviewer verdict (`pass` / `pass-with-comments`) is recorded (risk: elevated).

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `cargo test` file-set assertions + QA-02 | `engine/adapters/kiro/manifest.toml`, `cli/.../adapter.rs` | UNVERIFIED | | |
| AC-02 | cli | AI-observed | QA-03 | `engine/adapters/kiro/steering/mochiflow.md.tpl` | UNVERIFIED | | |
| AC-03 | cli | AI-observed | QA-04 | `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl` | UNVERIFIED | | |
| AC-04 | cli | automated | QA-05 | `cli/crates/mochiflow-core/src/adapter.rs` | UNVERIFIED | | |
| AC-05 | cli | automated | QA-02, QA-06 | `cli/crates/mochiflow-core/src/{adapter.rs,doctor.rs}` | UNVERIFIED | | |
| AC-06 | cli | automated | QA-01 | `cli/crates/mochiflow-core/src/adapter.rs`, `cli/crates/mochiflow-cli/tests/{cli.rs,conformance.rs}`, `cli/crates/mochiflow-core/src/present.rs` | UNVERIFIED | | |
| AC-07 | cli | automated | QA-06 + `mochiflow freeze --check` | `engine/MANIFEST.json`, `.mochiflow/engine/**` | UNVERIFIED | | |
| AC-08 | cli | AI-observed | docs review | `README.md`, `README.ja.md`, Kiro docs | UNVERIFIED | | |
