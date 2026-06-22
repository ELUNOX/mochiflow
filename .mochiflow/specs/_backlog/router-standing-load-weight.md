---
slug: "router-standing-load-weight"
title: "Reduce router/verb standing-load weight in CLI sessions"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "discuss"
created: "2026-06-22"
updated: "2026-06-22"
---

# Reduce router/verb standing-load weight in CLI sessions

## Signal

In kiro-cli, the router (and the steering/agent prompt that embeds it) is loaded
as an always-on standing instruction. Even in ordinary conversation where the
user does not intend a mochiflow verb, the agent carries the routing-decision
cost, and entering a verb pulls in several reference files at once
(router + plan + workflow + risk were all read before plan could start in the
2026-06-22 session). The router design is deliberately conservative about
activation, but the *load footprint* of reaching a decision is heavier than the
"keep router thin, lazy-load the rest" intent implies.

## Why It Matters

Standing-load weight is paid on every turn, not just when a verb runs. A heavier
baseline context reduces room for actual implementation work and makes the
"do not activate without explicit intent" principle more expensive to honor than
it should be. This is a usability/efficiency concern specific to CLI adapters
where the router is a persistent system instruction rather than an on-demand IDE
panel.

## Evidence

- `engine/router.md` is listed as loaded directly by adapter entrypoints
  (e.g. the kiro `spec-builder` agent `prompt`), and its frontmatter references
  8 sibling files (commands/*.md + reference/*.md).
- Routing Principle 1 and Decision Flow already aim for minimal activation, and
  step 8 says to lazy-load `commands/{verb}.md` and its references only once
  committed — but in practice reaching a plan decision required reading
  router + plan + workflow + risk together.
- The kiro adapter agent config (`spec-builder.json`) also lists ~30 `resources`
  files, all attached to the agent.

## Open Questions

- Can the always-loaded surface be reduced to a thin router core, with verb
  procedures and references strictly lazy-loaded at activation time?
- Is the large `resources` list in the kiro agent config necessary at all times,
  or can it be scoped per verb?
- Is there a measurable token/latency cost worth quantifying before changing the
  load strategy?
