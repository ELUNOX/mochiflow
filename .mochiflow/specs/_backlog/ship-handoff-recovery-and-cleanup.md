---
slug: "ship-handoff-recovery-and-cleanup"
title: "Improve ship handoff recovery and post-merge cleanup"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "doctor-freeze-coherence"
source_phase: "ship"
created: "2026-06-24"
updated: "2026-06-24"
---

# Improve ship handoff recovery and post-merge cleanup

## Signal

During `doctor-freeze-coherence` ship, `mochiflow pr` first failed at push with:

`Could not resolve host: github.com`

The raw git error was accurate, but the command did not provide MochiFlow-level
recovery guidance. After merge, local cleanup also required a manual sequence:
switch to base, fast-forward pull, safe-delete the feature branch, and remove
the spec's gitignored state directory.

## Why It Matters

PR handoff is the point where users need clear recovery options for network,
credential, and provider failures. Post-merge cleanup is deterministic local
hygiene and intentionally does not commit or push, so it is a good candidate for
a small command that reduces repeated manual steps.

## Evidence

- Retrying the same `mochiflow pr` command with network access succeeded and
  created the PR.
- The documented post-merge cleanup sequence succeeded after the merge.
- Several cleanup steps needed `.git` writes, but the operation was routine and
  safe.

## Decisions (tentative)

- Wrap push/provider failures with actionable guidance: retry with
  network/credentials, or manually open the PR with the approved title/body if
  the branch was pushed.
- Preserve the original git/provider error for diagnosis.
- Consider distinct exit codes or structured JSON fields for push failure vs PR
  creation failure.
- Add a command such as `mochiflow ship cleanup --spec <slug>` or
  `mochiflow cleanup --spec <slug>` that implements documented local hygiene.

## Guardrails

- Cleanup must not touch remote branches, create commits, or push.
- Branch deletion should remain safe-delete only.
- State removal should be limited to the spec's gitignored delivery scratch.

## Open Questions

- Should PR failure recovery and post-merge cleanup ship as one CLI improvement,
  or should cleanup be a separate small command after error guidance lands?
- What exit codes should distinguish push failure, provider PR failure, and
  manual-handoff success?
