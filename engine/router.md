---
name: spec
not_a_phase: true
description: |
  Compact standing router for mochiflow. Owns a compact route table and selects
  routes from it alone — whether to stay in normal conversation, enter a
  lifecycle verb, or run a non-phase command — without reading command
  frontmatter to route.
  Loaded directly by adapter entrypoints such as Kiro steering or generated
  agent instructions.
references:
  - commands/discuss.md
  - commands/plan.md
  - commands/build.md
  - commands/open.md
  - commands/update.md
  - commands/close.md
  - commands/review.md
  - commands/refresh-context.md
  - commands/onboard.md
  - reference/git.md
  - reference/delivery.md
  - reference/language.md
  - reference/agent-context.md
---

# spec

Compact standing router for the mochiflow verbs and non-phase commands. A tool
adapter entrypoint (e.g. Kiro steering or generated agent instructions) loads
this as a standing instruction. Do not load it from planning / reviewer roles.

## Standing Load Contract

`router.md` is the only standing router artifact. Do not create a second route
card for normal operation. The standing layer is:

- the adapter entrypoint;
- the configured constitution;
- this router;
- foundational context and project config only when a selected workflow or
  repository-specific task needs current-state orientation, verification, git,
  or surface details.

The `references` frontmatter above is a lazy-load catalog, not an instruction to
read every file before routing. Route vocabulary comes only from the router's
`## Route table`; never read command frontmatter to route. Load `reference/git.md`
only when branch-based active-spec resolution is needed, and
`reference/delivery.md` only when delivery state is needed for feedback or merge
routing. After the router selects a lifecycle verb or non-phase command, read the matching
`commands/{verb}.md` and that command's declared load contract (`load.required`
immediately, then the `load.conditional` entries whose `when` condition
resolves). Read ADR records only on demand: load the store `INDEX.md` first,
then the relevant active records.

## Routing Principles

1. **Do not activate without explicit intent.** Stay in normal conversation unless the user clearly intends to discuss / spec / implement / open a PR. Generic phrases alone ("organize this", "go ahead", "let's talk") do not activate. When in doubt, do not activate.
2. **Resolve state from its owners.** Use current spec artifacts for asserted
   state and `reference/delivery.md` when derived delivery state affects routing;
   never infer workflow state from conversation history.
3. **Activation strength follows the trigger form.** An explicit command (`mochiflow-<verb>`) or a slug pattern (`{slug} <verb>`) is unambiguous — declare the verb in one line and activate immediately. A natural-language trigger (e.g. "実装して" / "レビューして" / "進めて") is an intent hint, not a command: activate immediately only when an active spec context already scopes it; with no such context, propose "Start <verb>?" in one line and wait. With no trigger at all, propose only on clear intent.
4. **On a state/intent conflict, ask exactly one two-choice question.** Do not silently roll back — e.g. "rework the design" against an already-approved spec.
5. **Route to the selected owner before executing.** The matching command owns
   execution. Its declared loads supply lifecycle, review, delivery, and other
   workflow policy only after route selection.
6. **Small concrete work stays in the spec lane.** With no active spec, concrete small-edit requests are plan intent hints: propose `Start plan?` and wait. Do not route them to a separate no-spec lane.

## Route table

The router owns the complete route vocabulary and routes from this table alone;
it does not read command frontmatter to select a route. A `mochiflow-<verb>`
token is an explicit command; a natural-language entry is an intent hint whose
activation strength is decided by `## Routing Principles`.

| target | explicit command | natural-language hints | slug / event patterns |
| --- | --- | --- | --- |
| `commands/discuss.md` | `mochiflow-discuss` | ブレストして · 壁打ちして · 相談したい | `{slug} discuss` (seed exception, step 4) |
| `commands/plan.md` | `mochiflow-plan` | 仕様作って · プランして · 計画作って | `{slug} plan` (requires existing draft, step 4) |
| `commands/build.md` | `mochiflow-build` | 実装して · 進めて · ビルドして | `{slug} build` |
| `commands/open.md` | `mochiflow-open` | PR出して · PRを作って | `{slug} open` |
| `commands/update.md` | `mochiflow-update` | 修正依頼 · PR feedback · PRを直して | `{slug} update` · `{slug} feedback` / `{slug} 修正依頼` / `{slug} PR feedback` |
| `commands/close.md` | `mochiflow-close` | merged · マージ済み · 完了 | `{slug} close` · `{slug} merged` / `{slug} マージ済み` / `{slug} 完了` (cleanup only) |
| `commands/review.md` | `mochiflow-review` | レビューして | `{slug} review` · `{slug} review fix` · `{slug} review fix 1` · `{slug} review fix 2` · `{slug} review fix 3` |
| `commands/onboard.md` | — | オンボーディングして · MochiFlow 入れて · mochiflow セットアップ · mochiflow setup · setup mochiflow | — |
| `commands/refresh-context.md` | — | コンテクスト更新して · コンテクストを再生成して · refresh context · refresh-context | — |
| (retired) | `mochiflow-patch` | — | — → say patch is retired, propose `Start plan?` (step 2) |

