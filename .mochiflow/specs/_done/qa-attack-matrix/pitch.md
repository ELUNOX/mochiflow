# Add a QA attack matrix to plan and reviewer flows

## Problem

The independent reviewer checks spec conformance and code quality, but nothing
forces adversarial "do not trust that it works" thinking *before* implementation.
When break-attempts happen only during code review, defects surface late and can
force re-planning. The workflow needs both halves: plan captures feature-specific
attack attempts and required evidence, and the reviewer verifies those attacks
were actually exercised and that the evidence proves the claim. This came from
the backlog seed `qa-attack-matrix` (source: conversation).

## Appetite

Small, docs-and-templates-only change to the engine prose. No Rust/CLI code, no
schema, no `contracts.lock` change. The seven-persona heuristic (P1 new user,
P2 power user, P3 malicious user, P4 data integrity, P5 migration, P6 regression,
P7 spec skeptic) becomes a lightweight authoring + review convention, not a new
enforced subsystem.

## Solution

Reuse existing machinery instead of inventing a parallel artifact:

- **Home / shape**: extend the existing `## QA Scenarios` table in `spec.md`
  with a persona dimension (P1-P7). No separate `## QA Attack Matrix` section and
  no metadata-generated checklist. `spec.md` always exists (micro and up), so
  standard specs are not left without attack coverage.
- **Risk scaling**: attack coverage strength follows `risk` (matching
  `reference/risk.md`). `standard` treats all seven personas as considered but
  allows reasoned `N/A: <reason>`, with at least the change-relevant personas
  (default set P1 / P3 / P6 / P7) exercised. `elevated` requires evidence for the
  relevant personas (especially P3 / P4 / P5); `N/A` needs a concrete reason.
  `critical` requires strong evidence (test output, logs, human confirmation) and
  does not accept casual `N/A`.
- **Reviewer integration**: extend the reviewer's existing Stage 1 (Spec
  Conformance) to also verify (1) the risk-appropriate persona rows exist,
  (2) each `N/A` carries a concrete reason, and (3) exercised rows have evidence
  that actually backs the attack. No new reviewer stage; Stage structure, Finding
  shape, and Completion output format stay unchanged.
- **AC Matrix interaction**: attacks are not promoted to formal ACs. Each attack
  is a `QA-XX` row in `## QA Scenarios`. When an attack backs an AC, reference its
  `QA-XX` in the AC Matrix `Planned test/QA` or `Evidence` column. `## QA
  Scenarios` carries no result columns (the AC Matrix is the results ledger), so
  a purely exploratory attack that backs no AC stays scenario-only and does not
  mint an AC row unless it surfaces a defect worth its own AC.

Files in scope (edit repo-root `engine/` source, then sync to vendored copy):
`engine/commands/plan.md`, `engine/agents/independent-reviewer.md`,
`engine/reference/risk.md`, `engine/templates/spec/spec.standard.md` (and
`spec.md` if needed). Micro specs keep persona coverage optional.

## Rabbit Holes

- Editing the vendored `.mochiflow/engine/` copy instead of repo-root `engine/`.
  Source is repo-root `engine/`; after edits run the constitution dogfood steps
  (`mochiflow freeze` -> `mochiflow upgrade --source engine` ->
  `mochiflow adapter generate --check`) before final verification.
- Template changes to `spec.standard.md` / `spec.md` may touch conformance golden
  fixtures or `engine/MANIFEST.json`; handle regeneration during build.
- Over-formalizing: do not turn every persona into an AC or a new ID system; keep
  `QA-XX` as the single trace unit.

## No-gos

- No new lint rule in the Rust CLI (`mochiflow-core`) for persona coverage in this
  spec; reviewer Stage 1 is the enforcement. Machine enforcement, if wanted, is a
  separate follow-up backlog seed.
- No new reviewer stage and no change to the reviewer output contract.
- No new AC Matrix column or token, and no `ATK-XX`-style parallel ID scheme.
- No `design.md` (standard risk, single surface, integration none).

## Alternatives Considered

- Metadata-generated attack checklist from risk/surface: DRY but prone to becoming
  paperwork the seed itself warns about, and hard to make evidence-bearing.
- `design.md`-only attack section: absent for standard specs, exactly where naive
  defects concentrate.
- Separate "QA Attack Review" reviewer stage / splitting Stage 1: large blast
  radius on the reviewer output contract and adapters, with responsibility
  overlapping the existing Stage 1.
- Requiring all seven personas for every spec: formalizes trivial changes and
  dilutes weight on high-risk work.
- Adding CLI lint enforcement now: pulls in `contracts.lock` / `engine/VERSION`
  churn before the convention has settled (raises risk).

## Open Questions

- None - ready for plan.
