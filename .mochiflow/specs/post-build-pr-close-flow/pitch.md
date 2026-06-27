# Post-build PR and Close Flow

## Problem

MochiFlow's current post-build delivery model overloads `ship` with several
different meanings: final acceptance, PR preparation, PR creation, archiving,
and completion. That creates a mismatch with common PR workflows, where a PR is
still an unmerged proposal that may receive review feedback or CI fixes.

The desired user experience is also not command-by-command. After implementation
work completes, MochiFlow should guide the user through the next safe delivery
action based on repository state: prepare a PR, open it after approval, handle
review/CI updates, and close the work after the PR is merged. Release handling
should remain a separate, explicit capability and is out of scope for this
change.

## Appetite

This is worth a workflow-level redesign because it changes lifecycle semantics,
router behavior, command documentation, CLI handoff behavior, and archived spec
state. Keep the implementation focused on the build-to-PR-to-close path; do not
try to solve release automation at the same time.

## Solution

Replace the user-facing `ship` delivery concept with a post-build flow composed
of state-driven actions:

- `build` finishes implementation and local verification, records acceptance
  evidence, and leaves the work ready for PR preparation.
- `open` prepares the PR title/body, presents it for human approval, then pushes
  and creates the PR. PR creation remains gated because it has external effects.
- `update` handles review feedback, CI failures, and PR-body corrections while
  the work remains in review. It applies fixes through the same spec context,
  re-verifies, commits, pushes, and updates PR metadata when needed.
- `close` runs after the PR is confirmed merged. It records durable learnings,
  archives the spec, updates the index, clears local state, and performs local
  branch cleanup.

The primary user-facing flow should not require users to type each internal
action name. MochiFlow should offer the next action from the current state via
plain-language prompts and choice cards. For example, after build completes it
should present "Create the PR" rather than requiring `mochiflow pr open`; after
review feedback it should offer "Update the PR"; after merge it should offer
"Close the work".

The durable state model should reflect the distinction between implemented,
in-review, merged, and closed work. The exact metadata representation can be
decided during plan, but the key invariant is that work is not archived merely
because a PR was opened. Archiving belongs to `close`, after merge confirmation.

## Rabbit Holes

- Do not design release automation here. `release` should remain an independent
  command and separate future feature.
- Do not preserve `ship` as a user-facing synonym if it keeps ambiguous meaning.
  It can remain only as a compatibility alias if plan decides the migration
  needs one.
- Do not require users to remember a sequence of low-level commands when the
  active spec and PR state can determine the next safe action.
- Do not move a spec to completed storage before PR review and CI feedback have
  finished.

## No-gos

- No package publishing, tag creation, GitHub Release creation, production
  deploy, or release-note automation in this change.
- No PR-provider replacement. MochiFlow should orchestrate around provider state
  and handoff contracts, not become a full review system.
- No automatic PR creation without human approval of the generated title/body.
- No automatic external publish/deploy side effects.

## Alternatives Considered

- Keep `ship` and make it smarter. Rejected because the word remains overloaded:
  users can reasonably read it as "open PR", "merge", "release", or "deploy".
- Keep the current archive-before-PR model. Rejected because PR feedback then
  has to resurrect completed work, which is backwards for a still-open review.
- Expose only explicit commands such as `mochiflow pr open` and
  `mochiflow pr update`. Rejected as the primary UX because the desired flow is
  state-driven. Explicit commands may still exist as advanced/direct entry
  points.
- Fold release into close. Rejected because release is a product delivery unit,
  often spanning multiple merged changes, and should be configured and approved
  independently.

## Open Questions

- None -- ready for plan.
