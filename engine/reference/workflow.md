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
| `draft` | pitch agreed, not yet approved for implementation | discuss |
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
`ship` sets directly (editing `spec.yaml` `status: done`, then re-running `lint`
to confirm; there is no CLI transition command) when the acceptance conditions
below hold.

When an approval gate is presented as a numbered choice card, selecting the
visible approval action by label or by its displayed number is the gate input.
For example, a plan card may display "confirm the plan" as the action that sets
`status: approved`; choosing that action dispatches approve-to-build. The old
approval words remain compatibility inputs, not the preferred user-facing label.

The no-PR fast path exists only after explicit human opt-in. It skips
**approve-PR** because no PR is created, but it still runs `ship`; `ship` still
sets `done` from acceptance conditions and creates the same close-out commit.

## Choice cards

Phase-completion choice cards present user-facing actions, each with a stable
action label and optional compatibility keywords. A displayed number is an
ephemeral alias for that action in the most recent unambiguous choice card only.
It is not a durable command, and it must not be interpreted without the active
card context.

Choice selection is the dispatch primitive: choosing a visible action by label,
compatibility keyword, or displayed number invokes that action. If a bare number
is stale, out of range, or contextless, ask the user to choose again using the
current action labels.

## Depth scaling

A change is always one folder under `{specs_dir}/{slug}/`. Documents grow only
as far as the change needs:

| Depth | Use case | Documents | Requirements detail | Tasks |
| --- | --- | --- | --- | --- |
| Patch | Small concrete fix | none | none | none |
| Micro spec | Trivial but worth recording | `pitch.md` + `spec.md` | problem / change / AC / verify | none or minimal |
| Standard spec | Normal feature/fix | `pitch.md` + `spec.md` + `tasks.md` | AC table + QA examples | checklist |
| Design spec | Design decision or multiple areas | `pitch.md` + `spec.md` + `design.md` + `tasks.md` | NFR / contract / examples | dependency checklist |
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

Canonical result values are exact tokens:

- `PASS` — done-eligible automated or AI-observed verification passed.
- `CONFIRMED` — done-eligible human/visual QA was confirmed.
- `N/A: <reason>` — done-eligible not-applicable result with a concrete reason.
- `FAIL` — failing result; not done-eligible.
- `PENDING_HUMAN` — provisional build-time result for human/visual QA that has
  not been performed yet; not done-eligible.
- `UNVERIFIED` — provisional build-time result for an automated/AI-observed AC
  row not yet verified; not done-eligible. Resolve to `PASS` / `FAIL` /
  `N/A: <reason>` before `done`.

The done-eligible tokens are exactly `PASS`, `CONFIRMED`, and `N/A: <reason>`.
`PENDING_HUMAN` and `UNVERIFIED` are provisional build-time placeholders only.
Deprecated aliases `人間確認済み` (equivalent to `CONFIRMED`) and
`対象外（<reason>）` (equivalent to `N/A: <reason>`) are permanently accepted
by lint for backward compatibility with archived specs.

`done` is an acceptance state, not a human approval. There is no CLI transition
command: `ship` edits `spec.yaml` `status: done` (and `updated`) directly once
all of these conditions hold, then re-runs `lint` to confirm — no approval word
is involved:

1. the AC Verification Matrix is present and complete — every spec AC appears as a row;
2. every row has a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`);
3. when `risk ≥ elevated`, the reviewer verdict is recorded (condition owned by
   `risk.md ## Consequences`; referenced here, not redefined).

For `status: done`, `lint` enforces matrix presence, AC↔task coverage, and final
result tokens; `PENDING_HUMAN`, `UNVERIFIED`, `FAIL`, empty cells, and
unknown/free-text results fail. It also warns on `[NEEDS-CLARIFICATION]` and AC
lines missing an EARS keyword (resolve before `approved`).

`build` never sets `done`.

## Verification profiles

Verification commands are not hardcoded in the engine. Each surface declares its
commands in `config.toml` under `[surfaces.<surface>.verify]` with named
profiles.

`default` is the canonical verification profile for spec-lane build completion:
it should be the reliable local command whose success is sufficient to say the
surface is ready for PR / merge, except for checks explicitly documented as
human-operated or CI-only. Optional profiles such as `quick` / `targeted` are for
faster intermediate feedback; they do not replace `default` for build completion.

Resolve commands via:

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

Ship identifies human-operated and visual QA items from `spec.md` QA Scenarios
(the `Type` column) and picks the adapter by `Scope` / kind:

| Scope / kind | adapter | main checks |
| --- | --- | --- |
| automated test | command verification | build/lint/test command + result |
| `api` | API QA | status / schema / error / auth / health |
| `web` | Browser QA | route / DOM / validation / network / responsive |
| configured app/device surface | app/device QA | simulator, device, accessibility, or visual check as applicable |
| `human` | Human confirmation | physical device, judgement, visual, external service |
| `cross-surface` | contract / workflow QA | contract or workflow across surfaces |

Human/visual AC are requested once, in ship, via the QA round-trip protocol —
not during build. During build, mark those Matrix rows `PENDING_HUMAN` with the
needed QA scenario and evidence expectation.

## Backlog seeds

`{specs_dir}/_backlog/{slug}.md` is a single-file inbox for raw ideas only. It
is not a spec and is not a plan-ready handoff.

- Raw seed: `maturity: seed`, created from `templates/backlog/seed.md`, and used
  as raw input for `discuss`. Body: `## Signal`, `## Why It Matters`,
  `## Evidence`, `## Open Questions`.
Shared frontmatter: `slug,title,maturity,source,created,updated` (+ optional
`module,surface,type_hint,source_spec,source_phase`).

Lifecycle: create raw seed → `discuss` reads it as input → when agreement is
reached, `discuss` creates `{specs_dir}/{slug}/spec.yaml` (`status: draft`) and
`{specs_dir}/{slug}/pitch.md`, creates/switches to `{prefix}/{slug}`, deletes the
raw seed when present, runs pitch-only lint, and commits the promotion. `plan`
then reads `pitch.md` as its durable input. Interrupted discuss keeps the raw
seed file. Do not put AC, QA, design, tasks, or final classification in backlog
files.

Legacy `_backlog/{slug}/` spec-format directories are deprecated and no longer
rendered by tooling; they remain on disk read-only.
