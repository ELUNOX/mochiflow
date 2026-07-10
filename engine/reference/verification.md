# Verification Reference

The AC Verification Matrix contract, verification profiles, and human/visual QA
acceptance. Lifecycle states and approval gates live in
`reference/lifecycle.md`; reviewer cadence and verdict recording live in
`reference/review.md`; risk classification and QA attack coverage live in
`reference/risk.md`.

## AC, DoD, Tasks, and Matrix

| concept | responsibility | source |
| --- | --- | --- |
| Acceptance Criteria | feature-specific success conditions | `spec.md` |
| Definition of Done | common quality bar for all specs | lifecycle / verification / risk / git references |
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
  `N/A: <reason>` before `accepted`.

The done-eligible tokens are exactly `PASS`, `CONFIRMED`, and `N/A: <reason>`.
`PENDING_HUMAN` and `UNVERIFIED` are provisional build-time placeholders only.
Deprecated aliases `人間確認済み` (equivalent to `CONFIRMED`) and
`対象外（<reason>）` (equivalent to `N/A: <reason>`) are permanently accepted
by lint for backward compatibility with archived specs.

`accepted` is an acceptance state, not a human approval (`reference/lifecycle.md`).
`open` reaches it by running `mochiflow accept {slug}` once all of these
conditions hold; no approval word is involved:

1. the AC Verification Matrix is present and complete — every spec AC appears as a row;
2. every row has a done-eligible result token (`PASS`, `CONFIRMED`, or `N/A: <reason>`);
3. when `risk ≥ elevated`, the reviewer verdict is recorded (condition owned by
   `reference/review.md ## Reviewer cadence`; referenced here, not redefined).

For `status: accepted`, `lint` enforces matrix presence, AC↔task coverage, and
final result tokens; `PENDING_HUMAN`, `UNVERIFIED`, `FAIL`, empty cells, and
unknown/free-text results fail. It also warns on `[NEEDS-CLARIFICATION]` and AC
lines missing an EARS keyword (resolve before `approved`). The legacy `done`
status remains lint-valid only for archived specs already under `_done/`; the
engine never writes `done` for an active spec.

`mochiflow accept` does not convert provisional `UNVERIFIED` rows to `PASS`.
Build/open must settle automated rows to `PASS`, `FAIL`, or `N/A: <reason>`
before running accept.

`build` never sets `accepted`.

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

## Acceptance adapters (open)

Open identifies human-operated and visual QA items from `spec.md` QA Scenarios
(the `Type` column) and picks the adapter by `Scope` / kind:

| Scope / kind | adapter | main checks |
| --- | --- | --- |
| automated test | command verification | build/lint/test command + result |
| `api` | API QA | status / schema / error / auth / health |
| `web` | Browser QA | route / DOM / validation / network / responsive |
| configured app/device surface | app/device QA | simulator, device, accessibility, or visual check as applicable |
| `human` | Human confirmation | physical device, judgement, visual, external service |
| `cross-surface` | contract / workflow QA | contract or workflow across surfaces |

Human/visual AC are requested once, in open, via the QA round-trip protocol —
not during build. During build, mark those Matrix rows `PENDING_HUMAN` with the
needed QA scenario and evidence expectation.
