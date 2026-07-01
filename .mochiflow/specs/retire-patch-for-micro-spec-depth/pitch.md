# Retire patch and redefine micro as the fastest spec depth

## Problem

The `patch` lane is fast because it leaves the spec lifecycle entirely: patch
changes have no `spec.yaml`, no AC Matrix, no PR, and no accepted state. That
creates a routing and delivery fork — some changes are traceable specs and some
are not. The parallel no-PR fast path is the same fork living inside the spec
lane. We want small, concrete fixes to stay lightweight without leaving the
traceability model.

## Appetite

Medium. This is an engine + CLI restructuring across the router / workflow / git
/ plan / build / discuss docs plus a few CLI modules (doctor, adapter,
conformance, lint), and template / CHANGELOG updates. No schema change and no
data migration.

## Solution

Retire `patch` as a non-spec lane and redefine the existing `micro` depth as the
smallest depth inside the single spec lane. Today the engine already mentions
micro in `workflow.md`, `plan.md`, `risk.md`, `git.md`, `build.md`, and the
`spec.micro.md` template, but that partial definition still assumes
`pitch.md + spec.md` and coexists with patch/no-PR forks. The depth ladder
becomes `micro < standard < design < critical`.

- Micro artifact set: `spec.yaml` + `spec.md` only (no `pitch.md`, `tasks.md`,
  or `design.md`). `plan` may create a micro spec directly from a concrete
  request with no prior `pitch.md`, and `discuss` is optional for micro. The
  `pitch.md` prerequisite applies to `standard` and larger.
- Depth detection: inferred from which documents exist; no new metadata field.
  `risk` stays the stored gate axis and `depth` is an emergent descriptor. lint's
  draft validation branches over three shapes: pitch-only, spec.md-expanded (with
  pitch), and micro (spec.md without pitch).
- ADR fold: micro carries no fold by definition. Discovering durable rationale or
  a pitfall worth an ADR is an escalation signal — escalate the spec in place to
  `standard` or higher, adding whatever that depth requires, and fold normally.
- Delivery: remove the no-PR fast path. Every spec, micro included, delivers via
  a feature branch + PR, so the approve-PR gate always applies.
- Retired `mochiflow-patch`: recognized as a deprecated trigger — a one-line
  notice, then route to `plan` (which picks micro when eligible). There is no
  on-disk migration because patch produced no artifacts, and no permanent
  compatibility alias.
- Router: with no active spec, concrete small-fix phrasings (for example
  "直して" / "quick fix", or an explicit no-spec request) become `plan` intent
  hints — propose to start planning and wait, with no auto-activation. With an
  active spec, fixes still route to `build` (approved / implementing) or `update`
  (in review), unchanged.

## Rabbit Holes

- Do not add a `depth` field to `spec.yaml`; that would churn the frozen contract
  surface and create a second source of truth that can contradict the files.
- Do not add a `micro` command or new trigger vocabulary; small fixes route
  through `plan`.
- Do not auto-activate a spec on a bare "直して"; keep the no-activation-without-
  explicit-intent discipline.
- Keep `risk` (standard / elevated / critical) and depth (micro and up) as
  distinct axes even though the names overlap; do not merge them.

## No-gos

- No permanent `mochiflow-patch` compatibility alias.
- No no-PR delivery path for any depth.
- No schema / `contracts.lock` / `engine/VERSION` change driven by this spec
  (engine markdown edits still require `mochiflow freeze`).
- No retroactive rewrite of specs already authored under the old model.

## Alternatives Considered

- Keep `patch` as a thin alias for micro — rejected: preserves the dual-lane
  routing and delivery fork this redesign is meant to remove.
- Add an explicit `depth` metadata field — rejected: touches the frozen contract
  surface (schema + lock + VERSION) and introduces file-vs-field drift.
- Mechanically forbid fold for micro — rejected: loses genuine knowledge found
  mid-fix; escalation preserves it in the right place.
- Keep an opt-in no-PR fast path for micro — rejected: leaves a delivery fork;
  unification and auditability win over shaving one PR.
- Auto-activate plan / micro on a bare "直して" — rejected: violates the
  explicit-intent activation principle.

## Open Questions

- None — ready for plan.
