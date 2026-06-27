# Post-build PR and Close Flow

## Problem

MochiFlow's current post-build delivery model overloads `ship` with several
different meanings: final acceptance, PR preparation, PR creation, archiving,
and completion. That creates a mismatch with common PR workflows, where a PR is
still an unmerged proposal that may receive review feedback or CI fixes.

The current model also archives too early. `ship` folds and moves the spec to
`_done/` and sets `status: done` in a close-out commit *before* the PR is even
opened. When review feedback arrives, that completed work has to be resurrected
(moved back out of `_done/`, `done → approved`) to be fixed — backwards for a
still-open review. The root cause is that lifecycle state is expressed by
*writing* a terminal state into tracked files, which forces the write to happen
pre-merge (since the base branch cannot be committed to directly).

The desired user experience is also state-driven, not command-by-command. After
implementation work completes, MochiFlow should offer the next safe delivery
action based on repository state. Release handling remains a separate, explicit
capability and is out of scope for this change.

## Appetite

This is worth a workflow-level redesign because it changes lifecycle semantics,
router behavior, command documentation, CLI handoff behavior, the durable state
model, and the board/index representation. Keep the implementation focused on the
build-to-PR-to-close path; do not try to solve release automation at the same
time.

## Solution

Replace the user-facing `ship` concept entirely with a post-build flow composed
of three state-driven actions, backed by a state model that separates *asserted*
states from *derived* delivery facts.

### Three actions

- `build` finishes implementation and local verification and records the AC
  Matrix. It ends at `approved`; it does not create a PR.
- `open` runs final acceptance (the human QA round-trip), sets `accepted`, folds
  durable knowledge (ADR decisions / pitfalls) *into the PR*, generates the PR
  title/body, takes human approval (the approve-PR gate), pushes, and creates the
  PR. PR creation stays gated because it has external effects.
- `update` handles review feedback, CI failures, and PR-body corrections while
  the work is in review. Code changes are delegated through the same `build`
  loop (not reimplemented); it re-verifies, pushes, and updates PR metadata. The
  spec is not moved and is never resurrected.
- `close` runs after the PR is confirmed merged. It performs local hygiene only:
  switch to base, fast-forward pull, delete the local branch, clear ephemeral
  state, and regenerate the board. It writes nothing to the base branch — the
  fold was already merged via the `open` PR.

The primary UX is the next-action prompt: after build, "Create the PR"; on
feedback, "Update the PR"; after merge, "Close the work". Direct command entry
still exists for advanced use.

### State model: asserted vs derived

- Asserted states are stored in `spec.yaml` and all settle *before* merge on the
  feature branch via normal PRs: `draft → approved → accepted`. `accepted` is a
  quality state (AC Matrix all done-eligible, plus the reviewer verdict when
  `risk ≥ elevated`), not a new human gate.
- Delivery states are *derived*, never stored: `in_review` (a PR is open) and
  `merged`. "Done" is observed from VCS/PR reality, not written into a file. This
  removes every post-merge write to the base branch, and with it the
  archive-before-PR and resurrection problems.

`merged` derivation signal priority: provider API (e.g. GitHub) → a
`Spec: {slug}` trailer present in `origin/{base_branch}` history → the human
merge report as the final fallback. Contract: a merge must leave the
`Spec: {slug}` trailer in the base branch — merge/rebase preserve it
automatically; squash must carry the trailer into the squash commit.

### Flat specs and the board

- A spec lives at a single flat location `{specs_dir}/{slug}/` for its whole
  life. There is no per-state folder and no `_done/` move. Existing `_done/`
  archives remain read-only.
- The kanban is computed, not stored: `mochiflow status` renders the live board
  on demand (with `--fetch` for network-accurate derivation). `INDEX.md` is a
  gitignored generated artifact, regenerated automatically at the end of every
  mochiflow command. There are no git hooks.

### Stale-base guard

Starting a new spec fetches and branches from `origin/{base_branch}` and warns
when the local base is behind, independent of the provider. This reduces the
"forgot to report merge → new spec on a stale base" accident without depending on
the human merge report.

Plan will specify the mechanism details: exact `merged`/`in_review` derivation
commands (including `provider = none` fidelity), the `accepted` lint gate, the
board rendering format, and how stored `done` in legacy `_done/` specs maps to
the derived model.

## Rabbit Holes

- Do not design release automation here. `release` stays an independent command
  and a separate future feature.
- Do not reintroduce directory-as-state: no `_done/` move, no symlinked
  projection board, no committed live kanban file.
- Do not require any base-branch commit or push for a lifecycle transition.
- Do not require users to memorize low-level command names when the active spec
  and PR state can offer the next safe action.

## No-gos

- No package publishing, tag creation, GitHub Release creation, production
  deploy, or release-note automation in this change.
- No PR-provider replacement. MochiFlow orchestrates around provider state and
  handoff contracts; it does not become a review system.
- No automatic PR creation without human approval of the generated title/body.
- No direct base-branch commit or push for archive or completion bookkeeping.
- No git hooks, no symlinked board, and no committed live kanban file.

## Alternatives Considered

- Keep `ship` and make it smarter. Rejected — the word stays overloaded (open PR
  / merge / release / deploy).
- Keep the archive-before-PR model. Rejected — PR feedback then has to resurrect
  completed work.
- Store `done` / move to `_done/` at `close` (post-merge). Rejected — it requires
  a base-branch commit, which is disallowed; an unpushed local commit diverges
  the base and breaks `pull --ff-only`.
- A follow-up archive PR after merge. Rejected — an extra bookkeeping PR per spec.
- Optimistic archive in the delivery PR plus in-place feedback. Rejected as the
  primary model — the spec appears done while still under review and it keeps
  directory-as-state.
- A gitignored symlink projection board (`specs/` + `board/`). Rejected — two
  parallel folder trees confuse users, even though navigation worked.
- A committed live `INDEX.md` kanban. Rejected — it reintroduces base-branch
  writes for the merged state and cross-spec merge conflicts.
- Git hooks to refresh the board on merge/checkout. Rejected — setup complexity;
  rely instead on per-command regeneration plus on-demand `mochiflow status`.

## Open Questions

- None -- ready for plan.
