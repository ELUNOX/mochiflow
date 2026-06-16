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
| `approved` | human approved implementation start | human gate |
| `done` | acceptance conditions met and accepted | ship |

State lives in `spec.yaml` `status`. There is no separate per-document status.

## Spec lane vs Patch lane

The four verbs are the spec lane: they create, approve, implement, ship, fold,
and archive durable spec artifacts.

`patch` is a non-phase lane for concrete small edits that do not need durable
spec artifacts. It has no `spec.yaml`, no `draft|approved|done` state, no AC
Verification Matrix, no ship, no archive, and no living-spec fold. If a patch
needs a new product/design decision, public contract change, migration, security
or data-loss judgment, multi-surface coordination, or human QA record, stop and
route to `plan`.

## Human gates (exactly two)

1. **approve-to-build** — human approves the spec for implementation (`draft → approved`).
2. **approve-PR** — human approves PR title/description before `[git].pr_command` runs.

Approval words: `OK` / `承認` / `LGTM` / `approved`. They apply **only** to these
two gates. The AI never sets `approved` without the gate-1 signal and never
creates a PR before gate 2. `done` is **not** a gate: it is an acceptance state
that `ship` sets mechanically when the acceptance conditions below hold — no
approval word is involved.

## Depth scaling

A change is always one folder under `{specs_dir}/{slug}/`. Documents grow
only as far as the change needs:

| depth | discuss | spec.md | design.md | tasks.md |
| --- | --- | --- | --- | --- |
| trivial | skip | required | — | — |
| single/near module, existing pattern | light | required | — | when multi-step |
| design decision / multi-surface / contract / migration | full | required | required | required |

`design.md` necessity is governed by `risk.md ## design.md required condition`.
Do not pick a "lane" up front; let the documents emerge.

## AC Verification Matrix

Written at the end of `spec.md` after all tasks complete (or `tasks.md` end when
present).

```md
## AC Verification Matrix

| AC | Scope | 実装箇所 | テスト | QA | 結果 | 備考 |
| --- | --- | --- | --- | --- | --- | --- |
| AC-01 | ios | `path/File.swift` | `Test.test_case` | QA-01 | PASS |  |
```

Result is `PASS` / `人間確認済み` / `対象外（理由）` / `FAIL`.

`done` is an **acceptance state**, not a human approval. `ship` sets
`status: done` mechanically (no approval word) once **all** of these acceptance
conditions hold; `build` never sets `done`:

1. the AC Verification Matrix is present and complete — every spec AC appears as a row;
2. no row is `FAIL` and none is still pending human verification;
3. when `risk ≥ elevated`, the reviewer verdict is recorded (condition owned by
   `risk.md ## Consequences`; referenced here, not redefined).

`lint` enforces the matrix presence and AC↔task coverage; it also warns on
`[NEEDS-CLARIFICATION]` and AC lines missing an EARS keyword (resolve before
`approved`).

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

Build `qa-instructions.md` from `spec.md` QA scenarios (reference, do not copy),
and pick the adapter by `Scope` / kind:

| Scope / kind | adapter | main checks |
| --- | --- | --- |
| automated test | command verification | build/lint/test command + result |
| `api` | API QA | status / schema / error / auth / health |
| `web` | Browser QA | route / DOM / validation / network / responsive |
| `ios` | iOS QA | simulator / device / accessibility / visual check |
| `human` | Human confirmation | physical device, judgement, visual, external service |
| `cross-surface` | contract / workflow QA | contract or workflow across surfaces |

Human/visual AC are requested once, in ship, alongside `qa-instructions.md` — not
pre-requested during build.

## Backlog seeds

`{specs_dir}/_backlog/{slug}.md` is a single-file seed inbox: raw input for
`discuss`, not a spec. Use `templates/backlog/seed.md`. Frontmatter:
`slug,title,maturity,source,created,updated` (+ optional `module,surface,type_hint,source_spec,source_phase`).
Body: `## Signal`, `## Why It Matters`, `## Evidence`, `## Open Questions`.

Lifecycle: create seed → `discuss` reads it as input (seed kept) → `plan` creates
`{specs_dir}/{slug}/` and deletes the seed (`rm {specs_dir}/_backlog/{slug}.md`),
recording origin in `spec.md`. Interrupted discuss keeps the seed. Do not put AC,
QA, design, tasks, or final classification in a seed.

Legacy `_backlog/{slug}/` spec-format directories are deprecated and no longer
rendered by tooling; they remain on disk read-only.
