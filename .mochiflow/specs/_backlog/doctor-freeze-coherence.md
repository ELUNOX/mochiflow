---
slug: "doctor-freeze-coherence"
title: "Clarify doctor vs freeze responsibilities and add context freshness check"
surface: "cli"
type_hint: "feature"
maturity: "seed"
source: "conversation"
source_phase: "build"
created: "2026-06-22"
updated: "2026-06-22"
---

# Clarify doctor vs freeze responsibilities and add context freshness check

## Signal

Three related gaps in the diagnostic and integrity tooling:

1. **`doctor engine` vs `freeze --check` responsibility overlap** — `doctor
   engine` checks vendored engine MANIFEST integrity (file hashes match
   MANIFEST.json). `freeze --check` checks source repo derived files
   (VERSION/MANIFEST/lock match SSOT). Users (and AI agents) don't know which to
   run when. In a source repo both are relevant; in an installed project only
   doctor applies. The distinction is undocumented.

2. **`context/` freshness is never checked** — The `.mochiflow/context/`
   files (product.md, structure.md, tech.md) go stale when commands are
   added/removed, modules change, or dependencies shift. The independent
   reviewer caught stale context referencing a removed command (`engine
   manifest`). There is no automated signal that context needs refresh.

3. **`resolve_repo_root` is cwd-only** — freeze resolves the repo root by
   walking up from cwd. Other commands accept `--config` for explicit paths. If
   CI runs from a subdirectory without `cd`-ing to root first, freeze fails.
   Adding `--root` or inferring from `--config` would align with other commands.

## Why It Matters

Ambiguity between doctor and freeze confuses contributors (which do I run?).
Stale context actively misleads AI agents reading it as orientation. The cwd
limitation may trip CI setups that don't `cd` to repo root.

## Decisions (tentative)

- Add a "which tool to use when" section to docs/versioning.md or a new
  docs/integrity.md.
- Consider `doctor` emitting a WARN when run in a source repo and `freeze
  --check` would fail (lightweight cross-check without duplicating logic).
- Add a lightweight context-freshness heuristic to `doctor` (e.g. check if
  context/structure.md references commands that don't exist in the binary's
  subcommand list).
- Consider `--root` flag for freeze, or infer repo root from `--config` path.
