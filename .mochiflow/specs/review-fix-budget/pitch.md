# Add review budget loops with optional automatic fixes

## Problem

`mochiflow-review` currently means one read-only review pass: it reports
findings and never edits artifacts or code. That is useful when the user wants
to hand the review result to another agent or human, but it does not cover the
equally common request to let the main agent apply straightforward in-scope
review findings automatically.

The desired behavior is a bounded review/fix loop, not an unbounded "keep
reviewing until clean" loop. Users should be able to ask for one result-only
review, or for one to three automatic fix rounds, without learning a second verb
or a complicated set of command flags.

## Appetite

Medium. This is worth changing because review sits on the main plan/build
handoff path, but the command surface must stay small. Prefer one command shape,
simple choice-card labels, and procedural changes in the engine contracts over a
large new state machine.

## Solution

Keep `review` as the only user-facing verb. Add an optional `fix` modifier and a
small numeric budget:

- `{slug} review` runs one read-only review and reports findings only.
- `{slug} review fix` runs one review and applies one automatic in-scope fix
  round.
- `{slug} review fix 2` runs up to two independent review/fix rounds and ends
  after the second fix round when findings remain fixable.
- `{slug} review fix 3` runs up to three independent review/fix rounds and ends
  after the third fix round when findings remain fixable.

The number after `fix` is the maximum number of fix rounds. It is not the number
of reviewer calls, and it does not require the loop to end with a clean
post-fix review. Do not add user-facing caveat wording after completion about a
final post-fix review being absent; the command contract itself owns that
meaning.

Choice cards should map to the same primitive:

- "Review results" maps to `{slug} review`.
- "Review and fix" maps to `{slug} review fix`.
- For higher-impact work, a stronger label may map to `{slug} review fix 2` or
  `{slug} review fix 3`.

The reviewer profile is selected from the current work state, as today:

- before implementation, use `plan-auditor`;
- after implementation exists, use `change-reviewer`.

Each review cycle must be fresh and independent. The next reviewer receives the
current artifacts or code and may receive the diff since the previous cycle as
focus input, but must not receive previous findings, previous verdicts, prior
reviewer summaries, or conversation history. The main agent keeps prior findings
only for applying fixes and deciding whether to stop.

Automatic fixes are bounded by the existing shared judgment: no task-structure
change, no new AC, and no new design decision. If a finding needs scope change,
spec split, human judgment, or a return to planning, the loop stops and reports
the decision point instead of improvising.

Reviewers remain read-only. The main agent applies fixes inline, runs the
verification cadence appropriate to the current phase, and records review-loop
evidence in the same place as existing review results unless planning finds a
better artifact location.

## Rabbit Holes

- Adding a second public verb such as `revise` or `refine`; the user explicitly
  wants to keep the surface centered on `review`.
- Treating the numeric budget as reviewer-call count. That makes `review fix 2`
  ambiguous and tends to force a final post-fix review the user does not want.
- Passing previous findings into the next reviewer. That turns a fresh review
  into a patch-confirmation pass and anchors the reviewer on the prior report.
- Letting the reviewer apply fixes. This would reverse the active decision that
  reviewer roles are read-only and implementation stays on the main agent.

## No-gos

- Do not introduce severity/gate flags such as `--gate medium`.
- Do not add an unbounded fix-until-pass loop.
- Do not make natural-language intensity words the canonical control surface.
- Do not change the two delivery approval gates.
- Do not make Low / nit / optional findings part of the automatic loop stop
  condition unless planning explicitly defines a narrower local rule.
- Do not delegate write-capable implementation work to a reviewer or worker.

## Alternatives Considered

- `review` plus `revise` / `refine`: rejected because the extra verb is
  conceptually clean but increases the command vocabulary.
- `{slug} review 2`: rejected because it cannot distinguish "show me two
  independent read-only opinions" from "review and fix with a budget".
- Always ending `review fix N` with a review pass: rejected because the desired
  workflow is to end after the requested fix rounds.
- Natural-language labels such as "strict review" or "careful review" as the
  primary control: rejected because they are easy to misinterpret. Choice cards
  can still use plain labels and map them to the fixed command forms.

## Open Questions

None - ready for plan.
