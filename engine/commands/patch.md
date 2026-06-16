---
name: mochiflow-patch
description: |
  MochiFlow's patch lane. Apply a concrete, local, reversible fix without
  creating a spec. This is a non-phase command for standard-risk changes that
  follow existing intent and need no new product/design decision. Activate on
  the explicit command `mochiflow-patch`, or natural phrasing like "patchして" / "quick fix". It does not create spec
  artifacts, change spec status, fold ADR, archive, or create PRs.
triggers:
  - mochiflow-patch
  - patchして
  - quick fix
trigger_patterns: []
artifacts: []
prerequisites: []
execution: inline
allowed_writes:
  - "{write.allow}"
forbidden_writes:
  - "{specs_dir}/**"
  - "{constitution.project}"
  - "{constitution.local}"
  - "{context.product}"
  - "{context.structure}"
  - "{context.tech}"
  - "{adr.decisions}"
  - "{adr.pitfalls}"
  - "{index}"
  - "{install_dir}/state/**"
  - .git/**
references:
  - reference/workflow.md
  - reference/git.md
  - reference/risk.md
  - reference/engineering-standards.md
---

# patch

## Purpose

Make a small implementation change without creating or advancing a spec. Patch
is for concrete, local, reversible work that follows the current code's existing
intent. It is not a faster build phase; it is a non-spec lane.

## Eligibility

Patch is allowed only when all of these are true:

- The requested change is concrete enough to implement without discovering new
  product intent.
- The expected behavior is already clear from the request, tests, current code,
  or existing documentation.
- The blast radius is local, reversible by git revert, and equivalent to
  `risk: standard`.
- The change fits within one surface and does not alter public contracts.
- No durable rationale, pitfall, migration, or acceptance ledger is needed.

Patch is not allowed when the work touches or requires any of:

- new feature scope, product/design decision, or unclear expected behavior;
- public API, schema, config/spec format, contract, adapter contract, or
  compatibility policy change;
- data migration, persistence lifecycle, deletion/archival behavior, auth,
  permissions, security, payment, or user-data loss risk;
- multiple surfaces, cross-module responsibility relocation, or wide refactor;
- human/visual acceptance that needs a durable QA record.

When eligibility fails or becomes uncertain, stop and propose `Start plan?`.

## Procedure

1. Read the constitution (`[constitution].project` / `[constitution].local`) and foundational context (`[context].product` / `[context].structure` / `[context].tech`)
   and inspect the smallest relevant slice of code. Do not create spec files.
2. Run `git status --short` before editing. Identify intended target files.
   - If an intended target file is already dirty, continue only when the edit can
     be made without overwriting the existing changes; auto-commit is disabled.
   - Ignore unrelated dirty files and never stage them.
3. Apply the minimal change that satisfies the concrete request. Match existing
   patterns and stop if a new decision or higher-risk behavior appears.
4. Verify using `reference/workflow.md ## Patch verification`.
5. If verification fails, fix and re-run until PASS, or report the blocker.
6. When verification PASSes and no intended target file was dirty before the
   patch, commit the changed files explicitly per `reference/git.md ## Patch
   commit`. If any intended target file was already dirty, do not commit; report
   the changed files and verification result.
7. Summarize what changed, what was checked, and whether a commit was created.

## Stop Conditions

- Do not write `{specs_dir}/`, `[constitution]`, `[context]`, `[adr]`, `{index}`, PR metadata, or
  delivery state.
- Do not create, update, approve, ship, archive, or fold any spec.
- Do not create a branch or PR.
- Do not stage unrelated files or files that were already dirty before patch
  started.
- Do not continue patch after discovering elevated/critical risk; propose `plan`.
