---
id: 2026-06-24-freeze-root-must-not-walk-upward
date: 2026-06-24
area: [cli]
status: active
---
## `freeze --root` must not walk upward (2026-06-24)

**Applies to:** `cli/crates/mochiflow-core/src/freeze.rs`
`validate_repo_root` and the CLI `freeze --root` path.
**Signal:** `mochiflow freeze --root some/subdir --check` passes by resolving an
ancestor source repository, or an invalid explicit path writes derived files
somewhere else.
**Cause:** Reusing `resolve_repo_root(root)` for explicit roots makes `--root`
behave like cwd discovery instead of validating the caller's intended source
repo.
**Guardrail:** Keep explicit root validation marker-based and non-walking:
require `cli/Cargo.toml` and `engine/VERSION` directly under the supplied path.
Only the no-`--root` path should use upward cwd discovery.
**Check:** `validate_repo_root_does_not_walk_to_parent` and CLI root tests cover
exact-root success, invalid-root failure before writes, and cwd fallback.
**Status:** Active.
