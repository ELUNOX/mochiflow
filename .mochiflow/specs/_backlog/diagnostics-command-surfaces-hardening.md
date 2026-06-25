---
slug: "diagnostics-command-surfaces-hardening"
title: "Harden diagnostics command surfaces and source-repo guidance"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "doctor-freeze-coherence"
source_phase: "post-merge"
created: "2026-06-24"
updated: "2026-06-24"
---

# Harden diagnostics command surfaces and source-repo guidance

## Signal

`doctor-freeze-coherence` added stale context warnings that must understand two
public command vocabularies:

- terminal CLI subcommands owned by clap
- workflow command references such as `mochiflow discuss`, `mochiflow plan`,
  `mochiflow build`, and `mochiflow ship`

The terminal command allowlist now has a parity test against clap, but workflow
verbs are still manually listed in doctor code. The same work also clarified
that `mochiflow doctor` remains a project health check while source-repo derived
file drift belongs to `mochiflow freeze --check`.

## Why It Matters

The stale-reference feature should not introduce another manual drift point.
Adding or removing `engine/commands/*.md` can make doctor silently accept or
reject the wrong workflow references unless the list is synced. Source-repo
contributors may also need a clearer diagnostics surface if the current doctor
warning becomes noisy or repetitive.

## Evidence

- `doctor_terminal_command_allowlist_matches_clap_subcommands` covers clap
  command drift.
- No equivalent test currently verifies workflow command references against
  `engine/commands/*.md`.
- Doctor now warns in the source repo and points users to
  `mochiflow freeze --check`, while `freeze --root <source-repo> --check` gives
  scripts an explicit source root.

## Decisions (tentative)

- Prefer deriving the workflow reference set from `engine/commands/*.md` or a
  small manifest generated from that directory.
- If runtime derivation is too invasive, add a conformance test that compares
  the doctor workflow allowlist with the actual command document set.
- Evaluate whether the source repo needs an explicit diagnostics surface such as
  `mochiflow doctor source`, or whether the current doctor guidance is enough.

## Open Questions

- Which engine command documents are public workflow references, and which are
  internal only?
- Should a source diagnostics command compose `freeze --check`, adapter checks,
  and other release hygiene, or only report the intended command set?

