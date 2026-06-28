---
slug: "accept-verification-reuse"
title: "Let accept reuse a fresh build verification instead of always re-running it"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "open"
created: "2026-06-27"
updated: "2026-06-27"
---

# Let accept reuse a fresh build verification instead of always re-running it

## Signal

During `open`, `mochiflow accept` re-ran the full `default` verification (cargo
test + fmt + clippy + freeze --check) even though build had already verified the
same tree and nothing had changed since the last commit except the AC Matrix
edit and the ADR fold (docs only). For this repo it was seconds; for a larger
codebase the unconditional double-run is a noticeable tax on every close-out.

## Why It Matters

`accept` re-verification is a correctness guard (the tree must be green at the
close-out), but if the verified commit is unchanged from the last green build,
the re-run is redundant work. Speeding up close-out lowers the cost of the
open flow without weakening the guarantee.

## Evidence

- `cli/crates/mochiflow-core/src/ship.rs` `run_final_verification` runs each
  surface's `default` command unconditionally inside `run_accept`.
- In this session build had already passed `default`, and the only post-build
  changes were `spec.md` (AC-13 row) and `adr/*.md` (fold) — neither affects the
  Rust verification result.

## Decisions (tentative)

- Record the commit/tree hash of the last green verification and skip the re-run
  when HEAD's tracked source is unchanged (docs-only / spec-only diffs do not
  invalidate it).
- Or add an `--assume-verified` / `--skip-verify` flag for callers (like `open`)
  that just verified, keeping the default strict.

## Open Questions

- What is the right invalidation boundary — any tracked change, or only changes
  under the verified surface's source paths?
- Should the cache live in gitignored `state/` (per-machine) to avoid trusting a
  committed claim?
