---
slug: "develop-branch-workflow"
title: "Introduce develop branch to reduce PR ceremony for non-code changes"
surface: "cli"
type_hint: "refactor"
maturity: "seed"
source: "conversation"
source_phase: "ship"
created: "2026-06-22"
updated: "2026-06-22"
---

# Introduce develop branch to reduce PR ceremony for non-code changes

## Signal

Every change — including backlog seeds, spec files, context refreshes, and typo
fixes — requires a feature branch + PR + CI pass due to main branch protection.
This creates disproportionate overhead for documentation-only changes that carry
zero runtime risk.

## Why It Matters

- Adding a backlog seed (1 markdown file) requires: branch, commit, push, PR
  creation, CI wait, merge, cleanup. ~5 minutes of ceremony for 0 risk.
- Spec files (discuss/plan output) are pre-implementation and cannot break
  anything, yet face the same gate as production code.
- This friction discourages frequent small improvements and context updates.

## Impact Analysis

### Files that reference `main` or `base_branch`

| File | Reference | Change needed |
|------|-----------|---------------|
| `.mochiflow/config.toml:26` | `base_branch = "main"` | Change to `"develop"` |
| `.github/workflows/ci.yml:5` | `push: branches: [main]` | Add `develop` |
| `.github/workflows/release.yml` | Tags only, no branch ref | No change |
| `.kiro/steering/release.md:166` | `git push origin main` | Update to release flow |
| `CONTRIBUTING.md:84` | "direct push to main is discouraged" | Update wording |
| `CONTRIBUTING.ja.md:81` | Same in Japanese | Update wording |
| `engine/reference/git.md:23` | `origin/{[git].base_branch}` | No change (dynamic) |
| `engine/reference/git.md:195-196` | Post-merge cleanup uses `{[git].base_branch}` | No change (dynamic) |

### Engine docs (use `{[git].base_branch}` placeholder)

Engine references are already parameterized via `{[git].base_branch}`. Changing
the config value from `main` to `develop` propagates automatically. No engine
prose edits needed.

### CI workflows

- `ci.yml`: Currently triggers on `push: branches: [main]` and `pull_request`.
  Must add `develop` to push triggers so CI runs on develop pushes too.
- `release.yml`: Triggers on version tags only. No branch reference. Unaffected.
  Release tags are created from main (after develop → main merge).

### GitHub branch protection

| Branch | Current | Proposed |
|--------|---------|----------|
| main | PR required, CI required, no direct push | Same (release gate) |
| develop | Does not exist | Create with relaxed rules: allow direct push, CI on push |

### mochiflow pr

`mochiflow pr` reads `[git].base_branch` for the PR target. Changing to
`develop` means feature branches merge into develop via PR. No code change
needed — just config.

### Release workflow change

Current: feature → main (via PR) → tag → release
Proposed: feature → develop (via PR) → develop → main (release PR) → tag → release

Release steering must add a "develop → main" PR step before tagging.

## Proposed Solution

### Branch structure

```
main ────────────────── releases only (tagged)
  │
develop ─────────────── daily integration (base_branch)
  │
  ├─ feat/slug ──────── code changes (PR → develop)
  └─ (direct push) ──── docs, specs, backlog, context (no PR)
```

### What can be pushed directly to develop (no PR)

- `_backlog/*.md` (backlog seeds)
- `{specs_dir}/{slug}/` (spec files from discuss/plan)
- `.mochiflow/context/` (refresh-context output)
- `.mochiflow/adr/` (ADR fold)
- `docs/**` (documentation)
- Typo fixes in markdown

### What requires a PR to develop

- `cli/**` (Rust code)
- `engine/**` (engine source)
- `contracts/**` (frozen surface)
- `.github/workflows/**` (CI)
- `tests/**` (conformance fixtures)

### Release flow (develop → main)

1. All specs shipped, develop is stable.
2. Create release PR: develop → main.
3. CI runs on the PR.
4. Merge (squash or merge commit — TBD).
5. Tag on main. cargo-dist triggers.

## Decisions (tentative)

- `[git].base_branch` = `"develop"` in config.
- CI triggers on both `push: [main, develop]` and `pull_request`.
- Feature branches cut from develop, merge into develop.
- Direct push to develop allowed for docs/specs only.
- main remains protected (PR + CI required).
- Release is a develop → main PR followed by a tag.
- No change to `mochiflow pr` CLI code (reads config dynamically).
- Engine docs need no prose change (already use `{[git].base_branch}`).
