---
id: 2026-06-27-index-md-gitignored-derived-cache
date: 2026-06-27
area: [cli]
status: active
---
## INDEX.md is a gitignored derived cache; never stage it (2026-06-27)

**Applies to:** `cli/crates/mochiflow-core/src/index.rs`, the `main.rs`
post-command refresh, and the `accept` / `pr` close-out paths.
**Signal:** A close-out or board refresh stages `INDEX.md`, or `freeze` / CI
fails because a committed `INDEX.md` drifts from the derived board.
**Cause:** Treating `INDEX.md` as a tracked artifact. It is now a regenerated,
gitignored cache of the `delivery::derive_column` board.
**Guardrail:** Regenerate `INDEX.md` only via the shared post-command step
(`refresh_board_after_state_change`, after the commit returns) and never include
it in any staged pathspec. `mochiflow status` is read-only and must never write
it. Keep `INDEX.md` in the install `.gitignore`.
**Check:** `status_is_read_only_and_writes_no_index`,
`index_is_untracked_after_state_changing_command`, `init_writes_install_gitignore`.
**Status:** Active.
