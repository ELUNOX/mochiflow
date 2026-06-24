---
slug: "seed-wording-to-backlog"
title: "Use 'バックログに追加しますか？' instead of 'seed にしますか？'"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "ship"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Use 'バックログに追加しますか？' instead of 'seed にしますか？'

## Signal

When proposing to track an idea for later, the agent says "seed にしますか？"
which exposes internal vocabulary (seed = backlog file format). Users don't know
what a seed is. The intent is "add this to the backlog for later?" — plain
language per language.md's user-facing communication rules.

## Why It Matters

- "seed" is mochiflow-internal jargon that users must look up.
- language.md explicitly says to translate internal terms into plain
  project-collaboration language for user-facing speech.

## Proposed Solution

Add to language.md's user-facing phrasing table:

| internal term | English user-facing | Japanese user-facing |
| --- | --- | --- |
| `seed` / backlog file | add to backlog | バックログに追加 |

Patch-eligible: language.md table addition, no logic.

## Open Questions

- None.
