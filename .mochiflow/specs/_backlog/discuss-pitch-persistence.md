---
slug: "discuss-pitch-persistence"
title: "Persist discuss output as pitch.md in the spec directory"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Persist discuss output as pitch.md in the spec directory

## Signal

When discuss reaches agreement and flows directly into plan (no backlog
detour), the discussion context exists only in the conversation. If the session
dies between discuss and plan, or if plan starts in a new session, the agreed
decisions, constraints, and scope are lost. The user must re-explain everything.

Industry-standard approaches (Rust RFCs, Google Design Docs, Shape Up Pitches)
all persist discovery output as a formal document before the next phase begins.

## Why It Matters

- Discussion outcomes have high re-creation cost (repeating the same debate).
- Plan quality degrades without a written input (it hallucinates missing context).
- No traceability from discuss decisions to spec content.
- `_backlog/ready-for-plan` partially solves this but places the file in the
  wrong location (backlog = uncommitted ideas, not agreed work).

## Proposed Solution (Shape Up Pitch + Rust RFC hybrid)

### Discuss completion

When discuss reaches agreement:
1. Create `{specs_dir}/{slug}/` directory.
2. Write `spec.yaml` with `status: draft`.
3. Write `pitch.md` — the structured discuss output containing:
   - Problem statement
   - Agreed scope (in / out)
   - Key decisions made during discuss
   - Assumptions and constraints
   - Open questions (for plan to resolve)
4. Commit: `discuss({slug}): record pitch`
5. Present next-step choices: `[plan / review / later]`

### Plan uses pitch.md as input

Plan reads `pitch.md` + `spec.yaml` (status: draft) and generates:
- `spec.md` (AC, QA scenarios — pitch.md Background absorbed here)
- `design.md` (when needed)
- `tasks.md` (when needed)
- Updates `status: draft → approved`

`pitch.md` is deleted after plan absorbs it into spec.md Background (or kept
as a historical artifact — TBD).

### _backlog/ role simplification

- `maturity: seed` — idea notes, discuss input candidates. Unchanged.
- `maturity: ready-for-plan` — **deprecated**. Replaced by
  `{specs_dir}/{slug}/pitch.md` + `status: draft`.
- Existing ready-for-plan files remain supported for backward compatibility
  (plan still reads them if no spec directory exists).

### Spec lifecycle with new status

```
(no directory)  →  discuss  →  {slug}/ created (status: draft, pitch.md)
                                  →  plan  →  status: approved (spec.md, design.md, tasks.md)
                                                →  build  →  ship  →  status: done (_done/)
```

## Decisions (tentative)

- pitch.md is a structured template (not freeform notes).
- pitch.md lives in `{specs_dir}/{slug}/` — same home as all spec artifacts.
- `status: draft` means "discussed but not yet planned".
- Committing at discuss end means discuss output survives session loss.
- plan-approval-commit (separate seed) still applies — plan commits on approval.
- This is primarily engine docs changes: `commands/discuss.md`, `commands/plan.md`,
  `reference/workflow.md`, `templates/` (new pitch template).
- `mochiflow lint` needs to accept `status: draft` with only spec.yaml + pitch.md
  (no spec.md required at draft).

## References

- Rust RFC process: file created at discussion start, status transitions in header.
- Google Design Docs: document written before coding, reviewed and approved.
- Shape Up: Pitch is the shaped output presented at the Betting Table before build.
