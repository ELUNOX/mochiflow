---
id: 2026-06-26-do-not-make-localized-labels-canonical
date: 2026-06-26
area: [cli]
status: active
---
## Do not make localized choice labels canonical engine commands (2026-06-26)

**Applies to:** `engine/commands/*`, `engine/reference/workflow.md`, and
`engine/reference/language.md` choice-card guidance.
**Signal:** A non-Japanese conversation sees Japanese action labels, or the
engine requires one specific localized phrase instead of dispatching the visible
choice action.
**Cause:** Treating the Japanese wording used in a dogfood conversation as the
source-of-truth command vocabulary.
**Guardrail:** Define semantic actions in engine prose, render labels in the
conversation language, and keep stable internal tokens as compatibility inputs.
Numbers dispatch only the visible option in the most recent unambiguous card.
**Check:** Updated engine text should describe actions such as confirm plan,
start implementation, start PR preparation, create PR, and create resume prompt
without requiring Japanese literals as canonical triggers.
**Status:** Active.
