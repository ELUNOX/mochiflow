# Redesign the independent reviewer as a grounded adversary — Design

## Design Decisions

- **D1 — Grounded-adversary framing.** The reviewer's first duty shifts from
  "internal spec coherence" to "ground the spec's claims in code and adversarially
  find what the change breaks in reality". This applies mochiflow's
  code-is-source-of-truth rule to the reviewer. Rationale: the `retire-patch`
  review proved that a spec-alone read is structurally blind to out-of-spec
  conflicts.

- **D2 — Stage skeleton.** Always-on core: **S0 Grounding**, **S1 Internal
  Coherence**, **S2 Impact & Regression**, **S4 Knowledge Confrontation**, plus a
  cross-cutting **Falsification** pass. **S3 Code Quality** is conditional on an
  implementation diff existing. Mapping from the current doc: today's *Stage 1*
  (spec conformance + QA attack coverage + session-recoverability) becomes **S1**;
  today's *Stage 2* (code quality) becomes **S3**; S0/S2/S4/Falsification are new
  duties. The recast must **carry S1's existing duties forward, not drop them** —
  AC-14 makes the QA-attack-coverage and session-recoverability judgments explicit
  and pinned so the rename cannot silently regress a decision-backed
  responsibility (`2026-06-25-qa-attack-matrix`).

- **D3 — Modes collapse to "is S3 present".** The two public labels stay:
  `plan-quality mode` = core with S3 reported `N/A (no implementation yet)`;
  `post-implementation mode` = core + S3. Rationale: `commands/plan.md` is
  conformance-pinned on the exact phrase `reviewer's plan-quality mode`
  (`plan_offers_pre_approval_review_before_confirm_for_elevated`), and keeping the
  labels preserves user-facing continuity while removing the mutually-exclusive
  branch logic.

- **D4 — Confidence axis + evidence discipline.** Findings gain
  `Confidence: confirmed | predicted`. `confirmed` (verified in code) may be
  High/Critical and can drive `fail`; `predicted` (implementation-avoidable) is
  capped at Medium and never alone causes `fail`. Both require grounding evidence
  (`path:line` + observed fact); an ungroundable suspicion drops to an unverified
  note. Rationale: lets the reviewer raise grounded conflicts hard while not
  over-blocking at plan time for defects a good implementation can still avoid.

- **D5 — Impact sweep is un-scoped in search, bounded in reads.** S2 derives
  search targets from the spec's current-state claims, changed concepts,
  retired/renamed terms, new or relocated responsibilities, contract/lifecycle
  vocabulary, declared files, surfaces, and AC nouns. It greps those targets
  across the whole tree (a scoped sweep cannot detect the most important defect
  class — files the spec forgot to touch) and reports hits outside declared
  `Files` / design scope as coverage-gap candidates. For additive or otherwise
  non-rename specs with no obvious target, S2 falls back to the most distinctive
  nouns/identifiers from the spec instead of skipping the stage. It still bounds
  *verbatim reads* to `surfaces` + declared `Files` + hit neighborhoods to keep
  cost tractable.

- **D6 — ADR via `read` capability, not static resource** (refinement from the
  pitch's literal wording). The pitch proposed adding ADR `INDEX.md` to the
  template `resources`. Grounding showed `INDEX.md` is a **derived, gitignored
  cache** (pitfall `2026-06-27-index-md-gitignored-derived-cache`) and the
  reviewer template is generated for arbitrary consuming projects where the ADR
  store may be empty or the index absent. `adapter.rs` renders the template
  verbatim without validating resource existence, but a *static* `resources`
  entry that points at a missing/derived file is fragile at Kiro load time and
  couples the reviewer to a regenerable cache. Decision: keep `resources` limited
  to always-present engine reference files, and have S4 load ADR through the
  reviewer's existing `read` capability (INDEX first, then area-intersecting
  records) — which matches how the main agent already loads ADR on demand and
  degrades gracefully when the store is empty. If the store exists but the
  generated `INDEX.md` is absent, S4 must not claim "no records found"; it reports
  the index unavailable and either enumerates ADR records through the read
  capability when the runtime supports directory/search, or records an
  unverified knowledge-unavailable note. S4 intent is preserved; only the
  mechanism changes.

