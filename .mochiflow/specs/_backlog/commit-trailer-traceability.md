---
slug: "commit-trailer-traceability"
title: "Add git trailers for spec/task traceability and AI log reading recipes"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Add git trailers for spec/task traceability and AI log reading recipes

## Signal

Commit messages follow Conventional Commits but have no machine-parseable link
to the spec or task that motivated them. `git log` cannot answer "which commits
belong to spec X?" or "what was the last task completed?". The current rule
("no mochiflow vocabulary in commit messages") prevents putting slugs in the
subject/body, but trailers (footer metadata) are invisible to casual readers
and machine-parseable.

Additionally, when an AI agent resumes a build in a new session, it has no
standard procedure for reading git history to determine progress. It relies
solely on tasks.md checkboxes (which may not be up to date) and file reads.

## Why It Matters

- No commit-level traceability from code to spec/task decisions.
- Build resume in a new session requires re-reading all source files to infer
  progress instead of querying git.
- PR Feedback Loop diagnosis ("what was already done") requires reading full
  diff instead of targeted log queries.
- Review cannot quickly verify that each commit maps to a task.

## Proposed Solution

### Git trailers (write side)

Add to `reference/git.md`:

```
Spec: {slug}
Task: T-XXX
```

Rules:
- `Spec: {slug}` — required on every build/ship commit.
- `Task: T-XXX` — added when tasks.md exists and a specific task is being
  completed. Optional for ship close-out commits.
- Trailers go in the footer (after body, separated by blank line).
- Subject and body remain free of slugs/task IDs (external-reviewer rule kept).
- discuss/plan commits use `Spec: {slug}` only (no task yet).

### AI git log recipes (read side)

Add to `reference/git.md ## AI Git Log Recipes`:

```bash
# All commits for a spec
git log --format="%H %s%n%(trailers:key=Spec,key=Task,separator=%x2C )" \
  --grep="Spec: {slug}"

# Last completed task
git log --format="%(trailers:key=Task,valueonly)" \
  --grep="Spec: {slug}" | head -1

# Recent changes to a file with spec context
git log --format="%s | %(trailers:key=Spec,valueonly)" -- path/to/file -5
```

### Build resume procedure

Add to `commands/build.md` (resumption from a new session):

1. Read tasks.md checkboxes to identify completed tasks.
2. Run `git log --grep="Spec: {slug}"` to confirm implementation commits.
3. Resume from the first unchecked task.

## Decisions (tentative)

- Trailers are invisible in `git log --oneline`, GitHub PR subject view, and
  `git shortlog` — external-reviewer rule is satisfied.
- Uses native git trailer support (`git interpret-trailers`, `%(trailers:...)`
  format) — no custom tooling needed.
- `Spec:` and `Task:` keys are short, unambiguous, and unlikely to collide with
  existing trailers (Signed-off-by, Co-authored-by, etc.).
- This is engine docs only (reference/git.md, commands/build.md). No CLI change.
- Depends on tasks-checkbox-enforcement for reliable resume detection.
