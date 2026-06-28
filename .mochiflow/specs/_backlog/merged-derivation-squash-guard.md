---
slug: "merged-derivation-squash-guard"
title: "Guard merged-derivation and close against squash-merge trailer loss"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "post-build-pr-close-flow"
source_phase: "close"
created: "2026-06-27"
updated: "2026-06-27"
---

# Guard merged-derivation and close against squash-merge trailer loss

## Signal

`merged` is derived from the provider PR state or a `Spec: {slug}` trailer
reachable from `origin/{base_branch}`. In this session the PR was a true merge,
so the close-out commit survived into `main` and `git branch -d` (safe delete)
succeeded. With a squash merge the trailer must be carried into the squash
commit by hand, and the feature branch is not an ancestor of base, so
`git branch -d` fails as "not fully merged". The flow's correctness depends on
merge-style discipline that nothing enforces.

## Why It Matters

If a team squash-merges without preserving the trailer and no provider is
available, the spec stays In Review forever (the trailer fallback never fires).
And `close`'s `git branch -d` failing on a legitimately-merged squash branch is
a confusing dead end for users.

## Evidence

- `delivery.rs` `trailer_reachable_from_base` greps `origin/{base}` for the
  `Spec: {slug}` trailer; squash merges drop trailers unless configured to keep
  them.
- `engine/commands/close.md` step 4 uses `git branch -d` (safe delete) and says
  to leave it and ask the human if it fails.
- spec.md Edge Cases already note the squash case ("stays In Review until the
  trailer/provider confirms").

## Decisions (tentative)

- Document the squash requirement prominently (PR template / git.md): squash
  commits MUST carry the `Spec: {slug}` trailer.
- In `close`, when `git branch -d` fails but the provider reports the PR merged
  (or the user confirms merge), offer `-D` with an explicit confirmation rather
  than a bare stop.
- Consider a `mochiflow` pre-merge hint or a provider-setting check that warns
  when the repo default is squash.

## Open Questions

- Should provider-merged (gh) alone be enough to authorize `close`'s `-D` even
  when the trailer is absent from base?
- Is enforcing/checking merge style in scope, or purely a documented convention?