- **D7 — `resources` expansion (engine files only).** Add
  `reference/risk.md` (QA attack coverage, Review transport, design.md-required
  condition), `reference/authoring.md` (spec/design/tasks authoring rules,
  session-recoverability, SSOT), and `reference/git.md` (branch/commit/fold
  conventions the grounding + knowledge stages reason about) to the existing
  `agents/independent-reviewer.md`, `reference/workflow.md`,
  `reference/language.md`. All six are guaranteed-present engine files. The
  reviewer doc's frontmatter `references` is expanded to match
  (`+ authoring.md, git.md`) so declared dependencies track actual use (AC-15).

- **D8 — `tools` stays `["read"]`.** Kiro agent `tools` are coarse categories,
  not fine tool names (pitfall `2026-06-28-kiro-agent-tools-are-coarse-categories`);
  the `read` category already covers file read + directory listing + search, so
  S0/S2/S4 need no new capability. Adding `grep`/`glob`/`bash` would render as
  "unknown". No write/shell — the reviewer stays strictly read-only.

- **D9 — Self-conformance via single-line + composition assertions.** New
  conformance tests assert single-line stage headings/labels and the parsed
  `tools`/`resources` arrays (composition), never multi-line substrings, to avoid
  re-triggering pitfall `2026-06-28-conformance-substring-line-wrap`.

- **D10 — `build.md` unchanged for mode vocabulary.** `commands/build.md`
  references only the reviewer transport and risk cadence, not the mode labels,
  so it is out of scope for the vocabulary update (a tightening from the pitch's
  Appetite line). The transport contract in `risk.md ## Review transport` is not
  changed by this redesign.

## Architecture

Files changed and their role:

| File | Change | ACs |
| --- | --- | --- |
| `engine/agents/independent-reviewer.md` | Recast into S0-S4 + Falsification stage model; redefine modes; add `Confidence` finding field + evidence rule; preserve S1's QA-attack-coverage + session-recoverability duties; expand frontmatter `references`; S4 reads ADR via `read` | AC-01..AC-07, AC-14, AC-15 |
| `engine/reference/risk.md` | Update `## Review transport` mode description to the stage model | AC-12 |
| `engine/commands/plan.md` | Update pre-approval review parenthetical to the stage model; retain pinned phrase `reviewer's plan-quality mode` | AC-12 |
| `engine/commands/review.md` | Update mode description to the stage model | AC-12 |
| `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl` | Expand `resources` (+risk, +authoring, +git); keep `tools:["read"]`; refresh `description` | AC-08, AC-09, AC-10 |
| `cli/crates/mochiflow-cli/tests/conformance.rs` | Add self-conformance tests (single-line + composition) | AC-11 (verifies AC-01..AC-10, AC-12, AC-14, AC-15) |
| `engine/MANIFEST.json` | Regenerated by `mochiflow freeze` (hashes of edited engine files) | AC-13 |

The gitignored dogfood copy `.mochiflow/engine/**` and the generated
`.kiro/agents/spec-independent-reviewer.json` are **not** tracked deliverables;
they regenerate via `upgrade` / `adapter generate` and are out of scope.

## Data Model / Interfaces

Reviewer contract (signature-level; prose lives in the doc):

- **Stages:** `S0 Grounding`, `S1 Internal Coherence`, `S2 Impact & Regression`,
  `S3 Code Quality` (conditional), `S4 Knowledge Confrontation`, `Falsification`
  (cross-cutting). Each is a single-line `##`/`###` heading for pinning.
- **Modes:** `plan-quality mode` (core, S3 `N/A`), `post-implementation mode`
  (core + S3).
- **Finding shape** (extends the current shape): `Severity`, `Type`,
  `Confidence: confirmed | predicted`, `Location: path:line`, `Related AC/NFR`,
  `Expected`, `Actual`, `Why it matters`, `Required fix`.
- **Verdict:** `fail` (any Critical/High) | `pass-with-comments` (Medium/Low
  only) | `pass` (clean). Completion output keeps
  `Reviewer mode: delegated | inline` and `Verdict: ...` so `mochiflow accept` /
  lint continue to read the recorded verdict.
