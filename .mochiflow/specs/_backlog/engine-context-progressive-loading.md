---
slug: "engine-context-progressive-loading"
title: "Define a compact standing router and verb-scoped engine loading"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_phase: "discussion"
created: "2026-06-30"
updated: "2026-06-30"
---

# Define a compact standing router and verb-scoped engine loading

## Signal

MochiFlow's engine guidance is increasingly split across router, command,
reference, template, reviewer, and adapter documents. The documents are already
layered, but generated adapter instructions still present router, verb
procedures, and cross-cutting references side by side. Cautious agents can read
too broadly before they know which workflow path applies.

## Why It Matters

MochiFlow depends on procedural safety: routing, stop conditions, risk cadence,
git discipline, artifact boundaries, ADR usage, and approval gates must remain
reliable. The right fix is not to weaken those rules or create a second router.
The right fix is to make the standing layer compact and authoritative, then
load detailed command and reference rules only after routing has selected the
workflow path.

The best v1 outcome is a clear loading contract:

- standing layer: constitution, foundational context, project config, and one
  compact `router.md`;
- route decision: `router.md` contains only the information needed to decide
  whether to stay in normal conversation, run patch, run a lifecycle verb, or
  run a non-phase command;
- verb-scoped detail: after a command is selected, read only
  `commands/{verb}.md` and that command's frontmatter `references`;
- historical context: load ADR stores on demand by index first, then only
  relevant active records;
- adapter output: generated AGENTS / Claude / Copilot / Kiro instructions make
  the always-loaded vs lazy-loaded boundary explicit.

## Evidence

- `engine/router.md` is already the intended standing instruction and already
  says to consult `commands/{verb}.md` and its frontmatter `references` only
  once committed to a verb or non-phase command.
- `engine/adapters/agents/AGENTS.md.tpl`,
  `engine/adapters/claude-code/CLAUDE.md.tpl`, and
  `engine/adapters/copilot/copilot-instructions.md.tpl` list router,
  procedures, and cross-cutting references together. That reads more like a
  catalog to eagerly inspect than a strict loading protocol.
- `engine/adapters/kiro/steering/mochiflow.md.tpl` is always-on steering and
  file-references the router, constitution, and context. That is appropriate,
  but the rest of the file should be especially careful not to imply that
  command/reference files are always loaded.
- Current engine word counts are large enough that eager loading is material:
  router, commands, references, and reviewer prompt are roughly twenty thousand
  words combined.
- The engine is already naturally layered:
  - `router.md` for routing;
  - `commands/*.md` for phase/non-phase procedures;
  - `reference/*.md` for shared policy;
  - `templates/*` for authored artifacts;
  - `agents/independent-reviewer.md` for delegated read-only review.

## Open Questions

- Can `engine/router.md` itself be compacted enough to be the standing route
  card, avoiding a separate `router.card.md` that could drift?
- What exact rule set must remain in `router.md` for safe initial routing:
  activation strength, patch eligibility handoff, active spec resolution,
  state/intent conflict handling, review transport boundary, and next-file
  loading?
- Which details should move out of `router.md` into command procedures or
  references without changing behavior?
- Should command frontmatter remain file-level in v1, with section-level
  references explicitly deferred until the supported agent/tool surfaces can
  read sections consistently?
- What adapter wording makes the loading boundary obvious across AGENTS,
  Claude, Copilot, and Kiro without making adapter output verbose?
- What tests should prove parity: explicit command routing, natural-language
  hint handling, `{slug} discuss` backlog promotion, `{slug} plan` rejecting raw
  backlog seeds, patch eligibility routing, review trigger routing, and Kiro
  always-on steering contents?
- Should a later version add `guide --context-budget` or `context audit` for
  measurement, after the standing/lazy loading contract is stable?

## Suggested v1 Boundary

- In: compact `router.md`; adapter wording that separates always-loaded from
  lazy-loaded files; tests or golden assertions for adapter output and key
  routing invariants.
- Out: a new generated `router.card.md`; section-level reference anchors;
  context-budget CLI commands; semantic prompt-size optimization beyond the
  engine loading contract.
