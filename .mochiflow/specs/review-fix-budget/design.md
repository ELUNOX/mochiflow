# Add review budget loops with optional automatic fixes — Design

## Design Decisions

- Keep one public verb: `review`. The fix-capable forms are expressed as
  `{slug} review fix [N]`, not as a new lifecycle verb.
- Treat the number after `fix` as a fix-round budget. This matches the user's
  intent to end after the requested automatic corrections, rather than forcing a
  final post-fix review pass.
- Keep `{slug} review` report-only so users can pass findings to another agent
  or human.
- Keep reviewers read-only and independent. The main agent fixes; later
  reviewers see current artifacts/diff and cycle-local focus input, not prior
  findings or verdicts.
- Store review-fix loop recovery in a local state ledger under
  `{install_dir}/state/{slug}/`. The ledger is for the main agent only and is
  not passed to later reviewers.
- Keep the loop procedural in engine contracts. Do not add a new schema field
  or persistent state machine unless implementation discovers a deterministic
  need during build.

## Architecture

The change is an engine-contract update. The agent runtime interprets the
command forms and choice-card labels; the Rust CLI does not need a new subcommand
for this spec unless build discovers that current lint/conformance tooling needs
small supporting checks.

Affected contract layers:

- Router: recognizes the expanded `{slug} review fix [N]` intent and keeps
  review as a non-phase command.
- Review command: owns the user-facing grammar, result-only behavior, fix-mode
  loop, invalid forms, and presentation.
- Risk reference: owns shared review-loop boundaries, fresh-independent cycle
  rules, and automatic-fix stop conditions.
- Plan/build/open/update commands: place review/fix choice cards and state how
  review fixes interact with each lifecycle context.
- Reviewer contracts: state what focus input can be supplied in a later cycle
  and what prior review context must remain hidden.
- Local state ledger: records loop progress so a new session can recover
  requested budget, completed fix rounds, touched files, verification evidence,
  and stop reason without relying on conversation memory.
- Conformance tests: pin all contract text that protects the behavior.

## Data Model / Interfaces

User-facing command forms:

- `{slug} review`
- `{slug} review fix`
- `{slug} review fix 1`
- `{slug} review fix 2`
- `{slug} review fix 3`

Rejected command forms:

- `{slug} review 2`
- `{slug} review fix 0`
- `{slug} review fix 4` and higher

Internal working terms:

- `result-only review`: one read-only review pass.
- `fix round`: one reviewer pass followed by at most one bounded main-agent fix
  pass.
- `fresh independent review`: a reviewer pass that judges the current artifact
  or current full diff without seeing previous review findings or verdicts.
- `review-fix ledger`: a local, gitignored state file for main-agent recovery,
  not reviewer input. Suggested path:
  `{install_dir}/state/{slug}/review-fix.json`.

Suggested ledger fields:

- `requested_fix_rounds`
- `completed_fix_rounds`
- `phase`
- `reviewer_profile`
- `touched_files`
- `verification`
- `stop_reason`
- `updated_at`

Choice-card labels should be conversation-language labels, with stable
compatibility keywords in parentheses where useful. For Japanese conversation,
the likely labels are:

- `レビュー結果を見る` -> `{slug} review`
- `レビューして修正する` -> `{slug} review fix`
- `重点レビューして修正する` -> `{slug} review fix 2` or
  `{slug} review fix 3` when the phase/risk calls for a stronger option.

## Error Handling

- Invalid numeric budget: stop before any review runs and explain that fix
  rounds are 1, 2, or 3.
- Ambiguous `{slug} review 2`: stop before any review runs and explain that
  result-only review has no numeric budget; automatic fixing uses
  `{slug} review fix 2`.
- Non-fixable reviewer finding: stop with the finding and the required human or
  planning decision.
- Verification failure after a fix: stop with the failed command/evidence and do
  not continue to another review cycle until the failure is resolved.
- Repeated finding after a prior fix: stop rather than spending remaining budget
  on the same unresolved issue.
- Missing or unreadable review-fix ledger during resume: do not invent prior
  state from memory. Report that the loop cannot be resumed safely and ask the
  user to restart review or provide explicit direction.

## Test Strategy

- Add conformance tests around engine text rather than introducing runtime
  parser tests, because the command grammar lives in the agent router/command
  contract.
- Preserve existing tests that pin reviewer profile split, no write-capable
  workers, ad-hoc review report-only behavior, plan pre-approval review, update
  hold/finalize, and build/open review cadence.
- Run `cargo test --manifest-path cli/Cargo.toml` after adding conformance
  tests.
- Run the full configured `cli` default verification after dogfood sync.

## Review Results

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass
Reviewed through: b229100
