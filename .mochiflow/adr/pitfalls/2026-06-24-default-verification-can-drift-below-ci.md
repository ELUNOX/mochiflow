---
id: 2026-06-24-default-verification-can-drift-below-ci
date: 2026-06-24
area: [cli]
status: active
---
## Surface `default` verification can drift below CI coverage (2026-06-24)

**Applies to:** `.mochiflow/config.toml` `[surfaces.*.verify]` and
`engine/reference/workflow.md` verification guidance.
**Signal:** Local build passes but CI fails on formatting, lint, freeze, or other
merge-blocking checks the agent was not asked to run.
**Cause:** Treating `default` as a fast unit-test command while build and
`mochiflow ready` use it as the canonical surface verification signal.
**Guardrail:** Keep `default` as the reliable local build-completion command for
the surface. Put fast loops in `quick` / `targeted`, and explicitly document
human-operated or CI-only checks that are not locally guaranteed.
**Check:** `mochiflow config show` should show the merge-equivalent command under
`default`; build evidence should cite the `default` command, not only `quick`.
**Status:** Active.
