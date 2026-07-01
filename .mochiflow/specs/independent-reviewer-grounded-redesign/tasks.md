# Redesign the independent reviewer as a grounded adversary — Tasks

Implementation Summary: Recast the independent reviewer into a grounded-adversary stage model, expand the Kiro reviewer resources, pin it with self-conformance tests, and re-freeze the engine manifest.
risk: elevated
Critical Stop Conditions:
- The Kiro reviewer `tools` must stay exactly `["read"]`; never add write/shell or fine-grained tools (`grep`/`glob`/`bash`) — they render as "unknown" (pitfall `2026-06-28-kiro-agent-tools-are-coarse-categories`).
- Conformance assertions must be single-line `contains` or parsed JSON-array composition; never multi-line substrings (pitfall `2026-06-28-conformance-substring-line-wrap`).
- Never change `contracts/contracts.lock`, `engine/VERSION`, JSON schemas, or `tests/conformance/golden/**`; freeze may only update `engine/MANIFEST.json`.

## Defaults

- Verification: the `default` cli profile — `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`.
- Engine-source sync after editing repo-root `engine/**`: run `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check` before final verification; stage only
  intended tracked deliverables.
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing.

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03, AC-04, AC-05, AC-06, AC-07, AC-14, AC-15] Recast the reviewer agent doc into the grounded-adversary stage model
  - Depends on: none
  - Files:
    - `engine/agents/independent-reviewer.md`
  - Done: The doc defines the always-on core (S0 Grounding, S1 Internal Coherence, S2 Impact & Regression, S4 Knowledge Confrontation) plus a cross-cutting Falsification pass, with S3 Code Quality conditional on a diff (AC-01); both mode labels are retained and defined as "is S3 present" (AC-02); S0 requires grounding every current-state/change claim against code and listing ungroundable claims (AC-03); S2 derives search targets from current-state claims, changed concepts, retired/renamed terms, new or relocated responsibilities, contract/lifecycle vocabulary, declared files, surfaces, and AC nouns; searches those targets across the whole tree; falls back to distinctive spec nouns/identifiers for non-rename specs; reports hits outside declared `Files` / design scope as coverage-gap candidates; and bounds reads to `surfaces` + `Files` + hit neighborhoods (AC-04); S4 loads area-intersecting ADR decisions/pitfalls via their `INDEX.md` through the `read` capability, INDEX first, distinguishes "no ADR store" from "ADR records exist but INDEX is absent", and treats an absent index as knowledge unavailable or as a bounded directory-scan fallback rather than "no records found" (AC-05); the finding shape adds `Confidence: confirmed | predicted` with the severity caps and grounding-evidence rule, demoting ungroundable suspicions to notes (AC-06); the completion output uses stage-named sections S0/S1/S2/S3/S4/Falsification, keeps `Required Fixes` and `Remaining Notes`, and reports `S3 Code Quality` as `N/A (no implementation yet)` in plan-quality mode; the verdict rule is preserved (AC-07); S1 explicitly preserves the current Stage-1 duties — spec/AC conformance, the QA attack-coverage judgment per `reference/risk.md ## QA attack coverage`, and `session-recoverability` (AC-14); the frontmatter `references` is expanded with `reference/authoring.md` and `reference/git.md` (AC-15); the doc notes that S3 keeps its number as the mapping from the former Stage 2 and is the one conditional stage sitting between S2 and S4; and the token `session-recoverability` and the `Reviewer mode:` / `Verdict:` completion vocabulary are retained.
  - Stop: If the stage model cannot preserve the pinned `session-recoverability` judgement, the QA attack-coverage duty, or the `Verdict:` completion vocabulary that `mochiflow accept` reads, stop and re-plan.
- [x] T-002 [AC-12] Align reviewer mode vocabulary across risk / plan / review
  - Depends on: T-001
  - Files:
    - `engine/reference/risk.md`
    - `engine/commands/plan.md`
    - `engine/commands/review.md`
  - Done: The `## Review transport` mode description in `risk.md`, the pre-approval review parenthetical in `plan.md`, and the mode description in `review.md` all match the stage model (plan-quality = core with S3 `N/A`; post-implementation = core + S3) and do not contradict the reviewer doc; every by-name `Stage 1` / `Stage 2` reference in `risk.md` is updated to the new stage names — both the `same Stage 1 / Stage 2 / verdict format` phrasing in `## Review transport` and the `(Stage 1)` enforcement reference in `## QA attack coverage`; `plan.md` still contains the conformance-pinned phrase `reviewer's plan-quality mode`.
  - Stop: If aligning the vocabulary would require editing `commands/build.md` (which carries only transport/cadence, not mode labels), stop — that is out of scope.
- [x] T-003 [P] [AC-08, AC-09, AC-10] Expand the Kiro reviewer template resources
  - Depends on: T-001
  - Files:
    - `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`
  - Done: `tools` stays exactly `["read"]` (AC-08); `resources` includes `reference/risk.md`, `reference/authoring.md`, and `reference/git.md` alongside the existing `agents/independent-reviewer.md`, `reference/workflow.md`, `reference/language.md` (AC-09); no ADR `INDEX.md` path appears in `resources` (AC-10); the `description` is refreshed to the grounded-adversary framing; the JSON stays valid and renders through `adapter.rs` without new placeholders.
  - Stop: If a resource must point at a derived/gitignored or project-specific path (e.g. ADR `INDEX.md`), stop — ADR is loaded via the `read` capability, not a static resource.
- [x] T-004 [AC-11] Pin the redesign with self-conformance tests
  - Depends on: T-001, T-002, T-003
  - Files:
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
  - Done: New tests assert the single-line stage headings/labels (S0-S4 + Falsification), both mode labels, S2 target-derivation labels for non-rename specs, ADR index-unavailable behavior, the stage-named completion output including `S3 Code Quality` as `N/A (no implementation yet)`, the `Confidence: confirmed | predicted` field and severity-cap wording, the preserved verdict rule, the preserved QA attack-coverage duty label (a sibling assertion to the existing `session-recoverability` pin), the reviewer doc frontmatter `references` composition (includes `authoring.md` and `git.md`), the cross-doc vocabulary in risk/plan/review, and — by parsing the template JSON — that `tools == ["read"]`, that `resources` contains the six engine entries, and that no ADR path appears; all assertions are single-line `contains` or JSON-array composition (no multi-line substrings); `cargo test` passes and existing reviewer/adapter tests stay green.
  - Stop: If pinning a claim would require a multi-line substring, restructure the target into a single-line heading/label instead.
- [ ] T-005 [AC-13] Re-freeze and run full verification
  - Depends on: T-001, T-002, T-003, T-004
  - Files:
    - `engine/MANIFEST.json`
  - Done: `mochiflow freeze` regenerates `engine/MANIFEST.json`; `mochiflow upgrade --source engine` updates the gitignored dogfood copy from repo-root `engine/`; `mochiflow adapter generate --check` passes after the engine/template edits; `git diff --name-only` shows no change to `contracts/contracts.lock` or `engine/VERSION` (and none to schemas/golden), and only intended tracked deliverables are staged; the full `default` verify profile passes including `freeze --check`.
  - Stop: If freeze reports a change to `contracts.lock` or `engine/VERSION`, stop and investigate — this redesign must not touch frozen contracts.
