# Guide delivery through conversational next actions

## Problem

MochiFlow's delivery flow still exposes too much command-shaped handoff at the
point where users leave the chat to review and merge a PR. After `open` creates
a PR, the agent can report the URL and stop, while the later merge report and
local cleanup are only implied by `close` documentation. New users can assume
the workflow is finished after PR creation, or forget that they should return to
the agent after merging.

This is a product gap because most users will not regularly run `mochiflow`
commands themselves. They expect the AI agent to keep the delivery flow moving
through conversation: explain the current state, say what action is next, and
accept ordinary merge reports such as "merged" or "マージした" when the PR is
done.

## Appetite

This is worth an elevated-risk delivery UX improvement because it touches router
intent, open/close presentation, status/board output, and PR CLI output. It
should not become a full activation-system redesign. The first version should
make the PR-created, in-review, merged-but-cleanup-pending, and close-complete
moments clear in conversation while preserving the current persisted lifecycle.

## Solution

Strengthen delivery as an AI-led conversational flow.

After PR creation, both the CLI output and the `open` command presentation should
include a conversation-language next action: merge the PR in the provider UI,
then return to the chat and report that it was merged. The instruction should be
natural language, not a request to memorize or run a `mochiflow` command.

During `In Review`, `status` / board output should keep that next action
visible so users can recover later. If the PR is already derived as merged but
the local feature branch or `.mochiflow/state/{slug}/` scratch still exists, the
board should show a local-cleanup-pending next action. This is a derived local
hint, not a new spec status.

The router should treat short reports like "merged", "I merged it",
"マージした", or "マージ完了" as merge-report intent in the active conversation
language. If there is a single obvious accepted / in-review / just-opened spec,
route to `close`. If multiple candidates exist, ask one small disambiguation
question instead of requiring exact `{slug} merged` syntax.

`close` remains local hygiene only: switch to the base branch, fast-forward,
safe-delete the feature branch, clear gitignored delivery scratch, and regenerate
the board. Its presentation should be conversational: say that main was updated,
the local branch and temporary delivery files were cleaned up, and the work is
locally wrapped up.

Language handling is part of the contract:

- conversational guidance follows `[i18n].conversation_language`, with `auto`
  resolving to the current user conversation language;
- PR descriptions and other durable artifacts remain governed by
  `[i18n].artifact_language`;
- CLI-only output should reuse the existing language-aware presentation layer
  and cover at least the supported English/Japanese examples;
- merge-report examples are illustrative intents, not fixed trigger strings.

## Rabbit Holes

- Replacing the whole trigger/frontmatter model. That belongs to
  `trigger-routing-redesign`; this spec should only add enough contextual
  merge-report handling to make delivery conversational.
- Creating a new persisted cleanup state. `accepted` remains the last written
  status; `in_review`, `merged`, and `done` remain derived.
- Putting internal cleanup instructions in the external PR body. PR text is for
  reviewers, not local workflow guidance.
- Turning `close` into a command users must learn. The normal path should be a
  plain conversational report after merge.

## No-gos

- Do not introduce a third delivery approval gate.
- Do not require users to remember `mochiflow` commands for the normal delivery
  path.
- Do not write `status: done`, move specs to `_done/`, or persist cleanup
  completion in spec metadata.
- Do not create commits or pushes during `close`; it remains local hygiene only.
- Do not broaden this into the general phase-completion card problem beyond
  delivery.

## Alternatives Considered

- Only add a line to `mochiflow pr` output. Rejected because users may miss the
  one-time message and later need the status/board to recover the next action.
- Tell users to say `{slug} merged` exactly. Rejected for the normal path because
  it still makes users memorize engine syntax; exact slug syntax should remain a
  fallback for ambiguity.
- Keep `phase-completion-guidance` as a broad backlog item. Rejected because its
  old `ship` terminology and general phase-boundary framing are less actionable
  than this delivery-specific seed. Its useful delivery concerns are folded into
  this spec; any future all-phase guidance should be captured separately.
- Add a dedicated cleanup CLI first. Rejected because the missing experience is
  conversational guidance; cleanup mechanics already exist in `close`.

## Open Questions

- What should the board call the merged-but-not-cleaned hint: "cleanup pending",
  "local cleanup pending", or just a next-action line?
- How should router disambiguation be phrased when the user gives a bare
  merge-report intent and multiple accepted/in-review specs exist?
- Should CLI-only `mochiflow pr` output use conversation language exactly, or
  fall back to terminal locale when no active conversation context is available?
- Is local `origin/main` trailer derivation sufficient for cleanup-pending
  status, or should provider-derived merged status also be queried by `status`?
