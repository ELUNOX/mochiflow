# Make ship archive staging resilient to moved spec paths — Design

## Design Decisions

- Add `mochiflow ship [slug]` as the public command. The command name follows
  MochiFlow's existing workflow vocabulary and avoids exposing implementation
  details such as `stage`, `close-out`, or `finalize`.
- Make the slug positional and optional. clap's derive API supports positional
  arguments and optional fields; this matches the existing `ready <spec>` shape
  while still allowing `mochiflow ship` to infer the target from the current
  branch when the branch convention is unambiguous. Source: clap derive tutorial
  (https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html).
- Treat `mochiflow ship` as the CLI owner of automatable ship close-out, not as
  a thin staging helper. The command validates readiness, runs final
  verification, updates eligible automated AC Matrix rows, updates done
  metadata, moves the spec, regenerates the index, stages allowed lifecycle
  paths, validates the staged result, and creates the close-out commit.
- Keep human judgment outside the command. Human-operated QA, visual
  confirmation, and durable learning content must already be recorded in the
  durable artifacts before `mochiflow ship` proceeds.
- Use Git's standard index workflow with delimiter-safe parsing: `git add -A`
  constrained by configured lifecycle pathspecs, `git status --porcelain=v1 -z`
  for script-readable dirt checks, and `git diff --cached --name-status -z` for
  staged result validation. Sources: Git `add` documentation
  (https://git-scm.com/docs/git-add), Git `status` documentation
  (https://git-scm.com/docs/git-status), Git `diff` documentation
  (https://git-scm.com/docs/git-diff).
- Do not add a time dependency for completion timestamps. Use the Rust standard
  library or a small local formatter for UTC timestamps to avoid expanding the
  dependency set solely for this command.

## Architecture

- Add a new core module, `cli/crates/mochiflow-core/src/ship.rs`, with a
  `run_ship(cfg, slug_arg, dry_run)` entry point returning an exit code.
- Add a top-level `Ship` variant to `cli/crates/mochiflow-cli/src/main.rs`, with
  `slug: Option<String>` and `--dry-run`.
- Reuse existing modules where possible:
  - `spec_meta` to read target metadata;
  - `lint::run_lint` for acceptance-state validation after setting `done`;
  - `index::generate_index` for the generated index;
  - `Config` path helpers for specs, index, ADR, state, and verify commands.
- Share Git helper behavior between `ship` and `pr` where practical, either by
  moving the local `pr.rs` Git helpers into a small internal module or by adding
  focused helpers in `ship.rs` without widening public API.
- Update `doctor::TERMINAL_CLI_COMMAND_REFERENCES` so command-reference checks
  and generated docs recognize `ship` as a terminal CLI command.

## Data Model / Interfaces

- CLI:
  - `mochiflow ship`
  - `mochiflow ship <slug>`
  - `mochiflow ship <slug> --dry-run`
- Target resolution:
  - explicit slug resolves active `{specs_dir}/{slug}/` first, then
    `_done/{slug}/` only for interrupted/retry diagnostics;
  - omitted slug resolves from the current branch by stripping a known type
    prefix (`feat`, `fix`, `refactor`, `docs`, `chore`) and matching the
    resulting slug to exactly one active spec.
- Readiness:
  - target must be `approved` before mutation;
  - `mochiflow lint --spec <slug>` must pass before mutation;
  - human-operated, visual, `CONFIRMED`, `N/A: <reason>`, and existing `PASS`
    matrix rows must already be done-eligible before mutation;
  - `FAIL`, `PENDING_HUMAN`, and non-automated `UNVERIFIED` rows stop the
    command before mutation;
  - automated rows may be `UNVERIFIED` before mutation only when their scope
    maps to a declared surface that has a runnable final verification command;
  - after all final verification commands pass, the command updates eligible
    automated `UNVERIFIED` rows to `PASS` and records evidence identifying the
    surface and command that passed;
  - required elevated-risk review results must be present before `done`.
- Dry-run:
  - resolves the target and prints readiness blockers, planned verification
    commands, lifecycle paths, and close-out actions;
  - does not run verification, edit spec files, regenerate index files, stage,
    or commit.
- Generated index outputs:
  - `index::generate_index` updates the configured human-readable `{index}` and
    runtime `{install_dir}/state/index.json`;
  - `{index}` is staged as a lifecycle artifact;
  - `state/index.json` is runtime state and is never staged, but it may exist or
    change when the configured state directory is gitignored;
  - if runtime state is not ignored, `mochiflow ship` stops with the same class
    of hygiene failure expected before PR handoff.
- Staging allowlist:
  - configured `{specs_dir}` parent;
  - configured `{index}`;
  - configured ADR files;
  - configured ignored runtime state may be dirty but not staged;
  - no source files, unrelated active specs, unrelated archived specs, or
    unignored `state/` files.
- Commit:
  - one Conventional Commit close-out commit;
  - include `Spec: <slug>` trailer;
  - avoid AC IDs and internal workflow vocabulary in the subject.

## Error Handling

- Pre-mutation failures return a non-zero exit code and leave lifecycle files
  unchanged.
- Verification failure returns a non-zero exit code and leaves lifecycle files
  unchanged.
- Interrupted lifecycle states are detected explicitly:
  - active only: proceed normally;
  - archived only and uncommitted: validate and resume staging/commit;
  - both active and archived: stop with a recovery message;
  - neither: stop as missing target.
- Any pre-existing unrelated working tree or staged change stops the command
  before mutation.
- Any unexpected staged path after ship staging stops before commit, preserving
  the user's index for inspection.
- `mochiflow pr --spec <slug>` pre-flight should fail before push when the slug
  still has an active approved spec or lacks a committed done spec under
  `_done/{slug}`.
- `mochiflow pr --spec <value>` keeps the existing resolver split:
  - bare tokens are treated as spec slugs, resolve delivery artifacts under
    `{install_dir}/state/{slug}`, and run the slug-aware ship-complete guard;
  - path-like values, including absolute paths or values containing a path
    separator, remain explicit request directories and do not run the slug guard.

## Test Strategy

- Add CLI integration tests that build temporary MochiFlow projects with Git
  initialized, local user identity configured, and verify commands set to stable
  shell fixtures.
- Cover the happy path from approved active spec to committed archived done spec.
- Cover AC Matrix updates for eligible automated rows and precondition stops for
  `FAIL`, `PENDING_HUMAN`, and non-automated `UNVERIFIED` rows.
- Cover dirty working tree, pre-staged unrelated files, failed verification,
  missing verify profile, TODO placeholder verification, missing reviewer
  verdict, and non-default `specs_dir`.
- Cover retry states: active-only, archived-only uncommitted, both active and
  archived, neither present, already done, and partially staged.
- Extend PR tests so non-dry-run `mochiflow pr --spec <slug>` refuses an
  unshipped spec before push and accepts a committed done spec on the manual
  handoff path without requiring network access. Preserve the existing
  path-like request-dir behavior.
- Add conformance checks for engine guidance so the manual fallback uses the
  configured lifecycle parent pathspec instead of the moved-from slug path.
- Add path parsing tests with spaces or shell-special characters to prove the
  NUL-delimited Git helpers do not split paths incorrectly.

## Review Results

Mandatory elevated-risk review will be recorded during build before ship
acceptance.
