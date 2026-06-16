<!-- Thanks for contributing to MochiFlow! Please fill out the sections below. -->

## Summary

<!-- What does this change do, and why is it needed? 1-3 lines. -->

## Changes

<!-- Bullet the logical units of change: what changed and how. -->

-

## Testing

<!-- The commands you ran and their result. -->

- [ ] `cargo test --manifest-path cli/Cargo.toml` passes
- [ ] `cargo fmt --manifest-path cli/Cargo.toml --all -- --check` passes
- [ ] `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` passes

## Risk / impact

<!-- Reversibility, schema/contract impact, blast radius. 1-2 lines. -->

## Checklist

- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] I edited the engine **source** (`engine/`), not the vendored copy (`.mochiflow/engine/`)
- [ ] Generated adapters were regenerated via `mochiflow adapter generate` (not hand-edited), if templates changed
- [ ] If a schema or other locked file changed: `contracts.lock` was regenerated **and** `engine/VERSION` was bumped **in this PR**
- [ ] Docs updated (including **both** `README.md` and `README.ja.md` if user-facing)
- [ ] No secrets (`.env`, credentials) are included
