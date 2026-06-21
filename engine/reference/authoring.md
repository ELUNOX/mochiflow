# Authoring Reference

How to write `spec.yaml`, `spec.md`, `design.md`, and `tasks.md`. Used by
`plan`, `build`, and `ship`.

## spec.yaml schema

`spec.yaml` is the single source of truth for spec metadata. Markdown files
carry no frontmatter.

```yaml
version: 1
slug: measurement-sync
title: Measurement Sync
type: feature        # feature | fix | refactor | docs | chore
module: InspectionFeature   # optional
surfaces:
  - app              # surface names come from config.toml [surfaces.*]
integration: none    # none | contract | workflow
risk: standard       # standard | elevated | critical (ordered enum)
status: draft        # draft | approved | done
created: YYYY-MM-DD
updated: YYYY-MM-DD
```

`status` flow is `draft → approved → done` (`workflow.md`). Whether `design.md` /
`tasks.md` exist is expressed by file presence, not metadata.

## SSOT discipline

Fix each fact in one place; reference by ID elsewhere.

| fact | source of truth | elsewhere |
| --- | --- | --- |
| AC (pass/fail criteria) | `spec.md` | tasks references use IDs only |
| AC Matrix | `spec.md` | commands update rows, not duplicate tables |
| surface / risk / type / status | `spec.yaml` | not repeated |
| QA scenarios | `spec.md` | `qa-instructions.md` references, does not copy |
| reviewer / git / journal cadence | `risk.md` / `git.md` | not repeated |
| user-authored standing rules | `[constitution]` (project / local), written by the user | always-loaded; never generated from code |
| current-state orientation | code/config, mapped into `[context]` via onboard / `refresh-context` | always-loaded; never folded |
| design rationale / pitfalls history | `[adr]`, appended by ship's fold | on-demand / phase load |

The three durable guidance layers differ by lifecycle: `[constitution]` is
user-authored standing guidance, `[context]` is a code-derived current-state map
refreshed forward, and `[adr]` is dated history folded at ship. Code is always
the source of truth for current state; prose is not.

## Durable vs ephemeral artifacts

A spec folder holds durable artifacts, including the reviewer-facing QA
instructions. PR handoff scaffolding is ephemeral and lives under
`{install_dir}/state/{slug}/`.

| class | artifacts | home | archived |
| --- | --- | --- | --- |
| durable | `spec.yaml` · `spec.md` (incl. AC Matrix) · `design.md` · `tasks.md` · `qa-instructions.md` | `{specs_dir}/{slug}/` → `_done/` | yes |
| ephemeral | PR body file (`pr-body.md`) · `pr-request.json` (pr_driver only) | `{install_dir}/state/{slug}/` | no |

QA role split: `spec.md` QA scenarios are the source of truth for what to test;
`qa-instructions.md` is the durable reviewer-facing guide for how to run and
where to capture evidence; the AC Matrix is the results ledger.

## spec.md

`spec.md` is the product contract. It carries:

- Problem / Goal with explicit non-goals.
- Users / Actors and User Stories.
- Scope In / Out.
- Requirements / Acceptance Criteria table with stable `AC-01` IDs, Type,
  Priority, Requirement, and Verification.
- Business Rules when needed.
- Examples / QA Scenarios with stable `QA-01` IDs and AC references.
- Non-functional Requirements with stable `NFR-01` IDs when risk requires them.
- Open Questions as `[NEEDS-CLARIFICATION: ...]`.
- Verification Plan / AC Matrix created during plan.

Each AC must be independently checkable and should use EARS-style wording when
practical (`THE SYSTEM SHALL`, `WHEN`, `WHILE`, `IF...THEN`, `WHERE`). Prefer
observable behavior over implementation detail.

For a trivial change, `spec.md` may be concise, but it still needs ACs and the
AC Matrix if it goes through the spec lane.

## design.md

Write only when required (`risk.md ## design.md required condition`). It is the
technical contract and carries:

- Decision Summary with alternatives and consequences.
- Current State Scan limited to relevant files, patterns, and constraints.
- Proposed Design components and Integration Contract.
- Failure Modes.
- Migration / Rollout / Rollback and backward compatibility.
- Observability when relevant.
- Test Strategy mapped to AC/NFR IDs.
- Review Results for mandatory reviewer runs.
- Integration Log appended during build when the risk table calls for it.

Do not write AC tables here, duplicate spec scope, or define concrete
classes/structs that implementation must rediscover from source.

## tasks.md

Write when the change is multi-step. It is the executable checklist and carries:

- Execution Rules.
- Defaults (verify command and common stop conditions).
- Dependency-ordered waves.
- Top-level checkbox tasks with `T-001` IDs.
- Each task references AC, NFR, or chore reason.
- Each task has Type, Depends on, Files, Done, and Stop blocks.
- `[P]` parallel marks only when tasks do not share files or dependencies.
- Finalization tasks `T-900+` for Matrix completion and required review.

Mark a task `[x]` only after implementation, verification, and AC Matrix updates
are ready to be committed in the current commit unit. Include the checkbox
update in that same commit whenever practical.

## Consistency check (plan)

After authoring, verify once against the spec's own intent before approval:

- Open Questions are closed; unresolved `[NEEDS-CLARIFICATION: ...]` blocks approval.
- Every AC has a Verification value.
- Every AC appears in the AC Matrix.
- Every AC is covered by at least one task, QA scenario, or explicit QA-only row.
- Every QA scenario references valid AC IDs.
- Every task references AC, NFR, or chore reason.
- Every task has Depends on, Files, Done, and Stop.
- `design.md` decisions do not contradict `spec.md`.
- Required artifacts match risk, integration, and surfaces.
- `spec.yaml` metadata is consistent with artifact content.
- Generated prose follows artifact language; machine-readable IDs/enums remain English tokens.
