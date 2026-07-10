# Presentation Reference

Generic user-facing summaries, action cards, and internal-term suppression.
Commands own their phase-specific user actions and card options; this file owns
the shared wording discipline so each command does not restate it. Language
selection rules live in `reference/language.md`; choice-card state dispatch
semantics live in `reference/lifecycle.md ## Choice cards`.

## Completion summaries

After a verb or non-phase command runs, summarize in the conversation language
using plain user-facing labels: what changed / what was checked / what the user
needs to do next. Do not lead with an internal state list (`risk`, `status`,
reviewer mode). Include internal state only when useful, as a brief `MochiFlow:`
note after the summary.

## Action cards

When presenting next steps, prefer a numbered choice card whose labels describe
user actions in the conversation language. Numbers are aliases for the most
recent unambiguous card only; otherwise route by the explicit label, keyword, or
normal intent rules (`reference/lifecycle.md ## Choice cards`).

Choice-card labels are user-facing action labels, so they follow the
conversation language. Compatibility keywords such as `build`, `open`, `review`,
`later`, and `approved` remain stable inputs, but they should be secondary to the
plain action label displayed to the user.

## Internal-term suppression

MochiFlow uses precise internal vocabulary for routing and validation, but the
user experience should read like normal project collaboration. In ordinary
conversation and completion summaries, translate internal terms into plain
language (`reference/language.md` carries the meaning-guide table). Keep internal
terms only for file names, commands, metadata fields, schema enum values, and
canonical table tokens required by tooling. When a reviewer verdict is reported,
call it the review result in the project language and keep `delegated` / `inline`
as a short `MochiFlow:` detail only when it explains how the review ran or the
user asks.
