---
slug: "frozen-surface-ssot"
title: "Single source for frozen-surface input set definition and test performance"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "build"
created: "2026-06-22"
updated: "2026-06-22"
---

# Single source for frozen-surface input set definition and test performance

## Signal

Two related maintainability gaps in the contract/conformance layer:

1. **Frozen-surface input set defined in 4+ places** — The fact that the hash
   covers "contracts/*.json sorted, then tests/conformance/golden/** sorted" is
   repeated in: `freeze.rs` code + doc-comment, `conformance.rs` doc-comment,
   `design.md`, `spec.md`, and `contracts/VERSIONING.md`. Changing the input set
   (e.g. adding a new golden subdirectory) requires updating all of them. This
   violates the SSOT principle that version-ssot-freeze just established for
   version numbers.

2. **Conformance tests are fork-heavy** — Many conformance tests spawn the full
   CLI binary via `assert_cmd`. Pure logic tests (hash computation, version
   parsing) were moved to unit tests in freeze.rs, but golden/drift tests still
   fork. As the test count grows, CI wall time increases linearly with forks.
   Some of these could be library-level integration tests calling core functions
   directly with tempfile fixtures, avoiding the process overhead.

## Why It Matters

Scattered input-set definitions create the same drift problem that
version-ssot-freeze solved for version numbers — eventual inconsistency. The
fork overhead is tolerable today (sub-2s) but will compound as tests accumulate.

## Decisions (tentative)

- Define the frozen-surface input set in one authoritative location (a constant
  in `freeze.rs` or a declarative file like `contracts/FROZEN_INPUTS.toml`) and
  have documentation reference it rather than restate it.
- Evaluate which `assert_cmd` conformance tests can be converted to lib-level
  tests using `mochiflow_core` directly (golden-index, drift-doctor likely can;
  schema validation already doesn't need the binary).
