---
slug: "independent-reviewer-grounded-redesign"
title: "Redesign the independent reviewer as a grounded adversary"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_spec: "retire-patch-for-micro-spec-depth"
source_phase: "review"
created: "2026-06-30"
updated: "2026-06-30"
---

# Redesign the independent reviewer as a grounded adversary

## Signal

The independent reviewer reads a spec as a self-contained document. It therefore
structurally misses defects that only appear when the spec is read as a change
proposal against the actual repository. During the
`retire-patch-for-micro-spec-depth` ad-hoc review, several High-severity
conflicts were found only by grounding the spec's claims in real `engine/` and
`cli/` files, sweeping the repo for impact, and confronting the spec with a
recorded pitfall — none of which the current reviewer is equipped to do.

## Why It Matters

These are exactly the failures that should be caught at plan time, before build.
If the reviewer could ground claims in code, sweep impact across the repo, and
confront durable ADR pitfalls, plan-quality review would automatically catch
"the spec says the current state is X but the code says Y", "this change
silently breaks existing conformance test Z", and "this change repeats a known
pitfall" — instead of depending on an unusually thorough human pass.

## Evidence

- `engine/agents/independent-reviewer.md`: plan-quality mode is defined as
  "judge the spec artifacts alone" (no diff / changed-files / repo input), so
  out-of-spec conflicts are invisible by construction.
- `engine/adapters/kiro/agents/spec-independent-reviewer.json.tpl`:
  `tools: ["read"]` and a fixed three-file `resources` list (only
  `independent-reviewer.md` / `workflow.md` / `language.md`; `risk.md` is not
  even included). No search capability and no ADR / pitfall inputs.
- The reviewer's `references` are `language` / `workflow` / `risk` only; the ADR
  decisions / pitfalls stores are never loaded, so any "knowledge confrontation"
  pass is impossible today.
- `retire-patch-for-micro-spec-depth` review surfaced conflicts the current
  reviewer cannot reach: an existing `engine/reference/workflow.md`
  Depth-scaling "Micro spec = pitch.md + spec.md" row contradicting the spec's
  redefinition; `cli/crates/mochiflow-core/src/lint.rs` draft branching that
  makes a micro draft indistinguishable from a pitch-less standard draft;
  reverse-pinned conformance tests (`micro_template_has_no_ac_verification_matrix`,
  `router_preserves_named_routing_branches`); and pitfall
  `2026-06-28-conformance-substring-line-wrap`, whose recorded broken example is
  the exact string the spec removes.

## Open Questions

- Direction to explore is a "grounded adversary" reviewer: grounding-first
  (verify each spec claim against code, list unverifiable claims), repo-wide
  impact sweep (grep the retired / renamed concept and check that task `Files`
  cover every occurrence), knowledge confrontation (area-intersecting ADR /
  pitfalls), and falsification (construct one counter-example per major claim).
  What is the minimal stage set, and how do the two current modes
  (post-implementation / plan-quality) fold into it?
- How can read / search scope be granted safely given Kiro's coarse tool
  categories (pitfall `2026-06-28-kiro-agent-tools-are-coarse-categories`) while
  keeping the reviewer strictly read-only (never write)?
- How should the cost of reading the repository be bounded — scoped to the
  spec's `surfaces` and declared `Files`, or the whole tree?
- How can predicted-regression findings be kept from over-firing (separate
  "predicted" vs "confirmed"; require grounding evidence)?
- How should the redesign's own conformance be pinned without repeating the
  literal-substring fragility recorded in `2026-06-28-conformance-substring-line-wrap`?
- Confirm this is a plan-track change (engine contract + multi-surface: engine
  doc / Kiro template / conformance + reviewer responsibility and permission
  relocation, so risk is likely elevated), not a patch.
