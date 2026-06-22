---
slug: "ship-qa-experience"
title: "Ship QA experience: rework loop, result collection, PR testing section"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Ship QA experience: rework loop, result collection, PR testing section

## Signal

Four related gaps in ship's human QA flow surfaced during the version-ssot-freeze
delivery and subsequent UX discussion:

1. **QA FAIL has no defined rework path** — ship.md lacks an explicit flow for
   when human QA returns FAIL. The stop condition prevents `status: done`, but
   there is no guidance on how to fix and retry. Agents improvise or get stuck.

2. **PR Feedback Loop startup is invisible** — the loop exists in ship.md but
   users don't know about it or how to trigger it when a team member requests
   changes on the PR.

3. **QA result collection UX is undefined** — ship asks for human QA but
   doesn't specify how results are communicated back. Users don't know if they
   should say "OK", type in a file, or provide screenshots.

4. **QA instructions don't reach PR reviewers** — `qa-instructions.md` is
   ephemeral/gitignored. Team members reviewing the PR have no testing guidance
   unless they dig into the archived spec.

## Why It Matters

- QA FAIL without a rework path stalls the entire delivery.
- Invisible PR Feedback Loop means users manually figure out state transitions.
- Unstructured result collection leads to ambiguous PASS/FAIL interpretations.
- Missing PR QA section forces reviewers to test blind or skip QA entirely.

## Proposed Solution

### QA Rework loop (ship-internal)

When human QA returns FAIL on any item:
1. Record FAIL + reason in AC Matrix.
2. Ship pauses (status stays `approved`, never reaches `done`).
3. Run build-equivalent fix loop (modify → verify → commit).
4. Re-request QA on FAIL items only.
5. On all PASS → resume ship step 4.

No spec move, no status change — lighter than PR Feedback Loop.

### PR Feedback Loop startup UX

- Add router triggers: `{slug} feedback` / 「修正依頼」/ 「PR feedback」.
- On trigger, announce: "PR Feedback Loop に入ります: spec を active に戻し、
  修正 → 再 ship します。" then execute the existing loop steps.

### Structured QA result collection

Present QA items as a numbered list with expected results. Collect via:
- `OK` / `PASS` / `✓` → 人間確認済み
- `NG` / `FAIL` / `✗` + optional reason → FAIL (reason becomes evidence)
- `全部OK` → all items PASS
- Unanswered items → re-ask

### PR body `## Testing` section

When ship generates `pr-body.md`, include a `## Testing` section derived from
spec.md QA Scenarios with concrete reproduction steps. This replaces
`qa-instructions.md` as the team-facing QA artifact. The ephemeral
`qa-instructions.md` can be removed — AI/author QA uses conversation directly,
reviewer QA uses the PR body.

## Decisions (tentative)

- QA Rework is a ship.md procedure addition (engine docs change).
- PR Feedback Loop triggers are a router.md addition.
- Result collection format is a plan.md / ship.md Presentation rule.
- `## Testing` in PR body is a `templates/delivery/pr-description.md` change.
- `qa-instructions.md` generation becomes optional or removed.
