# Add user-owned MochiFlow instructions directories — Tasks

Implementation Summary: Add init-owned one-time scaffolding for opaque user Markdown, preserve it across normal detach, and document its explicit-path contract.
risk: elevated
Critical Stop Conditions:
- Stop if implementation requires a config, adapter, router, constitution, index, or drift contract.
- Stop if preserving local privacy would require overwriting an existing user instruction or custom ignore file.
- Stop if purge semantics or its exact confirmation phrase would need to change beyond adding the explicit warning.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-01, AC-02, AC-03, AC-04] Add idempotent init scaffolding and ignore safety
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-cli/src/main.rs`
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/first_run.rs`
    - `cli/crates/mochiflow-cli/tests/golden/init_npm.json`
    - `.mochiflow/.gitignore`
  - Done: Fresh init creates both directories and the explanatory README; repeated and forced init preserve user files; every fresh, repeated, dry-run, and JSON init presentation names both paths; existing custom ignore content is retained while the local rule is added exactly once; Git exposes shared instructions and ignores local instructions; join/upgrade and non-init subsystems remain outside the creation path. The shared CLI test file is left with init-focused fixtures passing and no detach assumptions changed.
  - Stop: The scaffold cannot remain fixed-path and config-free, or ignore safety cannot be added without replacing custom content.
- [x] T-002 [AC-05] Preserve user instructions across detach and make purge loss explicit
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/detach.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: Normal detach reports and retains both instruction directories; every purge path explicitly identifies both user-owned paths on stderr before removal and in its text/JSON report; JSON stdout remains one valid document; unconfirmed purge is non-destructive; confirmed purge still removes the full install tree using the unchanged phrase. The shared CLI test file retains all T-001 init fixtures and adds isolated detach/purge coverage.
  - Stop: The warning would require weakening confirmation, changing the phrase, or emitting invalid mixed JSON output.
- [x] T-003 [AC-04, AC-06, AC-07] Document the ownership contract and migrate local release guidance
  - Depends on: T-001, T-002
  - Files:
    - `README.md`
    - `README.ja.md`
    - `docs/getting-started.md`
    - `docs/configuration.md`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `.mochiflow/constitution.local.md`
    - `.mochiflow/instructions.local/release.md`
  - Done: English/Japanese docs describe shareable versus local-only ownership, explicit path use, no auto-loading/management, detach preservation, and purge deletion; conformance coverage prevents adapter/router/constitution mentions; the ignored release procedure is moved and the local constitution is restored to its stub. Only tracked product/docs/test paths are committed; both local migration paths remain unstaged.
  - Stop: Documentation would imply automatic discovery/loading, or the local migration would become tracked PR content.
