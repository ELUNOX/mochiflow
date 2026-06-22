---
slug: "build-completion-guidance"
title: "Build phase should announce next step on completion"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Build phase should announce next step on completion

## Signal

After build completed (AC Matrix all PASS, final commit made), the agent
stopped silently. The user had to ask "次はなに？" to discover that
`mochiflow-ship` was the next action. For first-time users this is a dead end.

## Why It Matters

mochiflow's value is guiding the user through the lifecycle. A silent stop
after build breaks that guidance chain and forces the user to know the verb
sequence by heart or read engine docs.

## Proposed Solution

Add a presentation rule to `engine/commands/build.md` (step 7 or Presentation
section) requiring the agent to report on build completion:

1. Summary of what was implemented and verified.
2. AC Matrix status (all PASS / has PENDING_HUMAN).
3. Explicit next-step guidance: "run `mochiflow-ship` to prepare the PR" (or
   "human QA items remain — ship will request them").

This is a documentation-only change to the engine command; no CLI code needed.
