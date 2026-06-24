# Verify profile should cover CI lint checks

## Problem

The `cli` surface's configured `default` verification command currently runs
only `cargo test --manifest-path cli/Cargo.toml`, while the repository's GitHub
Actions workflow also runs formatting, clippy, engine freeze, and cargo-deny
checks. The build procedure and `mochiflow ready` both treat the surface
`default` command as the canonical local verification signal, so work can pass
locally and still fail CI for checks the agent was never asked to run.

## Appetite

This is worth a small workflow/configuration fix plus targeted documentation and
tests. It should not become a broader redesign of verification profiles, CI
orchestration, or package management.

## Solution

Make local build verification cover the merge-blocking checks that are practical
and toolchain-backed in this repository. The agreed shape is to make the `cli`
surface's `default` verification profile run `cargo test`, `cargo fmt --check`,
`cargo clippy -D warnings`, and `freeze --check`, because current readiness and
build flow already validate and run `default`. A faster profile can exist as an
optional `quick` command, but it must not be the signal that build completion
depends on.

Update the workflow guidance so `default` is understood as the reliable
merge/CI-equivalent profile for a surface, not merely a unit-test command. Keep
the actual command list aligned with `.github/workflows/ci.yml` and the PR
template where local tool availability makes that reasonable. `cargo-deny`
remains a CI-only expectation unless the plan finds a standard way to guarantee
that tool locally.

## Rabbit Holes

- Do not add a `ci` profile alone unless `mochiflow ready` and the build
  procedure are also changed to require that profile; otherwise agents will
  still pass build using `default`.
- Do not turn MochiFlow into a CI runner. The goal is parity with this repo's
  own configured checks, not remote CI emulation.
- Keep checks that require nonstandard local tooling, such as `cargo-deny`, out
  of `default` unless the plan finds a standard way to guarantee that tool
  locally.

## No-gos

- Do not rely on docs-only guidance while leaving `.mochiflow/config.toml`
  narrower than CI.
- Do not weaken CI, remove checks, or make PR feedback rely on remote failures
  for issues that can be caught locally.
- Do not hand-edit generated adapter outputs as part of this work.

## Alternatives Considered

- Add only a `ci` profile: rejected because the current build path uses
  `default`, so this would not close the local verification gap by itself.
- Update only `commands/build.md`: rejected because the active project
  configuration would still tell agents to run only `cargo test`.
- Keep `default` as tests and expect humans to run CI commands separately:
  rejected because it preserves the round-trip this work is meant to remove.

## Open Questions

- No open questions remain; ready for plan.
