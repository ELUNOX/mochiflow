---
slug: "phase-completion-guidance"
title: "Every phase should present the next action clearly"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_phase: "post-merge"
created: "2026-06-24"
updated: "2026-06-24"
---

# Every phase should present the next action clearly

## Signal

During Codex-driven MochiFlow work, some phase completions did not clearly show
the expected next action even though the lifecycle is intended to guide users
through `discuss -> plan -> build -> ship`.

## Why It Matters

MochiFlow's value depends on keeping users oriented at phase boundaries. If the
agent skips the next-action prompt, users must already know the command sequence
or infer what to do next, which weakens the guided workflow.

## Evidence

- `engine/commands/discuss.md` already asks for `plan` / `later`, and
  `engine/commands/plan.md` asks for `review` / `build` / `later`, but Codex can
  still continue without surfacing the choice when the user says "承認、進んで".
- `engine/commands/build.md` has explicit guidance to direct users to
  `mochiflow-ship`, but this relies on the agent honoring the Presentation
  section.
- `engine/commands/ship.md` explains PR approval and post-merge cleanup, but its
  Presentation section is weaker about always showing the next user action after
  PR creation and after cleanup.

## Open Questions

- Should there be a shared "phase completion card" rule in `workflow.md` that
  every phase references?
- Should each command define an explicit final output block with stable choices
  and allowed auto-continuation behavior?
- Should ship explicitly tell the user to report merge with `{slug} merged` /
  `マージ完了` after PR creation?
