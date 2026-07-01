# Redesign the independent reviewer as a grounded adversary

## Problem

The independent reviewer reads a spec as a self-contained document, so it
structurally misses defects that only appear when the spec is read as a change
proposal against the actual repository. The `retire-patch-for-micro-spec-depth`
review surfaced several High-severity conflicts — an existing `workflow.md`
Depth-scaling micro row contradicting the spec's redefinition, a `lint.rs`
branch that cannot distinguish a micro draft from a pitch-less standard draft,
reverse-pinned conformance tests, and a recorded pitfall the change would
re-trigger — none of which the current reviewer could reach. Plan-quality mode
is defined to "judge the spec artifacts alone", and the reviewer has no repo
search, no ADR/pitfall input, and no claim-grounding step.

## Appetite

Medium. An engine-doc + adapter + conformance restructuring centered on
`engine/agents/independent-reviewer.md`, with mode-vocabulary updates in
`risk.md` / `build.md` / `plan.md` / `review.md`, a `resources` expansion in the
Kiro reviewer template, and new structure/composition-based conformance. No
schema, `contracts.lock`, or `engine/VERSION` change.

## Solution

Recast the reviewer from a document proofreader into a grounded adversary that
reads the spec as a diff against repository reality. Apply "code is the source of
truth" to the reviewer itself.

- **Stages.** An always-on core — S0 Grounding, S1 Internal coherence,
  S2 Impact & regression, S4 Knowledge confrontation, plus a cross-cutting
  Falsification pass — and a conditional S3 Code quality that runs only when a
  diff exists. The two current modes collapse into "is S3 present": plan-quality
  = core only (S3 `N/A`), post-implementation = core + S3.
- **Grounding (S0).** Verify every "current state is X" / "we change Y" claim
  against code; list the claims that could not be grounded.
- **Impact sweep (S2).** Grep the retired/renamed concept across the whole tree
  (never scope-limited) and report the difference against the tasks' declared
  `Files` as coverage-gap candidates. Bound cost by scoping the verbatim reads to
  the spec's `surfaces` + declared `Files` + hit neighborhoods.
- **Knowledge confrontation (S4).** Load area-intersecting ADR decisions /
  pitfalls (INDEX first) and confront the spec with them.
- **Findings.** Add `Confidence: confirmed | predicted`. `confirmed` (verified
  in code) may be High/Critical and can drive `fail`; `predicted` (avoidable
  depending on implementation) is capped at Medium and never alone causes
  `fail`. Both require grounding evidence (`path:line` + the observed fact);
  anything unprovable drops to a Remaining Notes / unverified list.
- **Permissions.** Keep Kiro `tools: ["read"]` (which already covers file read +
  directory + search); never add write/shell. Expand the Kiro template
  `resources` to include `risk.md`, `authoring.md`, `git.md`, and the ADR
  decisions/pitfalls `INDEX.md` as the grounding + knowledge entry points.
- **Self-conformance.** Pin the redesign by single-line headings/labels and by
  composition (the template's `tools` / `resources` arrays and the reviewer's
  frontmatter `references`), never by multi-line substring assertions.

## Rabbit Holes

- Do not try to test the reviewer's behavior with deterministic fixtures; it is
  an LLM prompt, not runnable logic.
- Do not scope-limit the impact sweep to make it cheaper — a limited sweep
  cannot detect the most important defect class (files the spec forgot to touch).
- Do not add fine-grained Kiro tools (`grep` / `glob` / `bash`); they are not
  categories and render as "unknown".

## No-gos

- No write/shell capability for the reviewer; it stays strictly read-only.
- No schema / `contracts.lock` / `engine/VERSION` change.
- No conversation history as reviewer input.
- No new multi-line-substring conformance assertions (they re-trigger pitfall
  `2026-06-28-conformance-substring-line-wrap`).

## Alternatives Considered

- Keep the two exclusive modes and bolt S0/S2/S4 onto plan-quality — rejected:
  double-manages the mode split and keeps the "spec alone" blind spot.
- Always run every stage regardless of diff — rejected: forces S3 `N/A` noise
  when there is no code.
- Scope the impact sweep to `surfaces` + `Files` — rejected: makes coverage-gap
  detection impossible.
- Allow `predicted` findings to reach High — rejected: over-blocks at plan time
  for implementation-avoidable conflicts.
- Pin conformance with multi-line substrings — rejected: re-triggers the
  recorded line-wrap pitfall.

## Open Questions

- None — ready for plan.
