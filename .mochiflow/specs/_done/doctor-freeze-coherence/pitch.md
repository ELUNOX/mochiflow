# Clarify doctor/freeze coherence and context freshness

## Problem

MochiFlow currently has three related quality-gate gaps:

- `doctor engine` and `freeze --check` both mention engine integrity, but they
  protect different scopes. `doctor engine` validates the installed/vendored
  engine MANIFEST, while `freeze --check` validates source-repo derived files.
  The distinction is documented in fragments but is not obvious from the CLI
  workflow.
- The foundational context layer can become misleading after CLI commands or
  source layout change. `doctor config` detects missing context and unfilled
  stubs, but it does not warn when existing context references removed public
  commands.
- `freeze` can only resolve the source repo by walking upward from the current
  working directory. CI and scripts cannot explicitly point it at the source
  repo root.

These gaps cause contributors and agents to run the wrong check, miss stale
orientation prose, or make scripts depend on `cd` discipline.

## Appetite

This is worth a focused CLI and documentation improvement, not a redesign of
project health checking. The change should preserve the existing split:
`doctor` remains the installed-project health check, and `freeze --check`
remains the source-repo derived-file check.

## Solution

- Keep responsibility separation intact. `doctor engine` continues to validate
  the installed/vendored engine MANIFEST. `freeze --check` continues to validate
  source-repo derived files (`engine/VERSION`, `engine/MANIFEST.json`, and
  `contracts/contracts.lock`).
- Add user-facing guidance so source-repo contributors see that both checks may
  be relevant: project health through `mochiflow doctor`, source derived-file
  coherence through `mochiflow freeze --check`.
- Add a narrow `doctor config` freshness warning for context files that mention
  non-existent public CLI commands using `mochiflow <command>` references. This
  should prompt `refresh-context` without treating prose freshness as a hard
  failure.
- Add `mochiflow freeze --root <source-repo> [--check]` while retaining the
  current cwd-upward source-root resolution when `--root` is omitted.
- Document the intended use in CLI-facing docs and README command guidance.

## Rabbit Holes

- Do not make the CLI regenerate or semantically diff the context layer. Context
  is agent-authored orientation prose, not a deterministic CLI output.
- Do not make `doctor` run `freeze --check` internally. That would merge
  installed-project health with source-repo release hygiene and contradict the
  existing Version SSOT decision.
- Do not infer `freeze`'s source root from `--config`; `--config` belongs to
  installed project configuration, while `freeze` targets the MochiFlow source
  repo.

## No-gos

- No change to the frozen contract hash definition.
- No change to `doctor engine`'s MANIFEST drift semantics.
- No automatic context rewriting.
- No new required dependency for command parsing.
- No direct CI configuration change unless plan finds tests cannot cover the
  new behavior locally.

## Alternatives Considered

- **Run `freeze --check` from `doctor`.** Rejected because it collapses two
  separately documented responsibilities and would fail for normal installed
  projects that are not MochiFlow source repos.
- **Use docs only for the doctor/freeze distinction.** Rejected because agents
  often rely on command output; a CLI hint or warning gives a local signal.
- **Detect context staleness by timestamps or full regeneration.** Rejected
  because both create false positives for prose that is intentionally minimal or
  manually refreshed.
- **Infer `freeze` root from `--config`.** Rejected because it gives a
  source-repo command project-config semantics and makes the mental model less
  clear.

## Open Questions

- None -- ready for plan.
