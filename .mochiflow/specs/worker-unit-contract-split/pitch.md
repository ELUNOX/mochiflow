# Name the worker execution units: build-task vs rework

## Problem

The delegated build orchestrator (shipped 2026-06-28,
`build-orchestrator-disposable-workers`) introduced one `worker` role reused by
`build`, `open`, and `update`. The word "worker" silently covers two different
execution units: a `build` task (`T-###` with a `tasks.md` row, checkbox, and
`Task:` trailer) and a bounded `open`/`update` fix (QA-`FAIL` rework or
PR-feedback, with no task row, no checkbox, no trailer).

`engine/agents/worker.md` already branches on the host phase in prose, but the
two units have **no name**, and the behavioral split is dispatched implicitly by
parsing the `unit` id prefix (`T-###` vs `qa-fail:` / `pr-feedback:`). That
prefix is not a clean 1:1 with the two behaviors (rework spans two prefixes), so
readers, reviewers, and the worker itself must infer "which kind of unit am I"
from string shape. This keeps the contract honest-by-convention rather than
honest-by-construction, which is the residual ambiguity the original review
flagged.

## Appetite

Small. A prose/contract clarification in the engine docs plus conformance
wording — no Rust struct, serialization, or runtime code change, and no new
generated agent file. One standard-risk spec.

## Solution

Make the two execution units explicit in the worker contract, keeping a single
shared role, a single shared delegation transport, and the single generated
`spec-worker.json` agent.

- Introduce `unit_kind` as the **primary behavioral discriminator** in the
  worker contract: `unit_kind ∈ {build-task, rework}`.
  - `build-task`: one `tasks.md` task; ticks its checkbox and commits with a
    `Task:` trailer (build only).
  - `rework`: one bounded `open` QA-`FAIL` fix or `update` PR-feedback fix; no
    checkbox, no `Task:` trailer; commits per the host verb's convention.
- The worker's behavior (checkbox + `Task:` trailer vs host-verb commit
  convention, STOP routing destination) is selected by `unit_kind`, not by
  parsing the `unit` id prefix. The `unit` id stays as the human-readable
  identifier (`T-###` / `qa-fail:<id>` / `pr-feedback:<id>`).
- The compact report keeps a **single uniform schema** across both kinds, adding
  `unit_kind` as the only new field: `unit_kind`, `unit`, `status`,
  `files_changed`, `verify`, `commit` (on `done`), `reason` (on `blocked`). No
  unit-specific fields (`feedback_id` / `qa_item` / `task`) — they collapse into
  `unit`.
- Naming style is tokens + plain prose: `unit_kind` values `build-task` /
  `rework`, referred to in prose as "build-task worker" / "rework worker" — not
  CamelCase type names, because the contract is agent-to-agent prose with no
  corresponding Rust type.
- `unit_kind` is set at dispatch time by the orchestrator (build) or the host
  verb (open/update) in the context pack; the generated `spec-worker.json`
  agent, its tools, and its model are unchanged.

Surfaces touched: `engine/agents/worker.md` (primary), the worker references in
`engine/commands/build.md` / `open.md` / `update.md`, and the worker-related
conformance wording. Per the dogfood constitution, an `engine/` edit requires
`mochiflow freeze` → `mochiflow upgrade --source engine` →
`mochiflow adapter generate --check` before final verification.

## Rabbit Holes

- Do not split into a second `agents/rework-worker.md` role or a second
  delegation transport. `build-orchestrator-disposable-workers` already rejected
  that (duplicate selection/fallback logic, SSOT violation); the split here is
  in the *unit*, not the role or transport.
- Do not turn the compact report into a parsed Rust data structure. It is a
  prose agent-to-agent contract; `unit_kind` is a documented field, not a serde
  type.
- Do not let `rework`'s two host sub-forms (`qa-fail` / `pr-feedback`) become a
  third `unit_kind`. They differ only in id prefix and host verb, not in worker
  behavior.

## No-gos

- No second generated Kiro agent and no change to `spec-worker.json` tools /
  model / prompt path.
- No change to the delegation threshold (≥ 2 open tasks) or its decoupling from
  `risk`.
- No runtime/CLI behavior change; engine-doc contract and conformance only.

## Alternatives Considered

- **Do nothing / prose-only rename (option A in discuss).** Rejected: naming the
  units without a discriminator leaves behavior keyed on prefix-parsing, so the
  provenance ambiguity persists.
- **Physical role/agent split (option C).** Rejected: contradicts the shipped
  "one transport, one write-capable agent, roles differ by prompt+scope" ADRs.
- **Per-unit report fields (`feedback_id` / `qa_item` / `task`).** Rejected:
  forces host verbs to read different shapes per unit, re-introducing
  per-unit contracts at the report layer; they collapse into one `unit` id under
  `unit_kind`.

## Open Questions

- None — ready for plan.