- **Kiro template composition:** `tools == ["read"]`;
  `resources == [independent-reviewer.md, workflow.md, language.md, risk.md,
  authoring.md, git.md]` (order per template); no ADR paths.
- **Completion output:** replace the current `Stage 1: Spec Conformance` /
  `Stage 2: Code Quality` sections with stage-named sections:
  `S0 Grounding`, `S1 Internal Coherence`, `S2 Impact & Regression`,
  `S3 Code Quality`, `S4 Knowledge Confrontation`, and `Falsification`. In
  plan-quality mode, `S3 Code Quality` is present with `N/A (no implementation
  yet)`, not omitted. Every finding in any stage section includes the extended
  finding shape with `Confidence: confirmed | predicted`; `Required Fixes` and
  `Remaining Notes` stay as the roll-up sections.

## Error Handling

- No ADR store in a consuming project: S4 reports "no ADR store found" / "no
  area-intersecting records found" and continues; generation and loading are
  unaffected because ADR is not a static resource (D6, AC-10).
- ADR records exist but the derived `INDEX.md` is absent: S4 reports the index as
  unavailable and either performs a bounded read-only directory scan through the
  read capability, or records an unverified knowledge-unavailable note when it
  cannot enumerate records (D6, AC-05, AC-10).
- Ungroundable finding: demoted to an unverified note (D4, AC-06) rather than a
  blocking verdict input.
- A pinned custom review `model`: preserved on regeneration by existing
  `preserve_kiro_agent_model` logic; the expanded `resources` must not change that
  path (QA-03).

## Test Strategy

- **Additive conformance tests** (`conformance.rs`, AC-11) pin: the S0-S4 +
  Falsification single-line headings (AC-01); both mode labels (AC-02);
  grounding / impact-sweep target derivation / knowledge-confrontation
  single-line labels (AC-03..AC-05); the `Confidence: confirmed | predicted`
  field and severity-cap wording (AC-06); the stage-named completion output with
  `S3 Code Quality` reported `N/A (no implementation yet)` in plan-quality mode;
  the preserved verdict rule (AC-07); the preserved
  QA-attack-coverage duty label — a sibling assertion to the existing
  `session-recoverability` pin (AC-14); the reviewer doc frontmatter `references`
  composition including `authoring.md` and `git.md` (AC-15); the parsed template
  `tools` array equals `["read"]` (AC-08); the parsed `resources` array contains
  the six engine entries and no ADR path (AC-09, AC-10); cross-doc vocab in
  risk/plan/review (AC-12). Assertions are single-line `contains` or JSON-array
  composition; no multi-line substrings.
- **Reuse existing behavioral tests** for generation/upgrade/model-preservation
  (`behavioral_upgrade_*`, `kiro_agent_json_matches_reviewer_only`,
  `removed_kiro_worker_is_deprecated_not_model_preserved`) — they must stay green
  with the expanded `resources` (QA-02, QA-03).
- **No deterministic behavioral fixture** for the reviewer's judgement: it is an
  LLM prompt. QA-01 (adversarial replay) and QA-06 (cost-bounding) are AI-observed
  doc reviews, not automated tests.
- **`freeze --check`** is part of the `default` profile and CI; running
  `mochiflow freeze` after the engine edits regenerates `engine/MANIFEST.json`
  and keeps the gate green (AC-13). `contracts.lock` hashes only
  `contracts/*.json` + `tests/conformance/golden/**` (unchanged) and `VERSION`
  syncs from `cli/Cargo.toml` (unchanged), so neither is touched.

Session-recoverability note (plan authoring): the **stage-model contract** in D2
is the shared contract that T-002/T-003/T-004 depend on; it is written here so a
resuming session recovers it without conversation memory. `conformance.rs` and
`engine/agents/independent-reviewer.md` are each edited by exactly one task, so no
shared-file consistency handoff is required.

## Review Results

- Reviewer mode: inline
- Verdict: pass
- Reviewed: full `origin/main...HEAD` diff after T-001..T-005 and follow-up
  reviewer-finding fixes.
- Note: delegated dispatch was attempted first, but the runtime failed with
  workspace credit exhaustion; inline fallback was used per
  `reference/risk.md ## Review transport`.
