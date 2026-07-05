# Concepts

MochiFlow is a workflow engine for AI coding agents. It keeps the agent in a
repeatable lifecycle and stores the useful knowledge that should survive beyond
one chat.

## Workflow

The development stages are:

- `discuss` — clarify intent, scope, edge cases, and how the feature fits the
  existing codebase.
- `plan` — write the design document and wait for your approval before coding.
- `build` — implement the approved plan, update tests, and run verification.
- `open` — run acceptance, record durable decisions and pitfalls, settle the
  mechanical `accept` close-out, and open the PR after you approve its content.
- `update` — apply PR feedback through the build loop, re-verify, and refresh the
  PR.
- `close` — after the PR merges, do local cleanup (no writes to the base branch).

You can start a stage with an explicit AI-tool message such as
`mochiflow-discuss`, or with natural language when the intent is clear. These
stage triggers are messages for your AI coding tool, not terminal commands.

## Project knowledge

MochiFlow separates current state from durable reasoning:

- `constitution` — project rules written by you and always loaded by the agent.
- `context` — a current-state project map filled from code during onboarding and
  refreshed by the AI agent when it becomes stale.
- `specs` — per-change working artifacts under `.mochiflow/specs/`.
- `adr` — decisions and pitfalls recorded when the PR is opened (the `open`
  close-out) so later work can reuse the reasoning.

Code remains the source of truth for current behavior. Prose helps the agent
orient itself, but it does not override the codebase.

Specs can be persisted in two modes. In tracked mode, `.mochiflow/specs/` and
ADR records are committed, so the accepted close-out commit and `Spec:` trailer
are part of the PR. In local mode, `.mochiflow/` or the spec path is gitignored;
MochiFlow keeps acceptance evidence local, skips spec/ADR staging, and puts the
verification evidence, review result, and durable decision summary into the PR
body instead. Tracked mode is recommended for team auditability; local mode is
for repositories that intentionally keep workflow artifacts out of git.

## Approval gates

MochiFlow keeps two human decision points:

- approve the design before implementation starts;
- approve the PR content before the PR is opened.

The lifecycle also distinguishes **asserted** spec states stored in `spec.yaml`
(`draft → approved → accepted`) from **derived** delivery facts (In Review,
Done) observed from git/provider and never written back. `mochiflow status`
renders these as a live board; `INDEX.md` is a generated, gitignored cache of the
same view.

Riskier changes can require stricter review cadence, but the normal user flow
remains the same.

## Review

MochiFlow review is a quality assist, not a third approval gate. Plain review is
result-only: `{slug} review` or `mochiflow-review` runs one read-only reviewer
pass and reports findings without editing files, changing status, committing,
pushing, or creating PR metadata.

For bounded automatic fixing, use `{slug} review fix`, `{slug} review fix 2`,
or `{slug} review fix 3`. The number is the maximum number of fix rounds.
Reviewers remain read-only; the main agent applies only in-scope fixes, verifies
the result in the current lifecycle context, and stops after the requested
budget. Later review cycles are fresh independent reviews of the current
artifacts or diff, not a review of prior reviewer output.
