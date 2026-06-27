# Make ship archive staging resilient to moved spec paths

## Problem

Ship close-out moves an active spec directory from `{specs_dir}/{slug}/` to
`{specs_dir}/_done/{slug}/`. Agents can then try to stage the close-out with a
pathspec that names the moved-from directory, such as
`git add -A {specs_dir}/{slug} {specs_dir}/_done/{slug}`. Once the source path no
longer exists in the working tree, Git can reject that pathspec even though the
intended change is a normal tracked directory move.

This makes the final handoff noisy and pushes agents toward broader staging
commands. The source of truth should be a narrow, repeatable close-out staging
operation that captures the archive move without risking unrelated user changes.

## Appetite

This is worth a real design pass, not a minimal wording patch. The change should
leave ship close-out mechanically boring for every adapter and should be backed
by tests, because the failure happens at the point where the branch is expected
to be clean and ready for PR handoff.

## Solution

Redesign close-out staging around Git's standard model:

- use `git add -A` with a stable parent pathspec for lifecycle artifacts, so
  deletions and additions under `{specs_dir}` are staged together;
- make the CLI own the deterministic ship close-out mechanics through
  `mochiflow ship`, rather than making adapters reason about moved pathspecs;
- have `mochiflow ship` infer the active spec from the current feature branch
  when possible, with an explicit slug argument available for ambiguous or
  scripted use;
- have the helper stage only the configured lifecycle paths: `{specs_dir}`,
  `{index}`, and configured ADR files, while keeping `state/` and unrelated
  source changes out of the staging set;
- validate the result with machine-readable Git output, using
  `git status --porcelain` / `git diff --cached --name-status` style checks, so
  the helper can reject missing or partial archive moves before `mochiflow pr`;
- update `ship` guidance to call the CLI command and document the safe manual
  fallback as `git add -A {specs_dir} {index} {adr_paths...}`;
- update `doctor` or the relevant quality check to flag a done-spec move that
  is present in the working tree but not fully staged/committed when close-out
  is in progress;
- update PR pre-flight so it verifies the close-out archive move is already
  committed before pushing when a spec slug is supplied.

Primary sources for the design basis:

- Git `add` documentation: `git add -A` updates the index for matched paths,
  including removals, and accepts pathspecs to constrain scope
  (https://git-scm.com/docs/git-add).
- Git `status` documentation: porcelain status is intended for stable,
  script-readable output (https://git-scm.com/docs/git-status).
- Git `diff` documentation: staged changes can be inspected with cached diff
  modes suitable for verifying name/status changes
  (https://git-scm.com/docs/git-diff).

## Rabbit Holes

- Do not make `mochiflow ship` replace the agent-driven workflow phase in this
  change. The command should cover deterministic close-out mechanics only:
  staging, validation, and any commit preparation that does not cross a human
  approval gate.
- Do not solve unrelated PR-provider, branch cleanup, or post-merge lifecycle
  gaps here.
- Do not replace Git's index model with a custom file ledger. The helper should
  compose standard Git commands and validate their observable result.

## No-gos

- Do not recommend or run repository-wide `git add -A`.
- Do not hard-code `.mochiflow/specs`; use the configured `{specs_dir}` and
  configured ADR/index paths.
- Do not stage `state/`, PR body scratch files, unrelated source changes, other
  active specs, or unrelated completed specs.
- Do not let `mochiflow pr` push a branch when a requested spec's close-out move
  is missing from the committed diff.

## Alternatives Considered

- Documentation-only guidance: rejected because the same pathspec mistake is
  easy for agents to repeat under pressure, especially after `git mv` has
  removed the source path.
- Add a narrowly named helper such as `mochiflow close-out stage` or
  `mochiflow ship stage`: rejected because it exposes implementation details
  and creates vocabulary that users should not need. The public command should
  be the workflow concept, `mochiflow ship`; internally it can perform safe
  staging and verification.
- Name the command `mochiflow deploy`: rejected because deploy usually means
  releasing an application or artifact into a runtime environment. This work
  prepares MochiFlow's repository handoff and PR state; it does not deploy the
  user's product, publish a package, or change an environment.
- Teach every adapter a staging recipe: rejected because adapter instructions
  are harder to keep coherent than a tested CLI helper plus one shared engine
  reference.
- Rely on `git mv` staging alone: rejected because close-out also includes
  `spec.yaml`, the AC Matrix, ADR files, and regenerated index changes that may
  be edited after the move.

## Open Questions

- None — ready for plan.
