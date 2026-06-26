---
slug: "active-spec-open-directory"
title: "Introduce an _open directory for active specs"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "discuss"
created: "2026-06-26"
updated: "2026-06-26"
---

# Introduce an _open directory for active specs

## Signal

Active specs currently live directly under `{specs_dir}/{slug}/`, while raw
ideas live under `_backlog/` and completed specs live under `_done/`. The
directory layout is functional, but the root of `specs/` mixes active spec
folders with lifecycle collection directories.

## Why It Matters

- `_open/{slug}/` would make the lifecycle collections visually symmetrical:
  backlog seeds, open specs, and done specs.
- The name `_open` avoids overloading "active", which can also mean the spec
  currently selected by a conversation, branch, or command.
- A cleaner storage contract may make future routing, indexing, and onboarding
  explanations easier, especially as parallel spec work grows.

## Evidence

- Current engine instructions repeatedly refer to `{specs_dir}/{slug}/` as the
  active spec path.
- `_backlog/{slug}.md` is already a raw-seed collection, and `_done/{slug}/` is
  already the archive collection.
- The likely change is broad: router rules, verb procedures, CLI path
  resolution, lint/index/ready behavior, generated adapters, conformance tests,
  and PR feedback restore flow all reference the current active spec path.

## Open Questions

- Should `_open/{slug}/` become the canonical path for new specs while legacy
  `{specs_dir}/{slug}/` remains readable for compatibility?
- Should an upgrade or migration command move existing open specs into `_open/`,
  or should projects migrate manually?
- Should `specs_dir` continue to mean the lifecycle root, or should config gain
  explicit `open_specs_dir`, `backlog_dir`, and `done_specs_dir` concepts?
- How should duplicate slug detection work during the compatibility window
  across `_open/{slug}/`, legacy `{slug}/`, and `_done/{slug}/`?
