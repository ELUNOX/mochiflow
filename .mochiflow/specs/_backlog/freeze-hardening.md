---
slug: "freeze-hardening"
title: "Freeze module hardening: error types, format stability, visibility"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "build"
created: "2026-06-22"
updated: "2026-06-22"
---

# Freeze module hardening: error types, format stability, visibility

## Signal

Three code-quality gaps surfaced during the version-ssot-freeze build:

1. **`Result<_, String>` error type** — `freeze.rs` returns stringly-typed
   errors while the rest of the codebase (`manifest.rs`, `upgrade.rs`) uses
   `thiserror` enums. This prevents structured error handling and makes matching
   on specific failure modes impossible for callers.

2. **`contracts.lock` format fragility** — The lock file is a single-line JSON
   whose byte representation is an implicit contract. `freeze --check` does
   byte-level comparison, so any serialization change (e.g. switching to
   `serde_json::to_string_pretty`) silently breaks the check. The format should
   be normalized through a single canonical serializer, or comparison should
   operate on parsed values.

3. **`write_manifest_for_engine_dir` remains `pub`** — After freeze absorbed
   manifest generation, this function is only called internally by `upgrade.rs`.
   It should be `pub(crate)` to prevent external callers from bypassing freeze.

## Why It Matters

These are low-severity maintainability issues individually, but together they
create friction for future contributors: stringly errors make debugging harder,
format fragility creates surprising CI failures, and overly-broad visibility
invites misuse of the pre-freeze manifest writer.

## Decisions (tentative)

- Introduce `FreezeError` enum with `thiserror` in `freeze.rs`.
- Extract a `serialize_lock(version, hash) -> String` function as the single
  format owner.
- Narrow `write_manifest_for_engine_dir` to `pub(crate)`.
