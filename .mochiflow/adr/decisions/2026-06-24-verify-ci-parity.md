---
id: 2026-06-24-verify-ci-parity
date: 2026-06-24
area: [cli]
spec: verify-ci-parity
status: active
---
## 2026-06-24 — verify-ci-parity: `default` is the build-completion verification profile

**Decision:** Surface `default` verification is the canonical local command for
spec-lane build completion. Optional profiles such as `quick` and `targeted` are
for intermediate feedback only. For this repository, `cli.default` runs test,
format, clippy, and freeze checks, while `cli.quick` preserves the test-only
loop.

**Why:** `mochiflow ready` and build both center on the `default` profile. A
separate CI-equivalent profile would not close the gap unless every build path
also changed to require it. Keeping the merge-equivalent command in `default`
makes local build evidence match the command the workflow relies on.

**Rejected:** Adding only a `ci` profile (agents would still pass build through
`default`); making `cargo-deny` part of local `default` (CI provisions it through
a GitHub Action, while the checked-in Rust toolchain only guarantees rustfmt and
clippy); weakening CI to match the old local command.

**Consequence:** Project configuration may keep faster optional profiles, but
completion evidence must use `default`. Checks that are human-operated or
CI-only must be explicitly documented rather than implied by local verification.
