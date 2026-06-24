# Verify profile should cover CI lint checks — Design

## Design Decisions

- **`default` becomes the build-completion profile**: keep `mochiflow ready` and
  build flow behavior centered on `default` rather than introducing a separate
  required profile. This matches existing CLI behavior and avoids a split where
  `ready` passes but CI-equivalent verification is still optional.

- **`quick` is optional fast feedback**: add `quick` for the previous test-only
  command so agents can run narrow feedback when appropriate, while plan/build
  completion still records `default`.

- **`cargo-deny` remains CI-only**: CI uses `EmbarkStudios/cargo-deny-action`,
  while the checked-in Rust toolchain only guarantees `rustfmt` and `clippy`.
  Keeping `cargo-deny` out of `default` avoids making normal build work depend
  on a tool this repository does not locally provision.

- **Engine guidance owns semantics, project config owns commands**: update
  `engine/reference/workflow.md` and `engine/commands/build.md` to define how
  agents should choose profiles, while `.mochiflow/config.toml` holds this
  repository's concrete commands.

- **Explicit config write exception**: this spec permits build to edit
  `.mochiflow/config.toml` only for the `surfaces.cli.verify` profile change,
  even though that file is outside the current `[write].allow` globs. Without
  this exception, the approved ACs would require a file the build procedure
  cannot write.

- **Context refresh output is preserved**: the `.mochiflow/context/` changes
  already present on the branch came from `refresh-context` output and are kept
  as related context refresh output, not hand-authored build edits.

## Architecture

| File | Change |
| --- | --- |
| `.mochiflow/config.toml` | Change `surfaces.cli.verify.default` to the chained local CI-equivalent command; add `quick` with the existing `cargo test` command |
| `.mochiflow/context/**` | Preserve already refreshed context output that was present when this work started |
| `engine/reference/workflow.md` | Clarify that `default` is canonical build/merge-equivalent verification and `quick` is fast feedback |
| `engine/commands/build.md` | Clarify that build runs the canonical `default` profile for completion, not a narrower fast profile |
| `engine/MANIFEST.json` | Regenerate after engine source edits |
| `.mochiflow/engine/**` | Sync vendored engine from source after engine edits |
| Generated adapters | Check with `mochiflow adapter generate --check`; regenerate only if template output changes |

## Data Model / Interfaces

- No schema change is required. `RawSurface.verify` already accepts arbitrary
  named profiles as `BTreeMap<String, String>`, and `Config::verify_command`
  already falls back to `default` when a requested profile is absent.
- `mochiflow ready` continues to check `default`, preserving the existing
  lifecycle contract.

## Error Handling

- The `default` command must use failure-propagating shell chaining so a failed
  test, format, clippy, or freeze step returns non-zero.
- If `cargo fmt`, `cargo clippy`, or `freeze --check` fails, the build remains
  incomplete until the failure is fixed and the command passes.
- If `cargo-deny` fails in CI after local verification passes, treat it as a
  CI-only dependency-audit failure and route through the PR feedback loop; do
  not claim local `default` covered that check.

## Test Strategy

- Run `mochiflow lint --spec verify-ci-parity` after authoring and after
  approval.
- Run `mochiflow config show` to verify the resolved `cli.default` and
  `cli.quick` commands.
- Run the new `cli.default` command directly from the repository root.
- Because engine source changes are expected, run `mochiflow freeze`,
  `mochiflow upgrade --source engine`, and `mochiflow adapter generate --check`.
- Run `cargo test --manifest-path cli/Cargo.toml` as the Rust regression suite.

## Integration Contract

- Contract owner: MochiFlow workflow documentation and project configuration.
- Request: build flow asks for the canonical verification command for a surface.
- Response: the `default` profile is the command whose success is sufficient for
  local build completion, excluding explicitly documented CI-only checks.
- Compatibility: existing configs with only `default` remain valid; `quick` is
  additive and optional.
- Failure handling: any non-zero subcommand in `default` fails verification and
  blocks build completion.
- Verification: AC Matrix rows AC-01 through AC-06 record command output and
  file evidence.

## Review Results

- Reviewer mode: delegated
- Verdict: pass-with-comments
- Findings: 1 Low spec-conformance note: AC evidence for lint should be refreshed after recording this review and checking T-004, because the Matrix edit temporarily made lint report the dirty unchecked T-004 warning. No code quality findings.
