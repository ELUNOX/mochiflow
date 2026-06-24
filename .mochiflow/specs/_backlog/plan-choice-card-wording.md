---
slug: "plan-choice-card-wording"
title: "Clarify plan step-10 choice card labels for user-facing presentation"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-23"
updated: "2026-06-23"
---

# Clarify plan step-10 choice card labels for user-facing presentation

## Signal

The `later` choice in plan's step-10 card is presented as "ハンドオフプロンプトを
出力して終了". "ハンドオフプロンプト" is internal vocabulary — users do not know
what it means. The intent is "stop now, get a copy-paste memo to resume in a new
session", but the label exposes implementation detail.

## Why It Matters

- Users hesitate to pick `later` because the description is unclear.
- Breaks the language.md principle of translating internal terms into plain
  project-collaboration language for user-facing speech.

## Proposed Solution

Update plan.md step-10 Presentation guidance (or add a Presentation section if
absent) to specify plain user-facing labels. Example:

- **later** — 中断（再開用メモを出力） / "pause — outputs a resume note for the
  next session"

Patch-eligible: plan.md wording-only change, no logic, reversible.

## Open Questions

- Should this also update the `templates/handoff/build-session-prompt.md`
  filename or just the user-facing label?
