---
slug: "manifest-test-isolation"
title: "Isolate MANIFEST integrity check from functional conformance tests"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "build"
source_spec: "ac-matrix-token-normalization"
created: "2026-06-23"
updated: "2026-06-23"
---

# Isolate MANIFEST integrity check from functional conformance tests

## Signal

Any edit to `engine/**` causes 7 conformance tests to fail due to MANIFEST hash
drift, even though those tests are checking functional behavior (upgrade logic,
doctor warnings, drift detection) — not MANIFEST freshness. Developers must run
`mochiflow freeze` after every engine doc edit before tests pass. This adds a
manual step to every task that touches engine files and inflates tasks.md with
freeze instructions.

## Why It Matters

- DX friction: 5–6 manual freeze invocations per spec when editing engine docs.
- Task bloat: every task touching engine/ must document the freeze step.
- False failures: tests fail for "wrong reason" (stale hash, not broken logic).
- Conflation: "is the MANIFEST committed?" (integrity) is mixed with "does the
  feature work?" (functional).

## Proposed Solution

1. Separate conformance tests into two layers:
   - Functional tests: do not depend on MANIFEST being fresh. Compute hashes
     dynamically or skip MANIFEST checks entirely.
   - Integrity test (single): equivalent to `mochiflow freeze --check`. Only
     this test fails on drift.
2. Mark the integrity test as `#[ignore]` (or feature-gated) so `cargo test`
   runs functional tests only; CI runs `cargo test -- --ignored` or
   `mochiflow freeze --check` separately as its integrity gate.
3. Optionally add `mochiflow freeze` to `mochiflow pr` pre-flight or a
   pre-commit hook so developers never need to run it manually.

## Open Questions

- Which of the 7 currently-failing tests genuinely need MANIFEST (e.g. the drift
  detection test) vs. which just happen to invoke doctor/upgrade which
  internally reads MANIFEST?
- Should the integrity check live as a Rust test or purely as a CLI command
  (`freeze --check`) invoked by CI?
