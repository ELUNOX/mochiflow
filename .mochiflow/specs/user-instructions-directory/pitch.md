# Add user-owned MochiFlow instructions directories

## Problem

Users sometimes need a place near MochiFlow to keep their own Markdown
instructions: release procedures, repository-specific operating notes, personal
checklists, or prompts they want an agent to read only when explicitly cited.

The current fallback is to put this material in
`.mochiflow/constitution.local.md`, but that file is always-loaded standing
guidance. That makes occasional, task-specific material too prominent and mixes
MochiFlow's constitutional rules with user-authored notes that are not part of
MochiFlow's route contract.

Users need a simple, discoverable Markdown home under `.mochiflow/` that
MochiFlow does not parse, index, load, validate, or treat as engine drift.

## Appetite

This is a small workflow feature across initialization, local ignore policy,
detach messaging, and public docs. It should not become a plugin system, skill
registry, prompt compiler, or adapter integration.

The target is a conservative v1: scaffold an obvious place, document the
semantics, and otherwise leave the files fully user-owned.

## Solution

Create two user-owned Markdown directories during `mochiflow init`:

- `.mochiflow/instructions/` for shareable instructions that should remain a
  normal tracked candidate.
- `.mochiflow/instructions.local/` for private local instructions that should
  be ignored by Git.

Add `instructions.local/` to `.mochiflow/.gitignore`. Do not add
`instructions/` to `.mochiflow/.gitignore`.

Seed `.mochiflow/instructions/README.md` during init as a lightweight
explanation of the contract:

- these directories are for user-authored Markdown;
- MochiFlow never loads the files automatically;
- agents should use a file only when the user explicitly provides the path;
- MochiFlow does not parse frontmatter, build an index, validate filenames, or
  include the directories in drift checks;
- `instructions/` is shareable by normal Git policy;
- `instructions.local/` is local-only by default;
- `detach --purge` removes both because it removes the entire `.mochiflow/`
  tree.

The README is user-owned after creation. `mochiflow init` should create it only
when missing, and `mochiflow init --force` should not overwrite it.

Do not backfill the directories from `join` or `upgrade`; those commands should
avoid surprising users with new tracked candidates in existing repositories.
Existing projects can create the directories manually or receive them the next
time they intentionally run init in a context where scaffolding is appropriate.

Update user-facing docs and init output so users can discover the path without
adding adapter, router, or constitution references. The adapter should continue
to mention only the always-loaded constitution files and the MochiFlow router.

Keep `doctor`, `freeze`, engine drift checks, and adapter generation out of this
feature. They should neither require nor inspect the instructions directories.
Normal `detach` should leave both directories in place. `detach --purge` should
warn clearly that user instructions under `.mochiflow/instructions/` and
`.mochiflow/instructions.local/` will be deleted.

As a dogfood migration in this repository, move the local maintainer release
procedure currently stored in `.mochiflow/constitution.local.md` to
`.mochiflow/instructions.local/release.md`, leaving
`.mochiflow/constitution.local.md` as the default local constitution stub. This
local migration is not PR content because both files are local-only here.

## Rabbit Holes

- Do not create a plugin system, skill system, registry, manifest, or loader.
- Do not add `SKILL.md`, frontmatter, generated indexes, state caches, or search
  metadata.
- Do not mention the instructions directories from generated adapters, the
  router, or constitution files.
- Do not make MochiFlow infer when these files should be loaded from filenames
  or directory contents.
- Do not backfill existing installs through `upgrade` or `join`.

## No-gos

- No automatic loading into agent context.
- No drift failure for user-authored Markdown under either instructions
  directory.
- No gitignore entry for `.mochiflow/instructions/`.
- No overwrite of user-authored README or Markdown files after creation.
- No requirement that these files use frontmatter or any naming convention.

## Alternatives Considered

- **Keep using `constitution.local.md`.** Rejected because it is always-loaded
  standing guidance, while release procedures and optional notes should be
  loaded only when explicitly requested.
- **Build a plugin-like extension model.** Rejected because the user need is
  plain Markdown storage, not execution, discovery, packaging, or installation.
- **Add frontmatter and index files.** Rejected because they imply MochiFlow
  owns or understands the content. The desired contract is explicit path usage.
- **Mention the directory from adapters or router guidance.** Rejected because
  that would make user notes feel like part of the standing route contract.
- **Gitignore both directories.** Rejected because shared repository
  instructions should be easy to commit when a team wants them.

## Open Questions

- None -- ready for plan.
