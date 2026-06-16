# Engineering Standards Reference

How changes are made: every change conforms to the **official, standard best
practice** of the language, framework, and tools in use. "It runs" is not the
bar; the bar is the approach the upstream project recommends. This applies to
all verbs; depth scales with `risk.md`.

## Principle

Prefer the upstream-recommended approach over a locally-invented one. Verify
against **primary sources** — the official documentation of the relevant
language / framework / tool, read at the version actually in use — not memory,
blog posts, or "whatever compiles". When memory and a primary source disagree,
the primary source wins. Do not adopt an approach on assumption; confirm it.

## When this applies

Required for non-trivial technical decisions and any change that:

- adds or swaps a dependency, or changes how a tool / runtime is invoked,
- adopts a framework / library idiom or API surface,
- touches a boundary contract, persistence, or build / deploy shape,
- deviates from the upstream default.

Not required for trivial, reversible `standard`-risk edits (copy, in-screen
display, local refactors) where no tool / framework idiom is in question.

## Record the basis

For each in-scope decision, record the chosen approach **and its primary
source** (official doc URL + version or date) where that decision already lives:
`spec.md` 背景と設計判断 / `design.md` 設計判断 / build's `## 統合ログ` for fixes.
A decision without a verifiable basis is not done.

## Workarounds are last resort

If the standard approach genuinely cannot be used, label the change a
**Workaround** and record: (a) why the standard approach does not work, (b) what
the standard approach is, (c) the backlog seed slug tracking the proper fix
(`workflow.md ## Backlog seeds`). Never leave an unlabeled, untracked workaround.

## Project's canonical practices

Project-specific "what the standard approach is here" (chosen frameworks, run
commands, dependency tooling, linters) lives in the project context, not in this
engine file. This file states the rule; the project context is the catalog the
rule points to.
