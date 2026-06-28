---
id: 2026-06-24-doctor-and-freeze-stay-separate-diagnostics
date: 2026-06-24
area: [cli]
status: active
---
## 2026-06-24 — doctor and freeze stay separate diagnostics

**Decision:** `mochiflow doctor` remains the project health check. When it is
run from the MochiFlow source repository, it only guides users toward
`mochiflow freeze --check` for source-derived files. Context freshness warnings
scan references to public CLI subcommands and workflow verbs, while
`mochiflow freeze --root <source-repo> --check` gives scripts an explicit source
root without changing the project-health surface.

**Why:** Project consumers can legitimately mention workflow commands such as
`mochiflow discuss` and `mochiflow build` even though those are not clap
subcommands. Treating only terminal subcommands as valid would create false
stale-context warnings. Conversely, folding freeze drift into doctor would mix
consumer project validation with source-repo release hygiene.

**Rejected:** A hardcoded doctor allowlist with no clap parity test (would drift
silently); walking upward from an explicit `--root` path (could validate a parent
source repo when the supplied path is wrong); adding a separate `--config` option
before a concrete scripting need exists.

**Consequence:** Doctor's allowlist is split between terminal CLI commands and
workflow command references, with a test that keeps the terminal set aligned
with clap. Freeze root validation is exact marker validation against
`cli/Cargo.toml` and `engine/VERSION`.
