---
id: 2026-07-02-local-mode-branch-tip-cleanup
date: 2026-07-02
area: [cli]
spec: local-only-spec-mode
status: active
---
## Local-mode cleanup depends on preserving the source branch tip (2026-07-02)

**Applies to:** local-mode delivery derivation in
`cli/crates/mochiflow-core/src/delivery.rs`, status/index rendering, and
post-merge cleanup guidance.

**Signal:** A provider-none local-mode spec has been merged, but status cannot
derive delivered/local-cleanup-pending after the source branch was deleted.

**Cause:** Local-only specs do not have committed accepted spec artifacts or a
reachable `Spec:` trailer. Without provider state, the fallback merge signal is
the local source branch tip being reachable from `origin/{base}`. Deleting the
branch before cleanup removes that signal.

**Guardrail:** For provider-none local mode, run post-merge cleanup before
deleting the source branch. Guidance should say that branch-tip reachability is
the fallback signal and that provider-backed projects can rely on provider merge
state instead.

**Check:** `delivery::tests::local_mode_branch_tip_in_base_is_done_without_trailer`
and `delivery::tests::local_mode_branch_tip_signal_is_lost_after_branch_delete`.

**Status:** Active.
