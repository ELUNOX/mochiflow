---
slug: "release-config-schema-context-terminology"
title: "Correct frozen config.schema.json context description to conditional"
surface: "cli"
type_hint: "docs"
maturity: "seed"
source: "conversation"
source_spec: "engine-context-slimming-redesign"
source_phase: "build"
created: "2026-07-09"
updated: "2026-07-09"
---

# Correct frozen config.schema.json context description to conditional

## Signal

`engine-context-slimming-redesign` moved foundational context out of the standing
layer and updated non-frozen Rust/docs terminology to describe it as loaded on
demand, but intentionally left the frozen `contracts/config.schema.json` context
description ("Foundational living-spec layer (refresh targets, always-loaded)")
unchanged to avoid an early version bump.

## Why It Matters

Live sources (`config.rs`, `init.rs`, `engine/README.md`, adapter entrypoints)
now describe context conditionally while the frozen schema still says
"always-loaded". This is a deliberate, temporary disagreement that must be
reconciled at the next release so the frozen contract matches reality.

## Evidence

- `contracts/config.schema.json` (context description still reads always-loaded)
- `.mochiflow/specs/engine-context-slimming-redesign/design.md` ## Deferred Release Follow-up
- `cli/crates/mochiflow-core/src/config.rs` (context now documented as loaded on demand)

## Open Questions

- Fold this into the next maintainer release (version choice, CHANGELOG, README
  badges, `Cargo.lock`, freeze, adapter generate, doctor, release branch) per
  `.mochiflow/constitution.local.md`, keeping config shape / validation /
  `schema_version` / paths unchanged.
