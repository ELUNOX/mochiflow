# Define a compact standing router and verb-scoped engine loading

## Problem

MochiFlow's engine guidance has grown across router, command, reference,
template, reviewer, and adapter documents. Those documents are naturally
layered, but generated adapter instructions still present router, verb
procedures, and cross-cutting references side by side. That can nudge cautious
agents into broad eager reads before they know which workflow path applies.

This raises token cost, latency, and procedural risk: routing, stop conditions,
risk cadence, git discipline, artifact boundaries, ADR usage, and approval gates
can become harder to see when too much unrelated guidance is loaded at once.

## Appetite

This is worth a focused elevated-risk refactor of the engine/adapters loading
contract, not a new CLI subsystem. The target is to preserve behavior while
making the always-loaded layer smaller and the lazy-loading boundary explicit.

## Solution

Use `engine/router.md` itself as the compact standing router. Do not introduce a
second generated `router.card.md` in v1. The router should contain only the
initial routing contract needed to decide whether to stay in normal
conversation, use patch, run a lifecycle verb, or run a non-phase command. Once
a command is selected, the agent reads `commands/{verb}.md` and that command's
frontmatter `references`.

Adjust generated adapter instructions so they clearly separate:

- always-loaded inputs: constitution, foundational context, project config, and
  the compact router;
- lazy-loaded inputs: verb procedures, non-phase commands, cross-cutting
  references, templates, reviewer prompt, and ADR records;
- ADR access: load the store `INDEX.md` first, then only relevant active records.

Keep the project context and constitution as standing orientation. The change is
about engine procedure loading, not weakening user-authored rules or current
state orientation.

Plan should prove parity with targeted conformance/golden coverage for key
routing scenarios and adapter output:

- explicit command routing;
- natural-language hint handling;
- `{slug} discuss` backlog promotion;
- `{slug} plan` rejecting raw backlog seeds;
- patch eligibility routing;
- review trigger routing;
- Kiro always-on steering contents.

## Rabbit Holes

- Designing a broad context-budget analyzer before the loading contract is
  stable.
- Adding section-level reference anchors before supported agent/tool surfaces
  can read sections consistently.
- Treating this as a chance to rewrite all engine prose instead of preserving
  behavior with a tighter load boundary.
- Using write-capable worker/subagent isolation as the context-pressure answer;
  current ADRs retired that path in favor of main-agent implementation plus
  durable artifacts.

## No-gos

- Do not create a separate `router.card.md` in v1.
- Do not remove constitution or foundational context from the standing layer.
- Do not weaken routing, risk, git, fold, review, or approval-gate rules to save
  tokens.
- Do not add a new `context audit` or `guide --context-budget` command in this
  change.
- Do not introduce section-level frontmatter references in v1.

## Alternatives Considered

- Generate `router.card.md` from `router.md` and command frontmatter. Rejected
  for v1 because it creates a second routing artifact that can drift from the
  authoritative router.
- Keep adapter output as a broad catalog of engine documents. Rejected because
  it leaves the eager-read failure mode in place.
- Move more behavior into ADR or context. Rejected because ADR is historical
  rationale loaded on demand, and context is current-state orientation
  regenerated from code.
- Add a token-budget CLI first. Rejected because measurement is useful later,
  but the load contract must be explicit before measurement can enforce it.

## Open Questions

- Exactly which existing router details can move into command procedures or
  references without changing routing behavior?
- What wording makes the always-loaded vs lazy-loaded boundary obvious across
  AGENTS, Claude, Copilot, and Kiro without making adapter output verbose?
- Which conformance assertions are enough to prove routing parity without
  overfitting to long literal prose?
