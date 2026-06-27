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
  verification, updates done metadata, moves the spec, regenerates the index,
  stages allowed lifecycle paths, validates the staged result, and creates the
  close-out commit.
- Keep human judgment outside the command. Human-operated QA, visual
  confirmation, and durable learning content must already be recorded in the
  durable artifacts before `mochiflow ship` proceeds.
- Use Git's standard index workflow: `git add -A` constrained by configured
  lifecycle pathspecs, `git status --porcelain` for script-readable dirt checks,
  and `git diff --cached --name-status` for staged result validation. Sources:
  Git `add` documentation (https://git-scm.com/docs/git-add), Git `status`
  documentation (https://git-scm.com/docs/git-status), Git `diff`
  documentation (https://git-scm.com/docs/git-diff).
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
  - all matrix rows must already be ship-eligible or become ship-eligible after
    the command's final verification update according to the implemented
    command contract;
  - required elevated-risk review results must be present before `done`.
- Staging allowlist:
  - configured `{specs_dir}` parent;
  - configured `{index}`;
  - configured ADR files;
  - no `state/`, source files, unrelated active specs, or unrelated archived
    specs.
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

## Test Strategy

- Add CLI integration tests that build temporary MochiFlow projects with Git
  initialized, local user identity configured, and verify commands set to stable
  shell fixtures.
- Cover the happy path from approved active spec to committed archived done spec.
- Cover dirty working tree, pre-staged unrelated files, failed verification,
  missing reviewer verdict, and non-default `specs_dir`.
- Cover retry states, including already moved to `_done` before commit.
- Extend PR tests so `mochiflow pr --spec <slug> --dry-run` refuses an unshipped
  spec and accepts a committed done spec without requiring network access.
- Add conformance checks for engine guidance so the manual fallback uses the
  configured lifecycle parent pathspec instead of the moved-from slug path.

## Review Results

Mandatory elevated-risk review will be recorded during build before ship
acceptance.
