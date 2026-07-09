# Redesign engine context loading and contract layout

## Problem

MochiFlow's engine uses progressive loading in name, but the effective context
graph is still broad. The standing layer loads the adapter entrypoint,
constitution, all foundational context files, and a router that now contains
substantial workflow, review, delivery, and presentation detail. After a route
is selected, commands load large cross-cutting files such as `workflow.md`,
`risk.md`, and `git.md` in full even when they need only one responsibility from
each file. Reviewer profiles are especially expensive because their bootstrap
loads five full references before reading the spec, diff, or repository.

The same invariants are also repeated across the router, adapters, commands,
references, reviewer profiles, presentation guidance, and stop conditions.
That repetition consumes context, makes the active rule harder to identify, and
has already allowed wording and canonical-token descriptions to drift. The
previous `engine-context-progressive-loading` change clarified standing versus
on-demand categories, but intentionally retained the monolithic file-level
reference layout and standing foundational context. Those boundaries no longer
produce the desired result.

## Appetite

Treat this as one comprehensive elevated-risk engine refactor. Redesign the
internal contract layout from first principles rather than preserving old
engine file paths or limiting the work to prose edits. The work may reorganize
router, command, reference, reviewer, template, and adapter source files, but it
must preserve MochiFlow's public CLI, persisted schemas, lifecycle semantics,
approval gates, review behavior, git safety, and generated adapter ownership.

The target is the cleanest maintainable loading architecture, not the smallest
possible diff. Implementation remains a later phase; this discussion creates
only the draft agreement.

## Solution

Rebuild the engine guidance as a dependency-minimal contract graph:

- Keep one authoritative standing router, but reduce it to route recognition,
  activation strength, active-spec resolution, state/intent conflict handling,
  and selection of the next command. Procedure summaries, review mechanics,
  delivery details, transition narration, and generic completion wording move
  out of the standing router.
- Keep user-authored constitution files standing. Do not eagerly load
  `context/product.md`, `context/structure.md`, and `context/tech.md` merely to
  route a conversation. Load the relevant foundational context after a
  lifecycle command is activated or when ordinary repository-specific work
  actually needs current-state orientation.
- Split monolithic references by responsibility so file-level loading remains
  portable while commands read only the policies they use. Separate lifecycle
  state and gates, spec depth, verification and AC Matrix rules, QA policy,
  review policy, branch/commit mechanics, delivery/PR mechanics, and knowledge
  fold/context-refresh mechanics.
- Make command loading staged. A command first loads its core procedure and
  required policy references, then loads only the template or conditional
  policy selected by the resolved depth, risk, persistence mode, or delivery
  path. Mutually exclusive templates must not be eager peers.
- Replace reviewer bootstrap with a compact shared reviewer core plus small
  plan-auditor and change-reviewer profiles. Reviewer resources contain only
  the rules required to review; unrelated branch, PR, choice-card, lifecycle,
  and authoring material is not preloaded. Reviewers continue to discover code,
  specs, diffs, verification evidence, and relevant ADR records on demand.
- Define each invariant once in its owning reference and repeat it only as a
  short guardrail at the mutation boundary where violating it would be unsafe.
  Remove third and later copies from router summaries, presentation sections,
  compatibility prose, and unrelated stop-condition lists.
- Reduce command documents to executable procedures plus phase-specific stop
  conditions. Consolidate generic presentation, language, reviewer output, and
  completion-card rules into small shared contracts rather than restating them
  per command.
- Remove obsolete compatibility-only engine documents and old internal path
  aliases. `mochiflow upgrade` replaces the installed engine and regenerates
  adapters; generated targets that cannot be updated automatically continue to
  use the existing adapter-merge-required path.
- Update generated adapter templates and Kiro reviewer resources to point at the
  redesigned graph. Adapter targets, managed-block ownership, and overwrite
  protection remain unchanged.
- Replace brittle long-prose conformance assertions only where necessary to
  permit the new layout. Retain focused behavioral and structural guards for
  routing, reference ownership, adapter generation, and reviewer read-only
  boundaries without adding token or size budgets.

## Rabbit Holes

- Optimizing for line count while leaving the same files in every reference
  set. The loading graph, not cosmetic compression, is the primary target.
- Removing safety rules merely because they repeat. Keep the canonical owner
  and one local mutation-boundary reminder where omission could cause data,
  git, approval, or delivery mistakes.
- Introducing section-anchor reads that only some supported agent surfaces can
  resolve reliably. Prefer smaller responsibility-owned files with explicit
  file-level references.
- Turning the engine into a runtime state machine or adding a command that
  assembles prompts dynamically. Markdown contracts and existing adapter
  mechanisms remain the execution model unless planning proves they cannot
  express staged loading correctly.
- Mixing unrelated workflow feature changes into the refactor. Correctness gaps
  such as stronger reviewer-evidence validation require their own product scope
  unless a minimal change is strictly necessary for the new contract layout.

## No-gos

- Do not add context-budget, token-count, file-size, or word-count tests.
- Do not add a context measurement, audit, bundle, or prompt-compilation CLI.
- Do not retain old internal engine paths through redirect files, compatibility
  stubs, duplicate aliases, or legacy reviewer wrappers.
- Do not preserve the previous decision that all foundational context files are
  standing inputs.
- Do not preserve monolithic references solely to avoid reorganizing paths.
- Do not change public CLI command behavior, config/spec/ADR/PR schemas,
  lifecycle states, the two delivery approval gates, reviewer read-only
  ownership, spec persistence modes, or PR delivery semantics.
- Do not implement source changes during discuss or plan.

## Alternatives Considered

- **Only shorten `router.md`.** Rejected because command and reviewer bootstrap
  would still load the same large reference sets.
- **Keep monolithic references and introduce section anchors.** Rejected because
  supported adapters and agent runtimes do not provide one reliable portable
  section-read contract, while responsibility-sized files work everywhere.
- **Generate per-command prompt bundles through the CLI.** Rejected because it
  adds runtime machinery, generated prompt artifacts, and a second composition
  system when explicit staged file references can express the design.
- **Keep foundational context standing but compress its prose.** Rejected because
  routing does not require repository orientation; loading it later removes the
  cost without weakening code-grounded work.
- **Retain compatibility stubs for old engine paths.** Rejected because
  `mochiflow upgrade` performs full engine replacement and adapter regeneration.
  Stubs would preserve duplication indefinitely for an internal contract.
- **Split the work into several independent specs.** Rejected because router,
  reference ownership, command frontmatter, reviewer resources, adapters, and
  conformance guards form one loading graph and must land coherently.

## Open Questions

None — ready for plan.
