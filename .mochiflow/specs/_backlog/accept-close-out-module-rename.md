---
slug: "accept-close-out-module-rename"
title: "Rename the ship.rs close-out module and identifiers to accept"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "review"
created: "2026-06-27"
updated: "2026-06-27"
---

# Rename the ship.rs close-out module and identifiers to accept

## Signal

`post-build-pr-close-flow` removed `ship` as a user-facing concept and renamed
the close-out command to `accept`, but the implementation module is still
`ship.rs` and carries `ship`-named identifiers (`allowed_ship_paths`,
`is_allowed_ship_path`, `is_allowed_ship_path` helpers, etc.). The user-facing
FAIL string was fixed during review, but the internal naming still drifts from
the live vocabulary.

## Why It Matters

A future maintainer reading `ship.rs` will reasonably ask "is `ship` still a
thing?" The concept was deliberately retired; the code should not keep a
retired name as its primary surface. Pure naming debt, but it erodes the
asserted-vs-derived mental model the spec established.

## Evidence

- `cli/crates/mochiflow-core/src/ship.rs` defines `run_accept`,
  `allowed_ship_paths`, `is_allowed_ship_path`; referenced as `crate::ship::...`
  from `pr.rs`.
- Independent reviewer (post-build-pr-close-flow) flagged this as a Low
  maintainability finding; only the user-facing string was addressed inline.

## Decisions (tentative)

- Rename `ship.rs` -> `accept.rs`, update `lib.rs` `pub mod`, the `crate::ship::`
  references in `pr.rs`, the `allowed_ship_paths` / `is_allowed_ship_path`
  identifiers, and any test/`doctor` references.
- Mechanical rename; keep behavior identical. Verify with the full default
  profile.

## Open Questions

- Is there any persisted artifact or external doc that refers to the `ship`
  module name that would also need updating?
- Bundle with any other accept-mechanics change to avoid a churn-only PR, or
  ship as a standalone refactor?
