# Workflow Reference

The shared, cross-cutting rules for the four mochiflow verbs. Per-verb
procedure lives in `commands/{discuss,plan,build,ship}.md`; this file holds the
facts those commands share, each defined once.

## Verbs and state

```
discuss ──▶ plan ──▶ build ──▶ ship
```

| state | meaning | set by |
| --- | --- | --- |
| `draft` | spec authored, not yet approved | plan |
| `approved` | human approved implementation start | delivery approval gate |
| `done` | acceptance conditions met and accepted | ship |

State lives in `spec.yaml` `status`. There is no separate per-document status.

## Spec lane vs Patch lane

The four verbs are the spec lane: they create, approve, implement, ship, fold,
and archive durable spec artifacts.

`patch` is a non-phase lane for concrete small edits that do not need durable
spec artifacts. It has no `spec.yaml`, no `draft|approved|done` state, no AC
Matrix, no ship, no archive, and no living-spec fold. If a patch needs a new
product/design decision, public contract change, migration, security or
data-loss judgment, multi-surface coordination, or human QA record, stop and
route to `plan`.

## Delivery approval gates (exactly two)

1. **approve-to-build** — human approves the spec for implementation (`draft → approved`).
2. **approve-PR** — human approves PR title/description before `mochiflow pr` runs.

Approval words: `OK` / `承認` / `LGTM` / `approved`. They apply only to these two
delivery approval gates. The AI never sets `approved` without delivery approval
gate 1 and never creates a PR before delivery approval gate 2. Setup, context
refresh, and QA evidence may require human confirmations, but those confirmations
are not delivery approval gates and do not change spec lifecycle state except
where explicitly defined. `done` is not a gate: it is an acceptance state that
`ship` sets mechanically when the acceptance conditions below hold.

## Depth scaling

A change is always one folder under `{specs_dir}/{slug}/`. Documents grow only
as far as the change needs:

| Depth | Use case | Documents | Requirements detail | Tasks |
| --- | --- | --- | --- | --- |
| Patch | Small concrete fix | none | none | none |
| Micro spec | Trivial but worth recording | `spec.md` | problem / change / AC / verify | none or minimal |
| Standard spec | Normal feature/fix | `spec.md` + `tasks.md` | AC table + QA examples | checklist |
| Design spec | Design decision or multiple areas | `spec.md` + `design.md` + `tasks.md` | NFR / contract / examples | dependency checklist |
| Critical spec | migration / security / data loss / external contract | full | traceability / rollback / observability / reviewer | per-task verification checklist |

Let depth increase with risk, integration, surfaces, ambiguity, and external
contracts. Do not add prose for its own sake; detail should be checkable,
traceable, and executable.

`design.md` necessity is governed by `risk.md ## design.md required condition`.
`tasks.md` is required for standard-or-larger multi-step work and optional for
micro specs.

## AC, DoD, Tasks, and Matrix

| concept | responsibility | source |
| --- | --- | --- |
| Acceptance Criteria | feature-specific success conditions | `spec.md` |
| Definition of Done | common quality bar for all specs | workflow / risk / git references |
| Tasks | executable work plan to satisfy AC and DoD | `tasks.md` |
| AC Matrix | traceability from requirement to implementation to verification to evidence | `spec.md` |

Each AC must be verifiable. Each task must be executable. Each matrix row must
be auditable.

## AC Matrix

The AC Matrix is created during plan in `spec.md` under
`## Verification Plan / AC Matrix`.

```md
## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | {surface} | automated | `cargo test ...` | `path/File.ext` | PASS | test output |  |
```

Canonical result values are:

- `UNVERIFIED` — planned but not verified yet; allowed during plan and build, not at ship.
- `PASS` — automated or manual verification passed.
- `PENDING_HUMAN` — human QA is required but not completed yet; allowed during build, not at ship.
- `HUMAN_CONFIRMED` — human QA completed and confirmed.
- `N/A: <reason>` — not applicable with explicit reason; reason is required.
- `FAIL` — verification failed; not allowed at ship.

Do not use localized enum values as canonical Matrix values. Explain them in
prose when useful, but keep the table value as the English token.

`done` is an acceptance state, not a human approval. `ship` sets `status: done`
mechanically once all of these conditions hold:

1. the AC Matrix is present and complete;
2. every spec AC appears as one or more matrix rows;
3. no row has `UNVERIFIED`, `PENDING_HUMAN`, or `FAIL`;
4. every `N/A` result is written as `N/A: <reason>`;
5. required evidence is recorded;
6. required tasks in `tasks.md` are complete or explicitly not applicable;
7. when `risk ≥ elevated`, the reviewer verdict is recorded per `risk.md`.

`build` never sets `done`.

## Verification profiles

Verification commands are not hardcoded in the engine. Each surface declares its
commands in `config.toml` under `[surfaces.<surface>.verify]` with named profiles
(`default`, and optionally `quick` / `targeted` / others). Resolve them via:

- `mochiflow config show` — inspect the resolved commands for every surface.
- The verb runs the command for the spec's surface and the appropriate profile,
  substituting `{target}` when a profile expects a target.

A surface whose only profile is a `TODO:` placeholder is not yet runnable; define
its command before building that surface.

## Patch verification

For patch, use the narrowest reliable verification that proves the concrete
change:

1. Run the clearly related test command when the target is obvious from the code
   or request.
2. Otherwise run the surface's `quick` profile when it exists.
3. Otherwise run the surface's `default` profile.

If no runnable verification command exists, report that explicitly and do not
auto-commit.

## Acceptance adapters (ship)

Build `{specs_dir}/{slug}/qa-instructions.md` from `spec.md` QA scenarios
(reference, do not copy), and pick the adapter by `Scope` / kind:

| Scope / kind | adapter | main checks |
| --- | --- | --- |
| automated test | command verification | build/lint/test command + result |
| `api` | API QA | status / schema / error / auth / health |
| `web` | Browser QA | route / DOM / validation / network / responsive |
| configured app/device surface | app/device QA | simulator, device, accessibility, or visual check as applicable |
| `human` | Human confirmation | physical device, judgement, visual, external service |
| `cross-surface` | contract / workflow QA | contract or workflow across surfaces |

Human/visual AC are requested once, in ship, alongside `qa-instructions.md` —
not pre-requested during build.

## Backlog seeds

`{specs_dir}/_backlog/{slug}.md` is a single-file seed inbox: raw input for
`discuss`, not a spec. Use `templates/backlog/seed.md`. Frontmatter:
`slug,title,maturity,source,created,updated` (+ optional
`module,surface,type_hint,source_spec,source_phase`). Body: `## Signal`,
`## Why It Matters`, `## Evidence`, `## Open Questions`.

Lifecycle: create seed → `discuss` reads it as input (seed kept) → `plan`
creates `{specs_dir}/{slug}/` and deletes the seed, recording origin in
`spec.md`. Interrupted discuss keeps the seed. Do not put AC, QA, design, tasks,
or final classification in a seed.

Legacy `_backlog/{slug}/` spec-format directories are deprecated and no longer
rendered by tooling; they remain on disk read-only.
