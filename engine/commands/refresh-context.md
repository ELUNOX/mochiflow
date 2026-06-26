---
name: mochiflow-refresh-context
description: |
  MochiFlow's refresh-context command. Regenerate the foundational context layer
  (`[context].product` / `[context].structure` / `[context].tech`) from the current code, under
  human confirmation. This is the operational counterpart to the emergent fold:
  context is a current-state orientation map derived from code (forward-placed,
  refreshed), never a dated history log (the fold owns that, in `[adr]`).
  Activate when the user asks to refresh / regenerate project context. If ship
  detects a coarse structural shift, it records a follow-up; run this after PR
  creation / merge or as separate work so pre-PR clean-tree checks stay clean.
triggers:
  - コンテクスト更新して
  - コンテクストを再生成して
  - refresh context
  - refresh-context
trigger_patterns: []
execution: inline
references:
  - commands/onboard.md
  - reference/git.md
  - reference/language.md
  - templates/context/product.md
  - templates/context/structure.md
  - templates/context/tech.md
---

# refresh-context

## Purpose

Regenerate the always-loaded foundational context (`[context].product` /
`[context].structure` / `[context].tech`) from the current code so orientation never silently rots.
Code is the source of truth; this layer is a derived map, not new knowledge.

## When it runs

- The user explicitly asks to refresh project context.
- `ship` detected a coarse structural shift (new module / surface / moved entry
  point) and the human opted in after PR creation / merge, or as a separate
  follow-up (`commands/ship.md`). ship never refreshes automatically and does not
  trigger this during close-out; the regeneration happens here.

## Procedure

1. Read the code to fix current state (never ask what code can answer). Re-derive,
   do not diff prose against prose.
2. Regenerate, reusing onboard's foundational-generation step (`commands/onboard.md`)
   and `templates/context/{product,structure,tech}.md`:
   - `[context].product`: purpose / users / domain terms / core invariants / non-goals.
   - `[context].structure`: coarse code layout / entry points / "source is X,
     generated is Y, vendored is Z" map.
   - `[context].tech`: technology stack, verification surfaces, primary commands,
     generated artifacts, and contract/version gates derived from code/config.
   Include evidence pointers and the source commit/date for each context file.
   Keep all three to the minimal slice that is costly to re-derive yet rarely changes.
3. Present the regenerated context and the diff; the human confirms it matches
   current code before it is committed. Refresh does not auto-commit; if this
   was triggered by ship-detected drift, running it after PR creation / merge or
   as separate follow-up avoids dirtying the pre-PR tree.

## Stop conditions

- Refresh-context updates only the current-state context layer.
- ADR remains the fold target (`reference/git.md ## Living-spec fold`);
  constitution remains user-authored always-loaded guidance.
- Dated history and rationale belong to the fold, not context refresh.
- The human confirms current-state accuracy before any commit.
- Implementation code, branch, and PR handling are outside this command.
