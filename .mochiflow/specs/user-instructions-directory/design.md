# Add user-owned MochiFlow instructions directories — Design

## Design Decisions

- Use fixed install-relative directory names, `instructions/` and
  `instructions.local/`, rather than config fields. They are convenience homes
  for user files, not MochiFlow-owned resources.
- Keep the seeded README as a small English constant in the init implementation.
  Write it only when absent; `--force` applies to managed config/engine/adapter
  artifacts and never grants permission to replace user instructions.
- Treat directory creation as idempotent filesystem scaffolding. A path-type
  collision is an init error; no cleanup or replacement is attempted.
- Extend install-level ignore handling so a non-forced init appends a missing
  `instructions.local/` rule without replacing custom content. Forced init
  regenerates the managed default ignore file with that rule. Never emit an
  ignore rule for `instructions/`.
- Do not add directory accessors to `Config`. Keeping the paths out of config
  prevents doctor, join, upgrade, adapter generation, and drift code from
  acquiring accidental ownership.
- Represent purge data-loss guidance in the detach report so text and JSON
  callers receive the same explicit warning. Also emit the path-specific
  warning to stderr before adapter cleanup or install-directory removal; this
  preserves a single JSON document on stdout. The interactive confirmation
  prompt uses the same wording and the existing confirmation phrase remains
  unchanged.
- Keep the local release migration outside Git. The tracked product change to
  `.mochiflow/.gitignore` makes the new local path ignored before the release
  file is created.

## Architecture

The init path owns the only creation behavior:

1. Ensure the install directory and install-level ignore policy.
2. Load or create config and install the engine as today.
3. Create `instructions/` and `instructions.local/` alongside existing
   scaffolding.
4. Create `instructions/README.md` only when missing and always add one
   localized init-summary item naming both the shared and local paths, whether
   the README was created or preserved. Dry-run and JSON presentations carry
   the same discoverability contract.
5. Continue existing adapter generation and doctor checks without passing the
   new directories into either subsystem.

Detach continues to remove adapter integration first. Normal mode removes
runtime state and reports the instruction directories as kept when present.
Purge mode attaches a path-specific warning to its report and emits that warning
to stderr before any destructive action. It then removes the whole install
directory only after the existing confirmation succeeds. Human output may
repeat the warning in the final report; JSON stdout remains one document while
stderr carries the pre-action warning.

No component scans inside either directory. Git alone determines whether files
under `instructions/` are tracked; the install-level ignore rule determines the
default privacy of `instructions.local/`.

## Data Model / Interfaces

- New filesystem contract:
  - `.mochiflow/instructions/README.md`: created once, then user-owned.
  - `.mochiflow/instructions/**/*.md`: opaque, shareable user files.
  - `.mochiflow/instructions.local/**/*.md`: opaque, ignored local user files.
- Existing `.mochiflow/.gitignore` contract gains the exact line
  `instructions.local/`; `instructions/` remains absent.
- Detach's serialized report gains a warnings collection containing the
  instruction-path purge warning when mode is purge. This is an additive JSON
  field; existing fields and the confirmation phrase remain stable. The same
  warning is written to stderr before cleanup/removal so report serialization
  does not delay the only warning until after deletion.
- There is no config schema, manifest, frontmatter, index, or adapter interface.

## Error Handling

- Any directory-creation or README-write failure exits init with the existing
  `FAIL: could not ...` style and the concrete path/error.
- Existing instruction files are never opened for write.
- Existing `.mochiflow/.gitignore` read/write failures fail init because the
  local privacy rule cannot be guaranteed.
- A missing instruction directory is never a diagnostic error outside init.
- Purge confirmation failure remains non-destructive and includes the explicit
  warning in its report. Confirmed purge emits the warning before cleanup and
  reports it afterward without writing non-JSON text to JSON stdout.

## Test Strategy

- Extend CLI integration fixtures to cover fresh scaffolding, README content,
  repeated/forced preservation, custom ignore merging, Git visibility, path
  collisions, normal detach preservation, and purge warnings/deletion.
- Extend init presentation/golden coverage so fresh, repeated, dry-run, and JSON
  init paths all retain the discoverability message.
- Add negative regression assertions that join/upgrade do not backfill and that
  generated/standing contracts do not mention the new directories.
- Run the configured CLI default profile, spec lint, doctor, and adapter
  generation check. Engine source is not edited, so freeze remains a check-only
  integrity assertion.

## Integration Contract

- Owner: `mochiflow init` owns one-time scaffolding and install-level ignore
  safety; users own every Markdown file after creation.
- Request: an explicit init invocation may create missing directories/README
  and append the missing local ignore rule.
- Response: every successful or dry-run init output names the two paths and
  their shared/local semantics, whether the README is created or preserved.
- Compatibility: join and upgrade remain repair/update commands and do not
  create new tracked candidates; adapters, router, constitution, doctor,
  freeze, and drift behavior stay unchanged.
- Failure handling: init fails before claiming success when it cannot create
  the scaffold or guarantee the local ignore rule; existing user files remain
  intact.
- Deletion: normal detach preserves the directories; purge keeps its exact
  confirmation gate, warns on stderr before any removal, and includes the same
  warning in text/JSON reports.
- Verification: CLI fixtures prove filesystem, ignore, command-boundary, and
  detach behavior; documentation/content assertions prove discoverability and
  standing-contract isolation.

## Review Results

Review profile: plan-auditor
Reviewer mode: inline
Verdict: pass-with-comments

The bounded review-fix round clarified unconditional init discoverability,
pre-removal purge-warning transport, and executable separation between fixture
command tests and source-repository integrity checks.
