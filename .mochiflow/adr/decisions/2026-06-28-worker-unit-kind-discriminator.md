---
id: 2026-06-28-worker-unit-kind-discriminator
date: 2026-06-28
area: [cli]
spec: worker-unit-contract-split
status: superseded
superseded_by: 2026-06-29-retire-worker-unit-kind
---
## 2026-06-28 — Name worker execution units by an explicit unit_kind discriminator

**Decision:** The single shared `worker` role (reused by build / open / update
over one delegation transport) now distinguishes its two execution units by an
explicit **`unit_kind` ∈ {build-task, rework}** carried in the context pack,
instead of inferring behavior from the `unit` id prefix. `build-task` ticks the
`tasks.md` checkbox and commits with a `Task:` trailer; `rework` (open QA-`FAIL`
/ update PR-feedback) has no checkbox and no `Task:` trailer and commits per the
host verb's convention. The compact report is a single uniform schema that adds
`unit_kind` and keeps the human-readable `unit` id; there are no per-unit fields.
One role, one transport, and the single generated `spec-worker.json` agent are
unchanged.

**Why:** `worker.md` already branched by host phase in prose, but the two units
had **no name** and behavior was dispatched implicitly by the shape of the `unit`
id — and `rework` spans two prefixes (`qa-fail:` from open, `pr-feedback:` from
update), so the prefix is not 1:1 with behavior. Naming the kinds and keying
behavior/STOP routing on an explicit discriminator makes the contract
honest-by-construction: readers and dispatched agents no longer infer the unit
from a string-prefix convention.

**Key sub-decisions:**
- Tokens + plain prose (`build-task` / `rework`, "build-task worker" / "rework
  worker"), not CamelCase type names — the contract is agent-to-agent prose with
  no corresponding Rust type.
- `rework`'s two host sub-forms stay **one** kind; they differ only by host verb
  and id prefix, not by worker behavior, so a third kind would be noise.
- The compact report and context pack are prose contracts, not parsed Rust
  types, so `unit_kind` is a documented field — no serde / struct change.

**Rejected:** a second `agents/rework-worker.md` role or a second transport
(already rejected by `2026-06-28-build-orchestrator-disposable-workers` —
duplicate selection/fallback logic, SSOT violation); per-unit report fields
(`feedback_id` / `qa_item` / `task`) — they collapse into the `unit` id under
`unit_kind` and would force host verbs to read per-unit shapes; prose-only naming
with no discriminator — leaves behavior keyed on prefix parsing, so the original
ambiguity is unaddressed.

**Relation:** refines `2026-06-28-build-orchestrator-disposable-workers` (the
worker role and one-shared-transport decision); it does not supersede it.
