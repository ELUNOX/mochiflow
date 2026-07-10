---
name: spec-close
phase: close
description: |
  mochiflow's close action. Run after the PR is confirmed merged. It performs
  local hygiene only: switch to base, fast-forward pull, delete the local branch,
  clear ephemeral delivery state, and regenerate the board. It writes nothing to
  the base branch — the fold and the spec were already merged via the `open` PR.
artifacts: []
prerequisites:
  - "The PR is merged (the human reports it; `merged` is derived, never stored)"
execution: inline
load:
  required:
    - reference/delivery.md
    - reference/presentation.md
---

# mochiflow-close

## Purpose

Local hygiene after a merge. `close` writes nothing to the base branch: the
living-spec fold and the spec already merged via the `open` PR's close-out
commit. `close` **delegates nothing** — it is deterministic local hygiene with
no code change, so there is no worker dispatch and no separate delegation path.
The human merge report only initiates `close` locally; it is never
persisted as a merged signal (`merged` is derived from the provider or the
tracked-mode `Spec: {slug}` trailer reachable from `origin/{base_branch}`; in
local mode without provider state, it can also be derived while the local source
branch tip is reachable from `origin/{base_branch}`).

## Procedure

Run `reference/delivery.md ## Post-merge local cleanup`:

1. `git status --short` clean — else stop.
2. `git switch {[git].base_branch}`.
3. `git pull --ff-only origin {[git].base_branch}` — stop if ff-only fails
   (divergent local).
4. `git branch -d {prefix}/{slug}` (safe delete; fails if unmerged → leave it,
   ask the human). Resolve `prefix` from `type`: `feature` → `feat`; all other
   types use `type` as-is. For local mode without provider merge state, do this
   only inside close: deleting the branch before close removes the local-git
   merge signal.
5. Clear the spec's ephemeral delivery scratch: `rm -rf {install_dir}/state/{slug}/`
   (gitignored — PR body / `pr-request.json` are not archived).
6. Regenerate the board (`mochiflow index`) so the gitignored `INDEX.md` reflects
   the now-merged (derived Done) state. `INDEX.md` is never staged or committed.

## Presentation

- Describe close as wrapping up the merged work locally in the conversation
  language. Use `close` only for the command or when the user uses it.
- At close start, tell the user in conversation-language plain wording that this
  is post-merge local cleanup — switching to the base branch and tidying up the
  local feature branch and temporary delivery files — and that nothing is
  written to the base branch (the work already merged via the PR).
- At close completion, report in conversation language that the base branch was
  updated, the local feature branch and temporary delivery files were cleaned
  up, and the work is locally wrapped up. Its merged state is observed, not
  written.

## Stop conditions

- Do not commit or push anything to the base branch — the fold and spec are
  already merged via the PR; close is local hygiene only.
- Do not move the spec into `_done/` and do not write `status: done`. The spec
  stays flat; its merged state is observed, not written.
- Knowledge discovered at or after merge is routed to a follow-up (a small `fix`
  spec for code, or a backlog seed for pure rationale), never appended to the
  merged spec.
- A spec whose PR merged but whose `close` never ran still shows as Done in
  `mochiflow status` (derived), independent of `close`.
