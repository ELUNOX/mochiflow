# Present next-step choices after plan approval

## Problem

After plan sets `status: approved` and commits, the agent presents only two
options: a new-session handoff card or an inline "continue with implementation"
phrase. There is no structured moment offering a spec/design review before
build — users must know to ask for it. First-time users miss the option entirely,
and elevated/critical specs proceed without review.

## Appetite

Small — a documentation-only change to `engine/commands/plan.md` step 10. No CLI
code, no new templates, no router changes.

## Solution

Replace plan.md step 10 with a 3-choice card presented after the plan commit:

- **review** — run `mochiflow-review` (spec/design quality review). On pass,
  re-present build/later. On fail, report findings and stop.
- **build** — proceed to `mochiflow-build` in the same session.
- **later** — output the existing handoff card from
  `templates/handoff/build-session-prompt.md` and stop.

Display order depends on risk:
- `risk >= elevated`: review (← recommended) / build / later
- `risk = standard`: build / review / later

Choice keywords (`review`, `build`, `later`) are stable identifiers. Surrounding
labels are presented in conversation language.

## Rabbit Holes

- Do not make review mandatory — ad-hoc review is optional per risk.md.
- Do not add new router triggers — the keywords are handled inline within plan's
  completion output, not as standalone verb activations.

## No-gos

- No CLI code changes.
- No changes to router.md, risk.md, review.md, or templates.
- No forced review for any risk level.
- No new handoff template.

## Alternatives Considered

- Always fixed order (build first) — elevated/critical specs would miss the
  review nudge.
- Making review mandatory for elevated/critical — conflicts with "ad-hoc review
  is user-triggered and optional" in risk.md.
- Adding translated keywords (`レビュー`/`実装`/`あとで`) — increases router
  complexity without benefit since these are inline plan completions.

## Open Questions

- None — ready for plan.
