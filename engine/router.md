---
name: spec
not_a_phase: true
description: |
  Natural-language routing and lifecycle conductor for mochiflow. Aggregates
  triggers and trigger_patterns from commands/*.md to decide which verb to
  enter. Loaded directly by adapter entrypoints such as Kiro's spec-builder
  agent.
references:
  - commands/discuss.md
  - commands/plan.md
  - commands/build.md
  - commands/ship.md
  - commands/patch.md
  - commands/review.md
  - commands/refresh-context.md
  - reference/workflow.md
  - reference/risk.md
  - reference/language.md
---

# spec

Router for the mochiflow verbs. A tool adapter (e.g. Kiro's `spec-builder` agent)
loads this as a standing instruction. Do not load it from planning / reviewer roles.

## Routing Principles

1. **Do not activate without explicit intent.** Stay in normal conversation unless the user clearly intends to discuss / spec / implement / open a PR. Generic phrases alone ("organize this", "go ahead", "let's talk") do not activate. When in doubt, do not activate.
2. **Artifacts are the state.** A spec's `status` (`draft|approved|done`) and the documents in its folder are the source of truth for current state. There is no separate conversation-history state machine.
3. **Activation strength follows the trigger form.** An explicit command (`mochiflow-<verb>`) or a slug pattern (`{slug} <verb>`) is unambiguous — declare the verb in one line and activate immediately. A natural-language trigger (e.g. "実装して" / "レビューして" / "進めて") is an intent hint, not a command: activate immediately only when an active spec context already scopes it; with no such context, propose "Start <verb>?" in one line and wait. With no trigger at all, propose only on clear intent.
4. **On a state/intent conflict, ask exactly one two-choice question.** Do not silently roll back — e.g. "rework the design" against an already-approved spec.
5. **State lives in files; implementation is inline.** discuss / plan / build / ship are run inline by the main agent, which holds the whole picture. Review is the only separated procedure: run `agents/independent-reviewer.md` read-only, using a delegated subagent when available and inline reviewer role otherwise (`reference/risk.md ## Review transport`). Pass just the slug, command path, a summary of the latest artifact, and a pointer to the spec — never the conversation history as evidence. This includes both risk-cadence review (automatic, per `reference/risk.md ## Consequences`) and ad-hoc review (user-triggered via `レビューして` / `mochiflow-review`; see `reference/risk.md ## Ad-hoc review`).
6. **Patch is non-spec and narrowly scoped.** For concrete, local, reversible changes that need no new product/design decision, route to `patch` instead of starting a spec. Patch never creates `{specs_dir}/{slug}/`, never changes spec status, never archives, never folds memory, and never creates PRs. If a new decision, risk, public contract, migration, or multi-surface scope appears, stop and propose `Start plan?`.

## Decision Flow

1. Read `triggers` and `trigger_patterns` from the `commands/*.md` frontmatter. In `triggers`, a `mochiflow-<verb>` token is an **explicit command**; every other entry is a **natural-language hint**.
2. On an explicit command (`mochiflow-<verb>`) match, declare the command in one line and activate.
3. Match `{slug}` in `trigger_patterns` only against a spec slug that exists under `{specs_dir}/{slug}/`; on a match, declare the verb in one line and activate.
   - Exception: `{slug} discuss` resolves against a seed at `{specs_dir}/_backlog/{slug}.md` when the slug exists only there; if `{specs_dir}/{slug}/` already exists, re-open that spec instead.
   - Event patterns `{slug} merged` / `{slug} マージ済み` / `{slug} 完了` resume ship's post-merge local cleanup only, not a fresh ship. Fold + archive already happen in the close-out commit before `mochiflow pr`.
4. With no active spec context, route concrete small-edit requests ("直して" / "fix this" / "仕様書なしで" / "quick fix") through the `commands/patch.md ## Eligibility` check before proposing a spec verb.
   - If eligible, declare `patch` in one line and proceed.
   - If ineligible or uncertain, propose `Start plan?` in one line and wait.
5. On a natural-language spec hint, activate immediately only when an active spec context already scopes the verb; otherwise propose "Start <verb>?" in one line and wait. A generic "直して" with no active spec context is a patch hint, not a build hint.
6. With no trigger but clear mochiflow intent, propose the verb or non-phase command in one line and wait for approval.
7. With ambiguous intent, do not activate mochiflow.
8. Once committed to a verb or non-phase command, before starting, consult the matching `commands/{verb}.md` and its frontmatter `references` (reference / templates). If they are not in standing context, lazy-load them from the engine root with read.
9. For user-facing speech, follow `reference/language.md ## User-facing communication`: use project-language plain wording first, and keep internal MochiFlow vocabulary only for commands, file names, metadata, or a short `MochiFlow:` note.

## Verb Delegation

| verb | how | ref |
| --- | --- | --- |
| discuss | inline | `commands/discuss.md` |
| plan | inline | `commands/plan.md` |
| build | main agent implements / verifies / commits inline; review uses independent-reviewer transport | `commands/build.md` |
| ship | inline; through acceptance → close-out → PR → post-merge cleanup | `commands/ship.md` |
| patch (non-phase) | inline; no spec artifacts; edit / verify / optional commit; escalate to plan when ineligible | `commands/patch.md` |
| review (non-phase) | inline trigger; read-only review uses independent-reviewer transport; no state transition | `commands/review.md` |
| refresh-context (non-phase) | inline; regenerate foundational context (`[context]`) from code under human confirm; no state transition | `commands/refresh-context.md` |

## Transition Discipline

- discuss fixes current state from **code** and clarifies scope and trade-offs. The constitution (`[constitution]`) is user-authored always-loaded guidance, and the foundational context (`[context]`) is a code-derived current-state map (kept fresh via `refresh-context`); ADR (`[adr]`) is consulted only for *why*, never as the source of current state; re-verify any prose claim against code. A backlog seed is raw input for discuss.
- When readiness is clear, propose the next verb in one line. Never chain verbs without user approval.
- Let depth (spec.md / +design.md / +tasks.md) emerge per `reference/workflow.md ## Depth scaling`. Do not pick a lane up front.
- Patch is not a transition. It does not enter or advance `draft|approved|done`. A patch that discovers product intent, contract shape, migration, security, data-loss, or multi-surface risk stops and routes to `plan`.
- At the end of each verb, present the artifact and the next stage or the human action needed next.

## Completion Output

After running, summarize in the project language using plain user-facing labels:
what changed / what was checked / what the user needs to do next. Do not lead
with an internal state list. Include internal state only when useful, as a brief
`MochiFlow:` note after the summary.
