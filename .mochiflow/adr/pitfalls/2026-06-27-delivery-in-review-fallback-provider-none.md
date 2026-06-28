---
id: 2026-06-27-delivery-in-review-fallback-provider-none
date: 2026-06-27
area: [cli]
status: active
---
## Delivery In Review fallback is provider=none only; github relies on `gh` (2026-06-27)

**Applies to:** `cli/crates/mochiflow-core/src/delivery.rs` `gather_signals`.
**Signal:** Under `provider = github` with `gh` unavailable (offline / not
installed / unauthenticated), an accepted + pushed spec with an open PR shows in
Ready instead of In Review.
**Cause:** The local pushed-and-unmerged In Review signal is gathered only when
`provider == "none"`; for github, In Review depends on the `gh pr view` probe,
which degrades to `false` when `gh` is unavailable. The Done/trailer fallback
still works regardless of provider.
**Guardrail:** This is intentional and AC-08-conformant (the command never fails;
Done still falls back via the base-reachable `Spec:` trailer). Do not "fix" it by
unconditionally gathering the local signal for github without a design decision —
that changes the derivation contract. If In Review accuracy under github-offline
is needed, plan it (e.g. gate the provider probe behind `--fetch` plus a local
fallback).
**Check:** `github_pushed_without_open_pr_stays_ready`,
`provider_none_pushed_unmerged_is_in_review`,
`provider_unavailable_falls_back_to_local_signals`.
**Status:** Active.
