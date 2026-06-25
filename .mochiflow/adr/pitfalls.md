# Pitfalls (ADR)

MochiFlow `ship` folds active operational guardrails here. Keep entries short
and actionable; do not use this file as the source of current state.


## contracts.lock format is byte-sensitive (2026-06-22)

`freeze --check` compares the lock file byte-for-byte. The committed format is
single-line JSON: `{"version": "X.Y.Z", "hash": "..."}\n`. Using
`serde_json::to_string_pretty` or changing key order will make the check fail.
Always use the canonical `format!` in `freeze.rs` for lock serialization.


## draft has two valid shapes; lint must branch on spec.md presence (2026-06-23)

**Applies to:** `cli/crates/mochiflow-core/src/lint.rs` draft-status validation.
**Signal:** A pitch-only draft (`spec.yaml` + `pitch.md`, no `spec.md`) wrongly
fails design/AC-Matrix checks, or a plan-expanded draft (with `spec.md`) wrongly
skips the required `design.md` check.
**Cause:** `status: draft` no longer maps to one file set. discuss creates the
pitch-only form; plan expands it into the `spec.md` form, both while still
`draft`.
**Guardrail:** Branch draft validation on `spec.md` presence — skip
`spec.md`/`design.md`/AC-Matrix checks only while `spec.md` is absent; once it
exists, run the full plan-time checks. `approved`/`done` always require
`spec.md`.
**Check:** Conformance cases in `tests/conformance.rs` cover draft+pitch-only
pass, draft without `pitch.md` fail, draft+`spec.md`+elevated missing
`design.md` fail, and approved-without-`spec.md` fail.
**Status:** Active.


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
