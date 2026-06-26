# Clarify choice-card commands and numbered replies

## Problem

MochiFlow's phase-completion prompts expose internal workflow words such as
`approved`, `build`, `ship`, and `later` too directly. In particular, asking for
approval with `OK` / `承認` / `LGTM` / `approved` makes it look like the agent
will immediately begin implementation, even when the intended action is only to
confirm and commit the plan. The `later` label also hides the actual output: a
prompt for resuming in a new session.

At the same time, asking users to type long action labels such as
`計画を作る` or `実装を開始する` every time is unnecessarily heavy. The UX needs
plain user-facing action labels, short explicit triggers, and safe numbered
replies for the most recent choice card.

## Appetite

Medium. This is a documentation and agent-behavior contract change spanning the
phase procedures and shared routing guidance. It should not require CLI runtime
code, persistent UI state, or a new PR-body editing command.

## Solution

Define choice cards around user-facing actions rather than internal phase names.
Every displayed card may include numbered options, and users may reply with the
number instead of typing the full action name. Numbers are ephemeral aliases for
the most recent choice card only; they are not durable commands and must not be
interpreted without a live, unambiguous choice card in the current conversation.

Use these phase-specific choices:

- After discuss completes:
  - `計画を作る` — triggers: `計画を作る`, `plan`, `mochiflow-plan`.
  - `再開用プロンプトを作る` — triggers: `再開用プロンプトを作る`, `resume`, `later`.
- After plan draft is ready:
  - `計画を確定` — triggers: `計画を確定`, `approve plan`, `approved`.
  - Ordinary correction feedback revises the plan and re-presents it; do not add
    a `計画を修正する` command.
- After plan is confirmed and committed:
  - `レビューする` — triggers: `レビューする`, `review`, `mochiflow-review`.
  - `実装を開始する` — triggers: `実装を開始する`, `build`, `mochiflow-build`.
  - `再開用プロンプトを作る` — triggers: `再開用プロンプトを作る`, `resume`, `later`.
- After ad-hoc review reports:
  - `実装を開始する` — triggers: `実装を開始する`, `build`, `mochiflow-build`.
  - `再開用プロンプトを作る` — triggers: `再開用プロンプトを作る`, `resume`, `later`.
- After build completes:
  - `PR準備を始める` — triggers: `PR準備を始める`, `ship`, `mochiflow-ship`.
  - `再開用プロンプトを作る` — triggers: `再開用プロンプトを作る`, `resume`, `later`.
- After PR title/body are presented:
  - `PRを作成する` — triggers: `PRを作成する`, `create pr`, `approved`.
  - PR text corrections are ordinary feedback: revise the PR text and re-present
    it. Do not add a `PR本文を修正する` command.

Visible approval UX should say `計画を確定`, not `承認`. It must state that the
action updates `spec.yaml` to `status: approved`, re-checks consistency, and
commits the plan artifacts, but does not start implementation. The old
approval words may remain compatibility inputs for delivery gates, but they
should not be the primary displayed action labels.

Only show `再開用プロンプトを作る` at the agreed high-value handoff points:
discuss completion, plan confirmation, review completion, and build completion.
Do not make it a permanent option on every phase prompt.

## Rabbit Holes

- Do not turn numbers into global commands. `1` means nothing outside the most
  recent choice card.
- Do not require users to type long Japanese labels when a displayed number is
  enough.
- Do not add a dedicated PR-body editing command. Free-form feedback is clearer
  for that moment.
- Do not make `計画を確定` imply immediate implementation.
- Do not show a resume-prompt option at every possible pause point.

## No-gos

- No CLI runtime code unless plan discovers that documentation alone cannot make
  adapter behavior consistent.
- No persistent conversation-state file for numbered replies.
- No new `PR本文を修正する` command.
- No visible recommendation to use `OK`, `承認`, or `LGTM` for plan confirmation.
- No change to the rule that PR creation still needs an explicit approval action.

## Alternatives Considered

- Keep `OK` / `承認` / `LGTM` / `approved` as the main plan-confirmation wording
  — rejected because it reads like implementation approval and hides the actual
  operation.
- Use only stable internal keywords such as `review`, `build`, and `later` —
  rejected because first-time users see workflow jargon instead of actions.
- Add `再開用プロンプトを作る` everywhere — rejected because it makes common choice
  cards noisy.
- Add `PR本文を修正する` — rejected because PR text edits are better handled as
  ordinary feedback before the PR creation gate.
- Make numbered choices durable across sessions — rejected because the same
  number can mean different actions in different cards.

## Open Questions

- None — ready for plan.
