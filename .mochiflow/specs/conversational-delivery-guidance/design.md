# Guide delivery through conversational next actions — Design

## Design Decisions

- **Use conversation language for next actions.** Agent-facing guidance follows
  `[i18n].conversation_language`. CLI-only output uses
  `Config::conversation_output_language()`, so `auto` falls back to artifact
  language when no live conversation context exists.
- **Keep artifacts separate from delivery prompts.** PR titles/descriptions and
  durable spec artifacts remain artifact-language outputs. The post-merge
  cleanup instruction is not inserted into the PR body because it is local
  workflow guidance for the author, not review material.
- **Name the derived local hint `local cleanup pending`.** The label is explicit
  enough to explain why a Done item can still have a local next action, without
  introducing a new lifecycle state.
- **Prefer contextual merge reports over exact syntax.** Exact `{slug} merged`
  remains valid, but a bare merge-report intent can route to `close` when the
  current branch or the active delivery board yields exactly one candidate. More
  than one candidate triggers one disambiguation question.
- **Do not persist cleanup state.** Cleanup-pending is derived from local facts:
  an accepted spec is already delivered, and the corresponding local feature
  branch and/or `.mochiflow/state/{slug}/` still exists.

## Architecture

The implementation spans three layers:

- Engine procedure text:
  - `engine/commands/open.md` defines the PR-created conversational handoff.
  - `engine/router.md` defines contextual merge-report routing and
    disambiguation.
  - `engine/commands/close.md` defines local cleanup presentation.
  - `engine/reference/language.md` documents language ownership for delivery
    next actions.
- CLI delivery presentation:
  - `cli/crates/mochiflow-core/src/pr.rs` prints next-action guidance after
    successful automated PR creation, legacy command success, and manual
    handoff.
  - Existing PR command tests under `cli/crates/mochiflow-cli/tests/pr.rs`
    exercise these output paths.
- Delivery board rendering:
  - `cli/crates/mochiflow-core/src/delivery.rs` remains the source of delivery
    column derivation and gains or exposes helper logic for local cleanup
    pending when appropriate.
  - `cli/crates/mochiflow-core/src/status.rs` and
    `cli/crates/mochiflow-core/src/index.rs` render next-action hints for
    in-review and cleanup-pending specs.

The status command stays read-only. `mochiflow index` remains the only command
that writes the generated board files.

## Data Model / Interfaces

- No config schema changes are planned.
- No spec schema or persisted lifecycle changes are planned.
- The existing `DeliveryColumn` remains unchanged.
- Add a derived presentation concept, either as a helper struct or as fields on
  board entries, with enough data to render:
  - no hint;
  - in-review next action;
  - local cleanup pending next action.
- JSON board output uses a stable contract on every spec entry:
  - `next_action`: `null` or an object with:
    - `kind`: `"report_merge"` for in-review work, or
      `"local_cleanup_pending"` for done-derived work that still needs local
      cleanup;
    - `message`: the rendered conversation-language next action;
  - `local_cleanup_pending`: boolean, `true` only when cleanup remains for a
    done-derived spec.
  Markdown/status output may choose its wording, but tests must pin the JSON
  field names and `kind` values.

## Error Handling

- If merge-report intent has multiple candidate specs, ask one concise
  disambiguation question and do not run cleanup.
- If merge-report intent has no accepted in-review or cleanup-pending candidate,
  do not route to cleanup; fall through to normal routing (and, when the intent
  is otherwise clear, the agent may ask for the slug rather than acting).
- If cleanup-pending detection cannot inspect local branch state, degrade to no
  cleanup hint rather than failing status or index rendering.
- If provider state is unavailable, keep existing delivery derivation behavior:
  fall back to local git signals and the `Spec:` trailer in `origin/main`.
- If PR creation succeeds through a path that does not return a URL, such as
  manual handoff, describe the handoff instead of requiring a URL and still
  print the post-merge next action. If an automated backend is expected to
  return a URL but does not, preserve the existing backend failure behavior; do
  not print success guidance without a successful handoff.

## Test Strategy

- Add PR output tests for:
  - GitHub/provider success path where a URL is printed;
  - custom driver success path;
  - manual handoff path;
  - at least English and Japanese next-action wording where language-aware
    output is supported by the existing presentation layer.
- Add status/index tests for:
  - accepted + in-review renders a merge-then-report next action;
  - generated board JSON for in-review entries sets
    `next_action.kind = "report_merge"` and
    `local_cleanup_pending = false`;
  - accepted + done-derived + local branch or scratch renders
    `local cleanup pending`;
  - generated board JSON for cleanup-pending entries sets
    `next_action.kind = "local_cleanup_pending"` and
    `local_cleanup_pending = true`;
  - after the local branch and scratch are removed, the same done-derived spec
    no longer renders `local cleanup pending`, sets `next_action = null`, and
    sets `local_cleanup_pending = false`;
  - cleanup-pending does not write `status: done`, move specs, or mutate files
    in `status`.
- Add conformance assertions for:
  - `open.md` PR-created presentation;
  - `close.md` conversational local-cleanup presentation;
  - `router.md` contextual merge-report routing and disambiguation;
  - language policy ownership for conversation vs artifact text.
- Run the configured `cli` verification profile before build completion.
- After engine source edits, run `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check`.

## Review Results
