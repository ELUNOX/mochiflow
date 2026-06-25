# Lint: detect template residue and multi-line EARS ACs

## Background and Design Rationale

`mochiflow lint` is the mechanical gate that plan uses before implementation
approval. The plan and authoring guidance already require agents to remove
unfinished template residue before approval, but the CLI does not enforce that
rule. As a result, generated template artifacts can leave placeholders,
template-only comments, example rows, or bare `TBD` text in durable spec files
until human review catches them.

The same lint path currently checks EARS wording only on the line that contains
an AC ID. That produces a false warning for readable multi-line acceptance
criteria where the AC declaration line introduces the condition and the EARS
keyword appears on a continuation line.

The chosen approach is to harden the existing lint module rather than introduce
a full Markdown parser. The implementation should stay close to the current
line-oriented checks, but with small block-aware helpers for AC text and
Markdown code spans. Residue detection should fail unfinished authored spec
documents, while avoiding obvious false positives in fenced code blocks and
inline code examples.

This work came from the `lint-residue-and-multiline-ears` backlog seed.

## User Story

As a MochiFlow author, I want `mochiflow lint` to catch unfinished template
residue and correctly understand multi-line acceptance criteria, so that the
approval gate is stricter without forcing awkward spec wording.

## Scope

- In:
  - Add lint failures for unfinished template residue in expanded spec
    documents.
  - Update EARS warning detection to inspect the whole AC block.
  - Add conformance tests for residue failures, code-example false-positive
    avoidance, build-owned matrix tokens, and multi-line AC blocks.
  - Update relevant authoring or command guidance when behavior changes.
- Out:
  - Changing the `spec.yaml` schema or spec lifecycle states.
  - Changing canonical AC Matrix result tokens.
  - Rewriting lint around a full Markdown AST.
  - Applying residue checks to pitch-only draft specs before plan expansion.

## Edge Cases

- Placeholder-like text inside fenced code blocks or inline code spans should
  not fail residue checks.
- Legitimate AC Matrix build placeholders such as `UNVERIFIED` and
  `PENDING_HUMAN` should remain valid where the matrix allows them.
- A multi-line AC block ends before the next AC declaration or the next Markdown
  section heading.
- Residue checks should report actionable file-level failures without flooding
  output with duplicate messages for the same residue class.

## Acceptance Criteria (EARS)

- AC-01: THE SYSTEM SHALL fail `mochiflow lint` when expanded spec documents
  contain unfinished template residue such as unfilled template placeholders,
  template-only HTML comments, example-only rows, or bare `TBD` text.
- AC-02: WHEN residue-like text appears inside fenced code blocks or inline code
  spans, THE SYSTEM SHALL avoid treating that text as unfinished template
  residue.
- AC-03: WHEN an acceptance criterion spans multiple lines, THE SYSTEM SHALL
  detect EARS keywords across the whole AC block instead of only the AC
  declaration line.
- AC-04: THE SYSTEM SHALL continue accepting build-owned AC Matrix provisional
  result tokens where they are currently valid and SHALL NOT classify them as
  template residue.
- AC-05: THE SYSTEM SHALL keep authoring guidance and conformance coverage in
  sync with the new lint behavior.

## QA Scenarios

| QA | Scope | Type | Steps | Expected result |
| --- | --- | --- | --- | --- |
| QA-01 | cli | Automated | Run `cargo test --manifest-path cli/Cargo.toml` after adding focused lint cases. | Tests covering residue failures, residue false-positive avoidance, and multi-line EARS pass. |
| QA-02 | cli | Automated | Run the default CLI verification profile. | The full CLI test, format, clippy, and freeze checks pass. |
| QA-03 | cli | Automated | Run `mochiflow lint --spec lint-residue-and-multiline-ears` after planning and build updates. | The spec remains lint-clean while documenting residue patterns in code spans. |

## Completion Conditions

- Every AC appears in the AC Verification Matrix with a done-eligible result
  token (`PASS`, `CONFIRMED`, or `N/A: <reason>`).
- Verification commands and results are recorded.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | cli | automated | `cargo test --manifest-path cli/Cargo.toml` focused lint cases | `cli/crates/mochiflow-core/src/lint.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` passed; `lint_fails_on_template_residue_classes` covers placeholders, HTML comments, example rows, and bare `TBD` |  |
| AC-02 | cli | automated | `cargo test --manifest-path cli/Cargo.toml` false-positive cases | `cli/crates/mochiflow-core/src/lint.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` passed; `lint_ignores_template_like_text_in_code` covers fenced and inline code |  |
| AC-03 | cli | automated | `cargo test --manifest-path cli/Cargo.toml` multi-line AC case | `cli/crates/mochiflow-core/src/lint.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` passed; multi-line EARS tests cover positive, next-AC boundary, and next-heading boundary cases |  |
| AC-04 | cli | automated | Existing and new AC Matrix token cases in `cargo test --manifest-path cli/Cargo.toml` | `cli/crates/mochiflow-core/src/lint.rs`; `cli/crates/mochiflow-cli/tests/conformance.rs` | PASS | `cargo test --manifest-path cli/Cargo.toml` passed; existing AC Matrix token tests continue to pass |  |
| AC-05 | cli | automated | `cargo test --manifest-path cli/Cargo.toml`; `cargo run --manifest-path cli/Cargo.toml -- freeze --check`; `mochiflow lint --spec lint-residue-and-multiline-ears` | `engine/commands/plan.md`; `engine/reference/authoring.md`; generated engine artifacts if changed | PASS | Default CLI verification passed; `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate --check`, and spec lint passed |  |
