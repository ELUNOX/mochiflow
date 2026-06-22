---
slug: "plan-to-build-transition-ux"
title: "Present next-step choices after plan approval"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Present next-step choices after plan approval

## Signal

After plan sets `status: approved`, the agent either continues into build (if
the user says so) or presents a handoff prompt for a new session. There is no
structured moment where the user is offered a choice — including the option to
run a spec/design review before committing to implementation.

## Why It Matters

- First-time users don't know what actions are available after plan.
- Elevated/critical specs benefit from a design review before build, but users
  must know to ask for it explicitly.
- The transition is a natural decision point that should surface options.

## Proposed Solution

When plan completes and `status: approved` is set, the agent presents:

```
✅ Spec approved ({slug}, risk={risk})

次のステップ:
  [review]  spec/design をレビューしてから実装に進む{recommended}
  [build]   このまま実装に進む
  [later]   別セッション用の起動プロンプトを出す

どれにしますか？
```

Where `{recommended}` is ` ← 推奨` when `risk ≥ elevated`, absent otherwise.
Order of choices shifts by risk: elevated/critical puts review first, standard
puts build first.

Behavior per choice:
- **review** — run `mochiflow-review` (spec/design quality review, not code
  review). On pass/pass-with-comments, re-present build/later. On fail, user
  addresses findings before proceeding.
- **build** — proceed to `mochiflow-build` in the same session.
- **later** — output a minimal startup prompt (`{slug} build`) and stop. With
  plan-approval-commit, the spec is already committed, so no context is lost.

## Decisions (tentative)

- This is an engine docs change to `commands/plan.md` (Presentation / completion
  section).
- The review at this stage checks spec quality + design soundness (AC
  coverage, task decomposition, risk assessment, scope creep), not code.
- Conversation language (`auto` / explicit tag) governs the presented text.
- The choice keywords (`review`, `build`, `later`) are router-recognizable
  triggers.
