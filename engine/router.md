---
name: spec
not_a_phase: true
description: |
  Natural-language routing and lifecycle conductor for mochiflow. Aggregates
  triggers and trigger_patterns from commands/*.md to decide which verb to
  enter. Loaded directly by adapter entrypoints such as Kiro steering or
  generated agent instructions.
references:
  - commands/discuss.md
  - commands/plan.md
  - commands/build.md
  - commands/open.md
  - commands/update.md
  - commands/close.md
  - commands/patch.md
  - commands/review.md
  - commands/refresh-context.md
  - commands/onboard.md
  - reference/workflow.md
  - reference/risk.md
  - reference/language.md
---

# spec

Router for the mochiflow verbs. A tool adapter entrypoint (e.g. Kiro steering or
generated agent instructions) loads this as a standing instruction. Do not load
it from planning / reviewer roles.

## Routing Principles

1. **Do not activate without explicit intent.** Stay in normal conversation unless the user clearly intends to discuss / spec / implement / open a PR. Generic phrases alone ("organize this", "go ahead", "let's talk") do not activate. When in doubt, do not activate.
2. **Artifacts are the state.** A spec's `status` (`draft|approved|accepted`; `done` is legacy/derived only) and the documents in its folder are the source of truth for current state. There is no separate conversation-history state machine.
3. **Activation strength follows the trigger form.** An explicit command (`mochiflow-<verb>`) or a slug pattern (`{slug} <verb>`) is unambiguous — declare the verb in one line and activate immediately. A natural-language trigger (e.g. "実装して" / "レビューして" / "進めて") is an intent hint, not a command: activate immediately only when an active spec context already scopes it; with no such context, propose "Start <verb>?" in one line and wait. With no trigger at all, propose only on clear intent.
4. **On a state/intent conflict, ask exactly one two-choice question.** Do not silently roll back — e.g. "rework the design" against an already-approved spec.
5. **State lives in files; judgment and implementation stay single-threaded; review may delegate.** discuss / plan / build / open / update / close are conducted by the main agent, which holds the whole picture. **Invariant:** judgment, implementation, gates, integration, and the living-spec fold stay single-threaded on the top model — they are never delegated. **Review transport:** independent review runs `agents/independent-reviewer.md` read-only via `reference/risk.md ## Review transport`, preferring delegated subagent dispatch when available and falling back to inline reviewer role only when subagents are unavailable or dispatch fails for a runtime/tooling reason. A review trigger, or a user-approved build flow that reaches mandatory risk-cadence review, is an explicit request to use delegated reviewer transport when the runtime requires one. Pass just the review contract (the slug, command path, a summary of the latest artifact or diff, and a pointer to the spec) — never the conversation history. Risk-cadence review (automatic, per `reference/risk.md ## Consequences`) and ad-hoc review (user-triggered via `レビューして` / `mochiflow-review`; see `reference/risk.md ## Ad-hoc review`) both use this transport.
6. **Patch is non-spec and narrowly scoped.** For concrete, local, reversible changes that need no new product/design decision, route to `patch` instead of starting a spec. Patch never creates `{specs_dir}/{slug}/`, never changes spec status, never archives, never folds memory, and never creates PRs. If a new decision, risk, public contract, migration, or multi-surface scope appears, stop and propose `Start plan?`.

## Decision Flow

1. Read `triggers` and `trigger_patterns` from the `commands/*.md` frontmatter. In `triggers`, a `mochiflow-<verb>` token is an **explicit command**; every other entry is a **natural-language hint**.
2. On an explicit command (`mochiflow-<verb>`) match, declare the command in one line and activate.
3. Match `{slug}` in `trigger_patterns` only against a spec slug that exists under `{specs_dir}/{slug}/`; on a match, declare the verb in one line and activate.
   - Exception: `{slug} discuss` resolves against a seed at `{specs_dir}/_backlog/{slug}.md` when the slug exists only there; if `{specs_dir}/{slug}/` already exists, re-open that spec instead.
   - `{slug} plan` requires an existing active spec directory at `{specs_dir}/{slug}/` created by discuss. If only `{specs_dir}/_backlog/{slug}.md` exists, do not activate plan — guide back to `{slug} discuss` because backlog files are raw seeds, not plan-ready handoffs.
   - Event patterns `{slug} merged` / `{slug} マージ済み` / `{slug} 完了` are the only trigger-pattern exception for delivered specs: resolve `{slug}` against the flat `{specs_dir}/{slug}/` (specs are never moved to `_done/`), then run close's post-merge local cleanup only (`commands/close.md`), not a fresh open. `merged` is derived (provider or the `Spec: {slug}` trailer in `origin/{base_branch}`); the fold already merged via the open PR's close-out commit, so close writes nothing to the base branch.
   - Feedback patterns `{slug} feedback` / `{slug} 修正依頼` / `{slug} PR feedback` also resolve `{slug}` against the flat `{specs_dir}/{slug}/` (the spec stays flat the whole time), then route to `commands/update.md` — not a fresh open and no restore.
4. With no active spec context, route concrete small-edit requests ("直して" / "fix this" / "仕様書なしで" / "quick fix") through the `commands/patch.md ## Eligibility` check before proposing a spec verb.
   - If eligible, declare `patch` in one line and proceed.
   - If ineligible or uncertain, propose `Start plan?` in one line and wait.
5. On a natural-language spec hint, activate immediately only when an active spec context already scopes the verb; otherwise propose "Start <verb>?" in one line and wait. A generic "直して" with no active spec context is a patch hint, not a build hint.
6. With no trigger but clear mochiflow intent, propose the verb or non-phase command in one line and wait for approval.
7. With ambiguous intent, do not activate mochiflow.
8. Once committed to a verb or non-phase command, before starting, consult the matching `commands/{verb}.md` and its frontmatter `references` (reference / templates). If they are not in standing context, lazy-load them from the engine root with read.
9. For user-facing speech, follow `reference/language.md ## User-facing communication`: use conversation-language plain wording first, and keep internal MochiFlow vocabulary only for commands, file names, metadata, or a short `MochiFlow:` note.

## Active Spec Resolution

Resolve the active spec in this order:

1. Explicit slug from the user, for example `{slug} build`.
2. Explicit path to `{specs_dir}/{slug}/`.
3. Current git branch matching the spec branch convention in
   `reference/git.md ## Branch` (`{prefix}/{slug}`).
4. Exactly one active (non-merged) spec whose status allows the requested verb per the
   command prerequisites.
5. Exactly one recently modified spec only when the user refers to "this spec"
   or "the current plan".

If more than one candidate remains, ask one concise disambiguation question.
Never guess between multiple candidate specs.

## PR Feedback Loop Routing

If PR feedback, CI failure, reviewer comments, or PR-body approval follow-up
requires code changes before merge, route the work to the in-review spec via
`commands/update.md` instead of `patch`, unless the change is unrelated to that
spec. The spec stays flat at `{specs_dir}/{slug}/` (no restore needed); `update`
applies bounded inline fixes, re-verifies, pushes, and updates the PR body when
needed.

## Verb Delegation

| verb | how | ref |
| --- | --- | --- |
| discuss | inline | `commands/discuss.md` |
| plan | inline | `commands/plan.md` |
| build | inline; main agent confirms eligibility (`mochiflow ready {slug}`), then implements task units in order, verifies, commits, records the AC Matrix, and runs risk-cadence review through the independent-reviewer transport when required | `commands/build.md` |
| open | inline; through acceptance → fold + context-check → optional `docs(context)` commit (regenerated `[context]`, before accept) → accept close-out → PR title/body → approve-PR gate → PR. The QA-`FAIL` rework loop applies a bounded inline code fix, re-verifies, and refreshes review when needed; judgment / fold / PR-body / gates stay inline | `commands/open.md` |
| update | inline; the PR-feedback / CI-fix code change applies a bounded inline code fix, then re-verifies, pushes, and updates PR metadata. Feedback interpretation and PR-metadata updates stay inline; no move, no revert | `commands/update.md` |
| close | inline; post-merge local hygiene only; nothing written to the base branch | `commands/close.md` |
| patch (non-phase) | inline; no spec artifacts; edit / verify / optional commit; escalate to plan when ineligible | `commands/patch.md` |
| review (non-phase) | inline trigger; read-only review uses independent-reviewer transport; no state transition | `commands/review.md` |
| refresh-context (non-phase) | inline; regenerate foundational context (`[context]`) from code under human confirm; no state transition | `commands/refresh-context.md` |
| onboard (non-phase) | inline; setup / first-run project onboarding | `commands/onboard.md` |

## Transition Discipline

- discuss fixes current state from **code** and clarifies scope and trade-offs. The constitution (`[constitution]`) is user-authored always-loaded guidance, and the foundational context (`[context]`) is a code-derived current-state map (kept fresh via `refresh-context`); ADR (`[adr]`) is consulted only for *why*, never as the source of current state; re-verify any prose claim against code. A backlog seed is raw input for discuss.
- When readiness is clear, propose the next verb in one line. Never chain verbs without user approval.
- Let depth (spec.md / +design.md / +tasks.md) emerge per `reference/workflow.md ## Depth scaling`. Do not pick a lane up front.
- Patch is not a transition. It does not enter or advance `draft|approved|accepted`. A patch that discovers product intent, contract shape, migration, security, data-loss, or multi-surface risk stops and routes to `plan`.
- At the end of each verb, present the artifact and the next stage or the human action needed next.

## Completion Output

After running, summarize in the conversation language using plain user-facing labels:
what changed / what was checked / what the user needs to do next. Do not lead
with an internal state list. Include internal state only when useful, as a brief
`MochiFlow:` note after the summary.

When presenting next steps, prefer a numbered choice card whose labels describe
user actions in the conversation language. Numbers are aliases for the most
recent unambiguous card only; otherwise route by the explicit label, keyword, or
normal intent rules.
