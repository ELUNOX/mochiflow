---
id: 2026-06-23-draft-two-valid-shapes-lint-branch
date: 2026-06-23
area: [cli]
status: active
---
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
