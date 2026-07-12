# Agent Context API

## Problem

AI coding agents cannot currently obtain one authoritative, side-effect-free
view of MochiFlow repository state. Repository-wide delivery state is rendered
for humans by `status`; build eligibility and consistency failures are exposed
separately by `ready` and `lint`; and the existing JSON board is written to
`state/index.json` by `index`. An agent must therefore combine multiple command
outputs and Markdown contracts, repeat state interpretation in each adapter,
and distinguish missing facts from provider or Git observation failures on its
own.

This fragmentation creates two long-term costs. First, routing and lifecycle
eligibility can drift between Rust, engine procedures, and adapters. Second,
the current delivery probes collapse some unavailable observations to false and
run Git or provider commands per spec, so a larger repository receives a slower
and less trustworthy context view.

## Appetite

Treat this as one design-level CLI feature cycle. It is worth adding a frozen
public JSON contract and consolidating the internal state snapshot used by
`inspect`, `status`, `index`, and `ready`. It is not worth building a server,
prompt compiler, persistent cache, or general automation platform.

## Solution

Add a read-only `mochiflow inspect` command with two intentionally different
depths:

- `mochiflow inspect --json` returns a lightweight repository snapshot. It
  reports branch/base information, deterministic active-spec resolution,
  summaries for every spec, coarse next candidates, parse failures as visible
  entries, and ambiguity without running full lint for every spec.
- `mochiflow inspect <slug> --json` returns detailed context for one spec. It
  reports metadata, persistence mode, lint/readiness health, tri-state Git and
  provider signals, lifecycle eligibility with stable blocker codes, related
  repository-relative paths, and a suggested workflow only when repository
  state makes it unambiguous.

The CLI owns deterministic repository facts and eligibility for the six spec
lifecycle actions: `discuss`, `plan`, `build`, `open`, `update`, and `close`.
The engine router remains the sole owner of natural-language intent and route
selection, while command procedures remain the owners of execution steps.
Non-phase commands such as `review`, `onboard`, and `refresh-context` are not
part of v1 eligibility.

Keep workflow suggestions separate from human next actions. For example, an
approved incomplete spec may suggest `build`; an in-review spec without explicit
feedback suggests no workflow but may retain the existing `report_merge` human
next action. `update` can be eligible while remaining unsuggested because PR
feedback is an external intent signal. Repository scope binds an active spec
only through deterministic branch evidence; it never guesses from modification
time.

Make JSON a public versioned contract at
`contracts/agent-context.schema.json`. The top-level contract carries
`schema_version`, `scope`, `result`, freshness, warnings, and the repository or
spec payload. Machine-facing actions, blockers, warnings, and errors use stable
IDs distinct from localized display text. Human `inspect` output remains concise;
`--json` is the canonical API. JSON mode emits exactly one schema-valid document
on stdout for success, degraded, partial, and error results.

Use these result and exit semantics:

- complete observation: `result: ok`, exit 0;
- unavailable provider or other external observation: `result: degraded`, exit
  0, with affected facts represented as `unknown` rather than false;
- a repository snapshot containing malformed specs: `result: partial`, exit 1,
  while preserving valid entries and explicit error entries;
- an invalid config, missing requested slug, or malformed requested spec:
  `result: error`, exit 1;
- argument misuse: the existing Clap usage exit behavior.

The default command never writes files, regenerates indexes, or updates Git
refs. It may read local Git state and query a configured provider. A separate
`--fetch` flag may explicitly refresh origin before observation. Paths in the
contract are normalized repository-relative paths; file contents, diffs,
absolute machine paths, configured command bodies, and raw Git/provider stderr
are excluded. Diagnostics are sanitized into stable codes and safe messages.

Introduce a shared read-only repository/spec snapshot model rather than a
parallel implementation. `status`, `index`, and `ready` keep their public
commands and output contracts but consume the shared state and eligibility
logic. The existing `state/index.json` contract remains compatible. Repository
collection may scan O(N) spec files, but Git and provider subprocess/network
probes must remain constant-bounded rather than scaling per spec.

The frozen schema and representative fixtures cover repository, spec, degraded,
partial, and error documents. Behavioral coverage includes ambiguous branch
resolution, malformed specs, provider unavailability, tracked and local-only
persistence, all lifecycle eligibility results, existing output compatibility,
batch probe behavior, and proof that ordinary inspection changes no files.

## Rabbit Holes

- Turning eligibility into a second natural-language router instead of keeping
  intent selection in `engine/router.md`.
- Returning every Markdown document or full diagnostic output and creating an
  unbounded context dump.
- Implementing `inspect` as a new state engine beside `status`, `index`, and
  `ready` instead of sharing one snapshot model.
- Treating provider unavailability as a negative fact or failing the whole
  snapshot when useful local facts remain available.
- Preserving the current per-spec Git/provider process pattern behind a new API
  surface.
- Adding pagination before repository scale demonstrates a need for it.

## No-gos

- No HTTP server, daemon, socket, MCP server, or background process.
- No prompt assembly, automatic Markdown loading, or file-content embedding.
- No persistent context cache or generated `agent-context.json` file.
- No natural-language intent classification in Rust.
- No eligibility model for non-phase commands in v1.
- No Verification Receipt, recovery automation, Provider Driver v2, or
  adapter-specific auto-invocation in this change.
- No new lifecycle status, spec directory layout, or persisted delivery fact.
- No removal or incompatible output change for `status`, `index`, `ready`,
  `lint`, or `state/index.json`.

## Alternatives Considered

- Add only `status --json`: rejected because a board contract cannot express
  detailed blockers, lifecycle eligibility, malformed-target errors, and
  per-spec health without overloading the human status surface.
- Reuse `state/index.json` directly: rejected because producing it writes state,
  it lacks the required tri-state and error contracts, and it should remain a
  compatible derived board artifact.
- Return raw facts only: rejected because every adapter would have to recreate
  deterministic lifecycle eligibility and would drift over time.
- Let the CLI select workflows from natural language: rejected because it would
  compete with the router as a second route authority.
- Build only a per-spec endpoint: rejected because agents need a repository
  snapshot to resolve an unknown or ambiguous active spec before requesting
  details.
- Replace the existing commands immediately: rejected because it adds migration
  risk without increasing the first version's user value.

## Open Questions

None — ready for plan.
