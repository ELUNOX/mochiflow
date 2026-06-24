---
slug: "dogfood-engine-sync-constitution"
title: "Add engine source sync rule to constitution for dogfood repo"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_phase: "build"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Add engine source sync rule to constitution for dogfood repo

## Signal

This repo has a dogfood structure where `engine/` is the source and
`.mochiflow/engine/` is the vendored copy. When engine/ files are edited, the
vendored copy must be synced with `mochiflow upgrade --source engine` and
adapters checked with `mochiflow adapter generate --check`. This step is
currently undocumented — agents rediscover it per spec by trial and error or
by explicit tasks.md instructions.

## Why It Matters

- Every spec touching engine/ must redundantly document the sync step in tasks.md.
- Forgetting it causes adapter drift or stale vendored engine at ship time.
- The rule belongs to this repo, not to engine-docs (general mochiflow users
  don't have a source engine).

## Proposed Solution

Add to `constitution.md` (project-level always-loaded rules):

```markdown
## Dogfood: engine source → vendored sync

When any file under `engine/` is edited, run before final verification:
1. `mochiflow freeze`
2. `mochiflow upgrade --source engine`
3. `mochiflow adapter generate --check`
```

This makes the rule always-loaded so agents see it without per-spec instructions.

Patch-eligible: constitution.md addition only, no logic.

## Open Questions

- None.
