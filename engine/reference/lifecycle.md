# Lifecycle Reference

The asserted spec states, the two delivery approval gates, and choice-card
dispatch. Per-verb procedure lives in
`commands/{discuss,plan,build,open,update,close}.md`. Spec depth, artifact
roles, and authoring live in `reference/specs.md`; the AC Matrix and verification
profiles live in `reference/verification.md`; delivery/PR mechanics live in
`reference/delivery.md`.

## Verbs and state

```
discuss ──▶ plan ──▶ build ──▶ open ──▶ (update) ──▶ close
```

| state | meaning | set by |
| --- | --- | --- |
| `draft` | pitch agreed, not yet approved for implementation | discuss |
| `approved` | human approved implementation start | delivery approval gate |
| `accepted` | acceptance conditions met (quality-complete) | open's accept close-out |

State lives in `spec.yaml` `status`. There is no separate per-document status.
The asserted states settle on the feature branch before merge. Delivery facts
(`in_review` when a PR is open, `merged` after merge) are **derived** from
VCS/provider and never stored; `done` is observed from the merge, never written
(`reference/delivery.md ## Derived delivery state`).
`mochiflow status` renders the live board (Backlog / Active / Ready / In Review /
Done) from the asserted state unioned with the derived delivery state.

## Delivery approval gates (exactly two)

1. **approve-to-build** — human approves the spec for implementation (`draft → approved`).
2. **approve-PR** — human approves PR title/description before `mochiflow pr` runs.

Approval words: `OK` / `承認` / `LGTM` / `approved`. They apply only to these two
delivery approval gates. The AI never sets `approved` without delivery approval
gate 1 and never creates a PR before delivery approval gate 2. Setup, context
refresh, and QA evidence may require human confirmations, but those confirmations
are not delivery approval gates and do not change spec lifecycle state except
where explicitly defined.

`accepted` is not a gate: it is an acceptance state that `open` sets through the
deterministic `mochiflow accept {slug}` mechanical close-out when the acceptance
conditions hold (`reference/verification.md ## AC Matrix`). The command updates
`spec.yaml` `status: accepted`, re-runs verification and lint, then follows the
repository's spec persistence mode (`reference/delivery.md ## Persistence
modes`). Tracked mode stages only close-out paths and creates the close-out
commit. Local mode keeps accepted spec artifacts ignored, skips staging/commit,
and relies on local accepted state plus PR body evidence.

Independent review (`agents/plan-auditor.md` before implementation and
`agents/change-reviewer.md` after implementation, whether the mandatory
risk-cadence run, `plan.md`'s pre-approval review for `risk >= elevated`, or
ad-hoc `mochiflow-review`) is a **quality assist, not a delivery approval gate**.
It informs the human's gate decision and a recorded `pass` / `pass-with-comments`
is one of the acceptance conditions, but review never sets `status` by itself and
never adds a third gate — there are exactly the two gates above
(`reference/review.md`).

When an approval gate is presented as a numbered choice card, selecting the
visible approval action by label or by its displayed number is the gate input.
For example, a plan card may display "confirm the plan" as the action that sets
`status: approved`; choosing that action dispatches approve-to-build. The old
approval words remain compatibility inputs, not the preferred user-facing label.

## Choice cards

Phase-completion choice cards present user-facing actions, each with a stable
action label and optional compatibility keywords. A displayed number is an
ephemeral alias for that action in the most recent unambiguous choice card only.
It is not a durable command, and it must not be interpreted without the active
card context.

Choice selection is the dispatch primitive: choosing a visible action by label,
compatibility keyword, or displayed number invokes that action. If a bare number
is stale, out of range, or contextless, ask the user to choose again using the
current action labels. Generic user-facing summary and action-card wording is
owned by `reference/presentation.md`; this section owns only the state-dispatch
semantics.
