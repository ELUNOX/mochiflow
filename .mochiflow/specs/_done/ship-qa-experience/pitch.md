# Unify QA experience in ship: single-source scenarios, round-trip protocol, PR Testing section

## Problem

Ship's human QA flow has no structured round-trip between the human and the
agent. Four symptoms surface together:

1. When human QA returns FAIL, there is no defined rework path — the stop
   condition prevents `status: done` but offers no guidance to fix and retry.
2. The PR Feedback Loop exists in ship.md but has no router trigger, so users
   cannot discover or invoke it.
3. QA result collection is undefined — humans do not know what format to respond
   in, leading to ambiguous PASS/FAIL interpretations.
4. QA information never reaches PR reviewers — `qa-instructions.md` is
   ephemeral/gitignored, and the PR body has no testing section.

The root causes are two:
- No structured protocol for the QA round-trip (ask → respond → record → retry).
- The QA "source of truth" is split across spec.md, qa-instructions.md, and the
  AC Matrix, with no view reaching reviewers.

## Appetite

Medium — engine documentation changes across ship.md, router.md, workflow.md,
authoring.md, two templates, and three conformance tests. No Rust library logic
changes. One elevated-risk design spec, estimating 1–2 sessions.

## Solution

### 1. Single QA source: spec.md QA Scenarios

Consolidate all QA information into spec.md's QA Scenarios section. Each
scenario carries: name, scope, type (automated/human-operated/visual), steps,
and expected result. This replaces `qa-instructions.md` as the authoritative
definition of what to test and how.

### 2. QA round-trip protocol (ship-internal)

Define a structured loop inside ship's Acceptance section:

1. Present QA items as a numbered list derived from spec.md QA Scenarios
   (scenario name + steps + expected result).
2. Human responds in conversation language (free-form intent: "OK", "NG + reason",
   "all good", etc.). These are examples, not a fixed vocabulary.
3. AI interprets intent and records the canonical AC Matrix token
   (`人間確認済み` / `FAIL` / `対象外（<reason>）`).
4. On any FAIL: pause (status stays `approved`), run build-equivalent fix
   (modify → verify → commit), re-present only FAIL items.
5. On all items resolved → resume ship step 4.

No status change, no spec move — lighter than PR Feedback Loop.

### 3. PR Feedback Loop discoverability

Add router triggers `{slug} feedback` / 「修正依頼」 / 「PR feedback」 so
users can invoke the existing loop without memorizing internal procedure names.

### 4. PR body `## Testing` section

Add a `## Testing` section to `templates/delivery/pr-description.md`, derived
from spec.md QA Scenarios (scenario name + steps + expected result). This is the
reviewer-facing QA view. The ephemeral `qa-instructions.md` template is removed;
its role is fully absorbed by spec.md (author/AI view) and PR `## Testing`
(reviewer view).

### 5. Remove `qa-instructions.md` template and references

Delete `engine/templates/delivery/qa-instructions.md`. Update ship.md,
workflow.md, and authoring.md to reference spec.md QA Scenarios instead. Update
`conformance.rs` tests that read the deleted template. Regenerate
`engine/MANIFEST.json` to drop the entry.

## Rabbit Holes

- Do not invent a new fixed-token vocabulary for human QA responses. The
  existing two-layer design (free-form conversation input → canonical Matrix
  token) already handles multilingual input. Adding a lookup table would
  contradict language.md's intent-interpretation model.
- Do not add CLI logic for QA state management. The round-trip protocol is
  agent-driven from ship.md, not CLI code.
- Do not over-specify the spec.md QA Scenario format to the point it becomes a
  second AC Matrix. Keep it scenario-oriented (steps + expected), not
  column-oriented.

## No-gos

- No changes to AC Matrix result tokens or lint validation logic.
- No changes to language.md Stable Identifiers.
- No CLI Rust code changes (except conformance test updates referencing the
  deleted template file).
- No changes to the plan.md or build.md procedures.
- No new ephemeral QA file to replace `qa-instructions.md`.

## Alternatives Considered

- **Keep `qa-instructions.md` as optional** — avoids test/link churn but
  preserves the source-of-truth split. Rejected because the root cause (split
  sources) stays unresolved; reviewers still have no QA view unless the PR body
  gains a Testing section, at which point the worksheet is redundant.
- **Split into 4 separate specs** — each gap is individually small, but they
  share the same files (ship.md, workflow.md, templates). Parallel edits would
  conflict and multiply review overhead. Rejected.
- **Define a canonical response-token dictionary for QA** — contradicts
  language.md's free-form intent model and breaks for unsupported locales.
  Rejected.

## Open Questions

- None — ready for plan.
