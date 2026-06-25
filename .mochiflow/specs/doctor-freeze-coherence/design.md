# Clarify doctor/freeze coherence and context freshness — Design

## Design Decisions

- Keep `doctor` and `freeze` as separate checks. `doctor engine` remains an
  installed/vendored engine MANIFEST integrity check. `freeze --check` remains a
  source-repo derived-file check. The CLI should guide users to run both in the
  source repo, but `doctor` must not internally execute `freeze --check`.
- Implement context freshness as a narrow warning in `doctor config`. The check
  scans configured context files for `mochiflow <command>` references and warns
  when `<command>` is not in the public command-reference allowlist.
  Missing/stub context behavior remains unchanged.
- Define public command references as the union of terminal CLI subcommands and
  agent workflow vocabulary. Terminal CLI subcommands are `config`, `index`,
  `lint`, `doctor`, `adapter`, `upgrade`, `ready`, `backlog`, `init`, `join`,
  `detach`, `guide`, `completions`, `pr`, and `freeze`. Agent workflow
  vocabulary is `discuss`, `plan`, `build`, `ship`, `patch`, `review`,
  `refresh-context`, and `onboard`. This prevents legitimate prose such as
  `mochiflow discuss` from being classified as stale.
- Guard the hard-coded doctor allowlist with a test that compares the CLI
  subcommand portion to clap's actual top-level subcommands. Extra workflow
  vocabulary remains intentionally allowed and documented.
- Add `--root` as a `freeze` command option and keep the existing cwd-upward
  fallback when omitted. This follows the existing clap derive style in
  `main.rs` and the clap 4 derive documentation for long options and optional
  fields: https://docs.rs/clap/4.6.1/clap/_derive/_tutorial/index.html
- Do not infer `freeze` root from global `--config`. The global config path
  identifies installed project state, while `freeze` targets the MochiFlow
  source tree and its derived files.

## Architecture

- CLI parsing stays in `cli/crates/mochiflow-cli/src/main.rs`.
  `Commands::Freeze` gains an optional `root` field.
- Source-root validation stays in `cli/crates/mochiflow-core/src/freeze.rs`.
  Add a helper that validates an explicit root without walking upward, while
  preserving `resolve_repo_root(cwd)` for the no-root path.
- Doctor checks stay in `cli/crates/mochiflow-core/src/doctor.rs`.
  `validate_config` continues to own config/context warnings. It should call
  small helper functions for source-repo guidance and stale command references
  so tests can cover them without changing the public JSON shape.
- Source-repo detection for doctor guidance uses the same marker rule as
  explicit freeze-root validation against `cfg.repo_root`: `cli/Cargo.toml` and
  `engine/VERSION` must both exist at that exact root. It must not walk upward.
- Tests remain in `cli/crates/mochiflow-cli/tests/cli.rs` for end-to-end CLI
  behavior and, where useful, in `freeze.rs` unit tests for pure root-resolution
  helpers. A `main.rs` unit test may be used to compare clap's actual
  top-level subcommands against the doctor allowlist because `Cli` is private to
  the binary.
- Documentation updates live in `README.md`, `docs/versioning.md`, and
  `contracts/VERSIONING.md`, limited to `doctor` versus `freeze --check`
  guidance and `freeze --root` usage. Do not duplicate existing versioning
  explanations that already describe consumer drift detection.

## Data Model / Interfaces

- CLI interface addition:
  - `mochiflow freeze --root <source-repo>`
  - `mochiflow freeze --root <source-repo> --check`
- No `config.toml`, `spec.yaml`, schema, or contracts.lock format changes.
- Human output additions:
  - `doctor config`/full `doctor` may include a WARN advising source-repo users
    to run `mochiflow freeze --check`.
  - `doctor config`/full `doctor` may include a WARN identifying stale context
    command references and suggesting `refresh-context`.
- JSON output keeps the same top-level shape. New WARN issues appear in
  `checks.config`; `total_fail` remains unchanged for warning-only findings.
- The command-reference allowlist should be exposed through helper functions
  that tests can inspect separately as terminal CLI commands and workflow
  vocabulary. The terminal CLI command helper is the one compared against clap.

## Error Handling

- `freeze --root <path>` validates the supplied path directly. If it is not a
  source repo with `cli/Cargo.toml` and `engine/VERSION`, return the existing
  source-repo failure style and do not call the `freeze` writer. Do not reuse
  `resolve_repo_root(root)` for explicit root validation because walking upward
  could silently select an ancestor source repo.
- Context file read failures keep the current missing-file WARN behavior.
- Context command scanning must ignore malformed or partial text that does not
  match `mochiflow <command>`.
- Unknown context commands are WARN, not FAIL, because context freshness is
  advisory and refreshed by the agent.

## Test Strategy

- Add CLI tests that materialize a project whose context references a removed
  command, then assert `doctor config` reports a WARN and exits successfully.
- Add no-WARN tests for context that references a current terminal CLI command,
  a current workflow command, and no command references.
- Add an allowlist drift test that compares doctor.rs's terminal CLI command
  allowlist to clap's top-level subcommands.
- Add a source-repo doctor guidance test that runs against this repository's
  config and verifies guidance appears without converting to FAIL. The source
  repo detector must inspect `cfg.repo_root` directly for `cli/Cargo.toml` and
  `engine/VERSION`.
- Add freeze CLI tests for explicit valid root, explicit invalid root, and
  no-root cwd fallback.
- Add documentation assertion tests for the required doctor/freeze and
  `freeze --root` phrases, or extend existing documentation tests if a nearby
  assertion already covers public docs.
- Keep `cli.default` as the final verification: `cargo test --manifest-path
  cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check &&
  cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings &&
  cargo run --manifest-path cli/Cargo.toml -- freeze --check`.

## Workstreams

| Workstream | Surface | Responsibility | Depends on | Verification |
| --- | --- | --- | --- | --- |
| Doctor diagnostics | cli | Source-repo guidance and context command freshness WARNs | none | CLI tests for WARN/no-WARN behavior |
| Freeze root option | cli | `--root` parsing and explicit source-root validation | none | CLI/unit tests for valid, invalid, and fallback root resolution |
| Documentation | cli | User-facing distinction between `doctor` and `freeze --check` | Doctor diagnostics, Freeze root option | Documentation assertions or reviewed diff plus full verification |

## Integration Contract

- Contract owner: MochiFlow CLI command-line workflow.
- Request: Users run `mochiflow doctor`, `mochiflow doctor config`, or
  `mochiflow freeze [--root <source-repo>] [--check]`.
- Response: `doctor` reports advisory WARN issues without changing exit code
  unless FAIL issues already exist. `freeze` uses the explicit root when
  provided and otherwise preserves cwd-upward behavior.
- Compatibility: Existing `mochiflow freeze [--check]`, `doctor`, and
  `doctor --json` invocations remain valid. JSON shape does not change.
- Failure handling: invalid `--root` fails before writes. Context freshness
  findings never write files and never become FAIL.
- Verification: CLI integration tests cover command output, exit codes, and
  backward compatibility.

## Review Results

- No reviewer run has occurred during plan. Because risk is `elevated`, build
  completion requires one independent-reviewer result after all implementation
  tasks.
