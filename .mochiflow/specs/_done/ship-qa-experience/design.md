# Unify QA experience in ship — Design

## Design Decisions

- **QA Scenarios as single source**: spec.md QA Scenarios table is the only
  place QA information is authored. PR `## Testing` and the conversation
  presentation are derived views. Rationale: eliminates the three-location split
  (spec.md / qa-instructions.md / AC Matrix) that caused information to not
  reach reviewers.
  Source: pitch.md agreement + language.md stable-identifier model.

- **Round-trip protocol location**: defined in ship.md's Acceptance section as a
  sub-procedure (steps 1–3 rewritten). Not a separate file or CLI command.
  Rationale: the protocol is agent behavior, not CLI logic.

- **QA Scenarios table gains `Type` column**: `Automated | Human-operated |
  Visual`. This replaces the Legend from `qa-instructions.md`. The column tells
  ship which items require human interaction and which are satisfied by the
  verification command alone.

- **PR `## Testing` derivation rule**: ship derives the section from spec.md QA
  Scenarios rows where `Type ∈ {Human-operated, Visual}` or all rows when the
  spec is trivial. Concrete steps + expected result, no internal IDs.

- **qa-instructions.md removal strategy**: delete template file and the
  ephemeral generation target (`{install_dir}/state/{slug}/qa-instructions.md` is
  no longer produced — the round-trip protocol presents QA directly in
  conversation). Update MANIFEST via `mochiflow freeze`, update three conformance
  test call sites in `tests/conformance.rs`, update prose references in ship.md /
  workflow.md / authoring.md, and remove ship.md frontmatter `artifacts:` and
  `references:` lines pointing to the deleted file. The Kiro spec-builder agent
  file list (`.kiro/agents/spec-builder.json`) and its template
  (`engine/adapters/kiro/agents/spec-builder.json.tpl`) also reference
  qa-instructions.md — remove those lines, then regenerate the agent file via
  `mochiflow adapter generate`.

- **Router triggers for PR Feedback Loop**: add `{slug} feedback` / 「修正依頼」
  / 「PR feedback」 to ship.md `trigger_patterns` and update router.md's
  handling to route these to `## PR Feedback Loop` (not full ship restart).
  Router resolves `{slug}` against `{specs_dir}/_done/{slug}/` for feedback
  patterns specifically (since the spec is archived when PR feedback arrives),
  mirroring the existing `{slug} merged` exception in router.md Decision Flow
  step 3.

- **QA round-trip response model**: the agent presents items as a numbered list
  and accepts responses either per-item or as a numbered batch. Each response is
  mapped to its QA item by number. After a rework fix, the agent re-presents
  failed items plus any previously-passed items whose implementation files were
  modified by the fix (to catch regressions).

## Architecture

No new files or modules. Changes are engine documentation edits:

| File | Change type |
| --- | --- |
| `engine/commands/ship.md` | Rewrite Acceptance steps 1–3 (round-trip + rework); add triggers; remove `qa-instructions.md` from frontmatter `artifacts:` and `references:` |
| `engine/router.md` | Add PR Feedback trigger handling note |
| `engine/reference/workflow.md` | Remove acceptance-adapter qa-instructions reference; update QA role text |
| `engine/reference/authoring.md` | Update ephemeral table and QA role split paragraph |
| `engine/templates/delivery/pr-description.md` | Add `## Testing` section |
| `engine/templates/delivery/qa-instructions.md` | DELETE |
| `engine/templates/spec/spec.standard.md` | Add `Type` column to QA table |
| `cli/crates/mochiflow-cli/tests/conformance.rs` | Remove/replace 3 call sites reading qa-instructions template |
| `.kiro/agents/spec-builder.json` | Remove qa-instructions line from inputFiles |
| `engine/adapters/kiro/agents/spec-builder.json.tpl` | Remove qa-instructions line |
| `engine/MANIFEST.json` | Regenerated via `mochiflow freeze` |
| `.mochiflow/engine/**` | Synced from source via `mochiflow upgrade --source engine` |

## Error Handling

- If spec.md lacks a QA Scenarios section, ship logs a warning and proceeds with
  verification-only acceptance (no human round-trip). This is existing behavior
  for trivial specs.
- Ambiguous human response: agent re-asks once; on continued ambiguity, record
  the response verbatim and ask for explicit pass/fail.

## Test Strategy

- `cargo test --manifest-path cli/Cargo.toml`: conformance tests must pass after
  template removal and call-site updates.
- `mochiflow freeze --check`: MANIFEST must be consistent after template deletion.
- `mochiflow doctor` + `mochiflow lint --spec ship-qa-experience`: quality gates.
- Human QA (QA-05, QA-06, QA-07): read the changed engine docs and confirm
  clarity.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Findings: 2 Low (AC-01..06 still PENDING_HUMAN — expected pre-ship; CHANGELOG historical entry mentions deleted qa-instructions.md — optional fix at next release)
- No required fixes.
