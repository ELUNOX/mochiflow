---
inclusion: manual
description: Prepares a MochiFlow release — decides the next version, updates CHANGELOG and version files, verifies, commits, and creates the release tag. Stops before pushing. Use when the user says "prepare a release", "cut a release", "bump the version", or "tag a release".
---

# Prepare a Release

This is the **release preparation** runbook for MochiFlow. Pushing a tag that
matches `**[0-9]+.[0-9]+.[0-9]+*` triggers `.github/workflows/release.yml`
(cargo-dist), which cross-builds, creates the GitHub Release, and publishes the
Homebrew formula. Your job here is everything up to and including creating the
tag — **never push**. The human pushes the tag after a final review.

## Quick Start

1. Determine the next version from the changes since the last release.
2. Update `CHANGELOG.md` with a new section.
3. Update the version files (`engine/VERSION`, `cli/Cargo.toml`, and
   `contracts/contracts.lock` if a frozen surface changed).
4. Verify: `cargo build` then `cargo test`, then `mochiflow lint` and
   `mochiflow doctor`.
5. Create the release commit.
6. Create the tag.
7. Stop. Report the tag and the exact push command for the human to run.

## Step 1: Determine the next version

### Find the current version

The release version of record is `engine/VERSION` (see the versioning policy
in `contracts/VERSIONING.md`). Cross-check:

```bash
cat engine/VERSION                      # release version of record (X.Y.Z)
grep '^version' cli/Cargo.toml          # workspace package version
cat contracts/contracts.lock           # {version, hash} of frozen surface
git describe --tags --abbrev=0          # last released tag
```

> **Known drift to watch for:** these can fall out of sync (e.g. `engine/VERSION`
> = `1.1.0` while `cli/Cargo.toml` = `1.0.0-alpha.1`). Part of release prep is
> reconciling them so the tag, `engine/VERSION`, `cli/Cargo.toml`, and
> `CHANGELOG.md` all agree on one version. Flag the discrepancy to the user and
> confirm the intended target version before proceeding.

### Decide the bump (semver)

Review what changed since the last tag:

```bash
git log $(git describe --tags --abbrev=0)..HEAD --oneline
```

Classify by Conventional Commit type and choose the bump:

- **major (X)** — breaking change: incompatible `config.toml` / `spec.yaml`
  format, removed CLI command or flag, breaking schema change. Commits with
  `!` or `BREAKING CHANGE:`.
- **minor (Y)** — new feature, backward compatible (`feat:`).
- **patch (Z)** — bug fix or internal-only change (`fix:`, `refactor:`,
  `chore:`, `docs:` that ship in the binary).

Prereleases use a suffix: `1.2.0-alpha.1`, `1.2.0-beta.1`, `1.2.0-rc.1`. A
suffix makes cargo-dist mark the GitHub Release as a prerelease.

Always confirm the chosen version with the user before editing files.

## Step 2: Update CHANGELOG.md

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/) with
`Added` / `Changed` / `Deprecated` / `Removed` / `Fixed` / `Security` groups.

- Add a new `## [X.Y.Z] - YYYY-MM-DD` section above the previous one (use today's
  date).
- Summarize the commits since the last tag, grouped by category, in user-facing
  prose — not raw commit messages.
- Only include changes that affect users of the shipped binary.

## Step 3: Update the version files

Update every version location so they agree on the target version:

| File | What to change |
| --- | --- |
| `engine/VERSION` | The single line `X.Y.Z` (release version of record). |
| `cli/Cargo.toml` | `[workspace.package] version = "X.Y.Z"`. |
| `CHANGELOG.md` | New section (Step 2). |
| `contracts/contracts.lock` | **Only if a frozen surface changed** — see below. |

`Cargo.lock` updates automatically on the next `cargo build`; do not hand-edit it.

### Frozen-surface / contract changes (version gate)

The contract surface is frozen by `contracts/contracts.lock`. Per
`contracts/VERSIONING.md` and `CONTRIBUTING.md`, the hash covers:

- `contracts/*.json` (schemas)
- `tests/conformance/golden/**`

If this release **touches a schema or a golden fixture**, then in the **same
commit** you must:

1. Bump `engine/VERSION` (already done in Step 1/3).
2. Add the `CHANGELOG.md` section (Step 2).
3. Regenerate `contracts/contracts.lock` so its `version` matches the new
   `engine/VERSION` and its `hash` matches the new surface.

The `cargo test` version-gate check fails if these are inconsistent. Editing
engine prose (`commands/**`, `reference/**`, templates) does **not** trip the
gate — it only updates `engine/MANIFEST.json`.

> Do not edit `schema_version` in `config.toml`; it only changes on
> consumer-facing file-format breaks and is not part of normal version bumps.

## Step 4: Verify

Run the canonical verification before committing:

```bash
cargo build --manifest-path cli/Cargo.toml
cargo test  --manifest-path cli/Cargo.toml   # canonical verify (conformance suite)
```

Then run the MochiFlow gates (the project dogfoods itself):

```bash
mochiflow lint
mochiflow doctor
```

All must pass. The conformance suite enforces the version gate, MANIFEST drift,
and schema/golden consistency, so a green `cargo test` confirms the version
files line up.

## Step 5: Create the release commit

Stage the changed files explicitly (never `git add .`):

```bash
git add engine/VERSION cli/Cargo.toml cli/Cargo.lock CHANGELOG.md
# include only if a frozen surface changed:
git add contracts/contracts.lock contracts/*.json tests/conformance/golden
git commit -m "chore: release vX.Y.Z"
```

## Step 6: Create the tag

```bash
git tag vX.Y.Z
```

Match the version exactly, with a leading `v` (e.g. `v1.2.0`, `v1.2.0-alpha.1`).

## Step 7: Stop and hand off

**Do not push.** Pushing the tag is what triggers the public release pipeline
(cross-build + GitHub Release + Homebrew publish), so it is a human gate.

Report:

- The chosen version and why (bump rationale).
- The files changed.
- Verification results (`cargo test`, `lint`, `doctor`).
- The exact command for the human to run after review:

  ```bash
  git push origin <branch> --follow-tags
  ```

  (or `git push origin main && git push origin vX.Y.Z`)

## Done Checklist

- Next version chosen and confirmed with the user (semver-correct).
- `engine/VERSION`, `cli/Cargo.toml`, and the tag all agree on the version.
- `CHANGELOG.md` has a dated section summarizing user-facing changes.
- If a frozen surface changed: `contracts/contracts.lock` regenerated in the
  same commit.
- `cargo test`, `mochiflow lint`, `mochiflow doctor` all pass.
- Release commit created with `chore: release vX.Y.Z`.
- Tag `vX.Y.Z` created locally.
- **Not pushed** — push command reported to the human for final review.
