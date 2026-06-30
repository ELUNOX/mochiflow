---
slug: "trigger-routing-redesign"
title: "Redesign verb activation: explicit control + description-based intent + state-driven routing"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_phase: "discuss"
created: "2026-06-30"
updated: "2026-06-30"
---

# Redesign verb activation: explicit control + description-based intent + state-driven routing

## Signal

The current activation model in `engine/router.md` plus the per-command
`triggers` / `trigger_patterns` frontmatter conflates two separate concerns —
*whether/when to activate* (control) and *which verb* (selection) — across three
overlapping mechanisms: explicit commands (`mochiflow-<verb>`), slug patterns
(`{slug} <verb>`), and hand-curated natural-language keyword lists
(`実装して`, `進めて`, `完了`, ...). This makes routing brittle and forces a
large body of prose exceptions in the router's Routing Principles.

## Why It Matters

Activation is mochiflow's front door. Keyword lists fight the LLM harness's
strength (semantic understanding) and lean on its weakness (deterministic
keyword matching is not guaranteed). Common progression words (`進めて`,
`直して`, `完了`, `merged`) collide with ordinary conversation, which is exactly
why the router needs many "do not activate unless an active spec scopes it"
caveats. The bilingual (ja/en) keyword tables are unmaintainable and always
incomplete, and natural-language triggers are invisible (no discovery surface).

## Evidence

- `engine/router.md` encodes the state-dependent meaning of triggers ("activate
  immediately only when an active spec context already scopes the verb")
  entirely in prose Routing Principles, not in a checkable structure.
- Per-command frontmatter carries duplicated ja/en keyword lists, e.g.
  build = `[mochiflow-build, 実装して, 進めて, ビルドして]`,
  close = `[mochiflow-close, merged, マージ済み, 完了]` — high-frequency words
  that overlap normal chat.
- Three distinct "activation strengths" (explicit command / slug pattern /
  NL hint) are specified separately rather than derived from one policy.

## Proposed Direction (from industry survey)

Anchor on the two-tier model seen in Claude Code (deterministic slash commands +
description-matched, auto-invoked skills) plus an intent-classification layer,
adapted to mochiflow's lifecycle:

1. **Deterministic explicit layer (control).** Keep `mochiflow-<verb>` and
   `{slug} <verb>` as zero-false-positive escape hatches. Add a discovery
   surface (`mochiflow` with no args / `mochiflow help`) that lists the verbs
   playable from the current spec state.
2. **Description-based intent layer (convenience).** Replace `triggers` keyword
   lists with a natural-language `intent` description plus a few few-shot
   `examples` per verb; let the model semantically match instead of maintaining
   bilingual keyword tables.
3. **State x intent routing table.** Promote the prose rules to an explicit
   table keyed by spec status (none / draft / approved / accepted / in-review /
   merged). Bind progression words (`進めて`) to the verb that is *live* given
   current state rather than to a single verb — this removes their ambiguity.
4. **Single confidence-gated activation policy.** Collapse the three activation
   strengths into one: high -> activate (one-line declaration); medium ->
   one-line "Start <verb>?" confirm; low/ambiguous -> stay in conversation or
   ask one two-choice question.
5. **Lean on next-verb suggestion.** Make the post-verb numbered choice card the
   primary progression path so users advance by label/number, reducing reliance
   on trigger vocabulary.

Explicitly out of scope: embedding / vector semantic-router infrastructure — the
verb set is ~10 and the harness is already an LLM, so a single model
classification pass suffices (no new infra).

## Open Questions

- New frontmatter schema: replace `triggers` + `trigger_patterns` with an
  `invocation` block (`command`, `slug_form`, `intent`, `examples`,
  `requires_state`, `progression_of`)? How does `mochiflow lint` validate it?
- Where does the state x intent table live — `router.md`, a new
  `reference/routing.md`, or generated from per-command `requires_state`?
- How is the non-deterministic intent layer tested (scenario tests in the spirit
  of `workflow-state-transition-scenario-tests`)?
- Migration: do adapters (kiro / agents) need regeneration when `triggers`
  frontmatter is removed?
- Should natural-language triggers be retained at all, or fully replaced by
  description matching + the discovery surface?
