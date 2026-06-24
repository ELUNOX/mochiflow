# Prevent build-phase spec mutation

## Problem

During the `commit-trailer-traceability` build, lint failed at close-out because
one acceptance criterion had no covering task. The practical fix was to add a
new task to `tasks.md` during build, which made the approved plan mutable at the
point where implementation should only execute it. That weakens the
approve-to-build gate: a build agent can reshape the checklist to satisfy lint
instead of stopping for plan correction.

The current code supports part of the desired model already. `lint` extracts AC
IDs from task-line bracket references and from `Covers AC:` lines using the
shared `AC_RE`, so comma-separated or otherwise compound references are
technically parseable. The gap is policy and coverage: build instructions do not
explicitly stop on structural `tasks.md` changes, authoring guidance does not
document multi-AC task references, and tests do not lock the compound-reference
behavior in place.

This spec branch intentionally includes the existing local commit
`3621592 docs: add phase completion backlog seed`; the later plan/build should
account for that commit staying in the branch history.

## Appetite

Small but careful. This is a workflow-boundary fix, not a feature expansion. It
is worth touching engine instructions, task authoring guidance, task templates,
and focused lint tests, but not worth redesigning the whole phase lifecycle.

## Solution

Make the build phase treat approved `tasks.md` structure as a contract. During
build, the agent may update task checkboxes and AC Matrix result/evidence fields,
but it must stop and route back to plan for re-approval if implementation needs
task additions, deletions, splits, renumbering, AC/NFR/chore reference changes,
or meaningful changes to `Depends on`, `Files`, `Done`, or `Stop`.

Document that one task may cover multiple related acceptance criteria with a
compound reference such as `[AC-07, AC-08]`. Keep lint's behavior aligned with
that documented shape: task coverage and unknown-AC checks should treat every
AC ID in a compound reference as an individual reference. Add targeted tests so
the behavior is explicit rather than incidental.

The likely implementation areas are:

- `engine/commands/build.md` and the vendored `.mochiflow/engine/commands/build.md`
  after dogfood sync.
- `engine/reference/authoring.md` and `engine/templates/spec/tasks.md`, with the
  vendored copies updated through `mochiflow upgrade --source engine`.
- `cli/crates/mochiflow-core/src/lint.rs` only if tests expose a parsing gap.
- `cli/crates/mochiflow-cli/tests/conformance.rs` for compound AC references and
  any changed lint expectation.
- Generated/frozen artifacts required by the project constitution after engine
  source edits.

## Rabbit Holes

- Do not build an agent-side diff parser that tries to classify every possible
  Markdown edit. The contract should be clear enough that the build procedure
  stops before making structural edits.
- Do not make every AC require a separate task. Related ACs can share one task
  when that task's implementation and verification naturally cover them.
- Do not solve the broader phase-completion guidance backlog here except where a
  clear build stop message is needed for this boundary.

## No-gos

- No implementation code should be written during discuss.
- No plan-phase corrections should be silently performed during build.
- No direct `git push` or provider PR command should be introduced.
- No hand edits to generated adapter outputs.

## Alternatives Considered

- Allow minor task splits during build. Rejected because the line between a
  minor split and plan mutation is not auditable enough for the approval gate.
- Require one task per AC. Rejected because it creates artificial checklist
  churn when one implementation step legitimately satisfies multiple related
  criteria.
- Rely only on human discipline without lint tests or template updates. Rejected
  because the old failure mode came from tooling pressure at the end of a build;
  the safer behavior needs to be documented and exercised.

## Open Questions

- None - ready for plan.
