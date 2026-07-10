---
name: mochiflow-refresh-context
description: |
  MochiFlow's refresh-context command. Regenerate the foundational context layer
  (`[context].product` / `[context].structure` / `[context].tech`) from the current code, under
  human confirmation. This is the operational counterpart to the emergent fold:
  context is a current-state orientation map derived from code (forward-placed,
  refreshed), never a dated history log (the fold owns that, in `[adr]`).
  If open detects a coarse structural shift, it runs this on the feature branch under
  human confirmation and commits the result as a separate `docs(context)` commit
  before the accept close-out, so the refresh ships inside the PR; this command
  itself never auto-commits (branch / PR / commit handling is open's
  responsibility).
execution: inline
load:
  required:
    - reference/knowledge.md
    - reference/presentation.md
  conditional:
    - when: regenerating the foundational context layer
      files:
        - commands/onboard.md
        - templates/context/product.md
        - templates/context/structure.md
        - templates/context/tech.md
    - when: user-facing wording needs the rule
      files:
        - reference/language.md
---

# refresh-context

## Purpose

Regenerate the foundational context (`[context].product` /
`[context].structure` / `[context].tech`) from the current code so workflows can
load current orientation on demand without letting it silently rot.
Code is the source of truth; this layer is a derived map, not new knowledge.

## When it runs

- The user explicitly asks to refresh project context.
- `open` detected a coarse structural shift (new module / surface / moved entry
  point) and runs this **on the feature branch, before the PR**, under human
  confirmation (`commands/open.md` step 4). This is the primary open-triggered
  path: open regenerates the context here, then commits it as a separate
  `docs(context)` commit before the accept close-out so it ships inside the PR.
  Staleness discovered only at or after merge is the fallback, handled as a
  post-merge follow-up. open never refreshes automatically; the regeneration
  happens here and the commit is open's responsibility (this command does not
  auto-commit).

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
   was triggered by open-detected drift, `open` makes the separate
   `docs(context)` commit on the feature branch before the accept close-out
   (`commands/open.md` step (c)), so the refresh ships inside the PR while this
   command stays no-auto-commit.

## Stop conditions

- Refresh-context updates only the current-state context layer.
- ADR remains the fold target (`reference/knowledge.md ## Living-spec fold`);
  constitution remains user-authored always-loaded guidance.
- Dated history and rationale belong to the fold, not context refresh.
- The human confirms current-state accuracy before any commit.
- Implementation code, branch, and PR handling are outside this command.