Numeric review forms route to `commands/review.md` only for correction:
`{slug} review 2` is ambiguous, and `{slug} review fix 0` / `fix 4+` are out of
range. Slug / event pattern nuances (discuss seed exception, plan-requires-draft,
feedback, merged-event cleanup) are detailed in `## Decision Flow`,
`## PR Feedback Loop Routing`, and `## Merge Report Routing`.

## Decision Flow

1. Match the message against the `## Route table` above — the router's own route vocabulary; do not read command frontmatter to route. In the table, a `mochiflow-<verb>` token is an **explicit command**; every natural-language entry is a **natural-language hint**.
2. On the retired explicit command `mochiflow-patch`, say in one line that `patch` is retired and small fixes now start with `plan`; then propose `Start plan?` and wait.
3. On any other explicit command (`mochiflow-<verb>`) match, declare the command in one line and activate.
4. Match `{slug}` against the `## Route table` slug / event patterns column, only for a spec slug that exists under `{specs_dir}/{slug}/`; on a match, declare the verb in one line and activate.
   - Exception: `{slug} discuss` resolves against a seed at `{specs_dir}/_backlog/{slug}.md` when the slug exists only there; if `{specs_dir}/{slug}/` already exists, re-open that spec instead.
   - `{slug} plan` requires an existing active spec directory at `{specs_dir}/{slug}/` created by discuss. If only `{specs_dir}/_backlog/{slug}.md` exists, do not activate plan — guide back to `{slug} discuss` because backlog files are raw seeds, not plan-ready handoffs.
   - Event patterns `{slug} merged` / `{slug} マージ済み` / `{slug} 完了` resolve the existing spec, confirm merge/cleanup eligibility via `reference/delivery.md`, and route to `commands/close.md`, not a fresh open.
   - Feedback patterns `{slug} feedback` / `{slug} 修正依頼` / `{slug} PR feedback` resolve the existing spec, confirm in-review eligibility via `reference/delivery.md`, and route to `commands/update.md`, not a fresh open.
   - Review patterns `{slug} review`, `{slug} review fix`, `{slug} review fix 1`, `{slug} review fix 2`, and `{slug} review fix 3` route to `commands/review.md`. Numeric review forms route there only for correction: `{slug} review 2` is ambiguous, and `{slug} review fix 0` / `fix 4+` are out of range.
5. With no active spec context, treat concrete small-edit requests ("直して" / "fix this" / "仕様書なしで" / "quick fix") as plan intent hints: propose `Start plan?` in one line and wait.
6. On a natural-language spec hint, activate immediately only when an active spec context already scopes the verb; otherwise propose "Start <verb>?" in one line and wait. A generic "直して" with no active spec context is a plan hint, not a build hint.
7. With no trigger but clear mochiflow intent, propose the verb or non-phase command in one line and wait for approval.
8. With ambiguous intent, do not activate mochiflow.
9. Once committed to a verb or non-phase command, before starting, consult the matching `commands/{verb}.md` and that command's declared load contract (`load.required`, then the `load.conditional` entries whose `when` resolves). If they are not in standing context, lazy-load them from the engine root with read.
10. For user-facing speech, follow `reference/language.md ## Conversation Language`: use conversation-language plain wording first, and keep internal MochiFlow vocabulary only for commands, file names, metadata, or a short `MochiFlow:` note.

## Active Spec Resolution

Resolve the active spec in this order:

1. Explicit slug from the user, for example `{slug} build`.
2. Explicit path to `{specs_dir}/{slug}/`.
3. Current git branch matching the spec branch convention in
   `reference/git.md ## Branch` (`{prefix}/{slug}`).
4. Exactly one spec whose current asserted and, when relevant, derived delivery
   state allows the requested command prerequisites.
5. Exactly one recently modified spec only when the user refers to "this spec"
   or "the current plan".

If more than one candidate remains, ask one concise disambiguation question.
Never guess between multiple candidate specs.

## PR Feedback Loop Routing

If PR feedback, CI failure, reviewer comments, or PR-body approval follow-up
requires code changes before merge, route the work to the in-review spec via
`commands/update.md`, unless the change is unrelated to that spec. Resolve
in-review eligibility through `reference/delivery.md`; update owns the fix and
publication procedure.

## Merge Report Routing

A bare merge report in the conversation language — e.g. "merged", "I merged it",
"マージした", "マージ完了" — is a merge-report hint (these are illustrative
intent examples, not fixed trigger strings). It means the user finished the
external merge and is ready for cleanup routing. Resolve the active spec per
`## Active Spec Resolution`, using `reference/delivery.md` to identify eligible
delivery-state candidates, then:

- exactly one such candidate → route to `commands/close.md`;
- more than one candidate → ask exactly one disambiguation question; never guess
  which spec was merged;
- no in-review or cleanup-pending candidate → do not route to cleanup; fall
  through to normal routing (treat it as ordinary conversation, or ask for the
  slug when the intent is otherwise clear).

Exact `{slug} merged` / `{slug} マージ済み` / `{slug} 完了` remains the
unambiguous explicit path (Decision Flow step 4) and is unaffected by this
contextual handling.
