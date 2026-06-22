# Version SSOT + freeze command — Tasks

Implementation Summary: Make cli/Cargo.toml workspace version the single source of truth and add `mochiflow freeze` (+ `--check`) that regenerates engine/VERSION, engine/MANIFEST.json, and contracts/contracts.lock from one shared hash implementation.
risk: elevated
Critical Stop Conditions:
- The frozen-surface hash input set changes (must stay schemas + golden only).
- Removing the version-gate escape hatch makes an existing legitimate state fail — stop and re-evaluate the gate rule.
- `freeze` would write a partial/empty lock because a frozen surface is missing.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [x] T-001 [AC-06] Extract pure builders into mochiflow-core (hash, version reader, manifest builder, repo-root)
  - Depends on: none
  - Files: `cli/crates/mochiflow-core/src/freeze.rs`, `cli/crates/mochiflow-core/src/lib.rs`, `cli/crates/mochiflow-core/src/upgrade.rs`
  - Done: New `freeze` module exposes pure (write-free) functions: `compute_contracts_hash(repo_root)` (same input set: `contracts/*.json` sorted, then `tests/conformance/golden/**` sorted), `read_workspace_version(repo_root)` parsing `cli/Cargo.toml` `[workspace.package].version`, `build_manifest(engine_dir, version) -> String` (takes version as an argument, performs no write), and `resolve_repo_root(cwd)` (walk up to the first ancestor containing both `cli/Cargo.toml` and `engine/VERSION`; error if none). `upgrade::write_manifest_for_engine_dir` is refactored to call `build_manifest` with no behavior change. Unit tests (read-only against the real tree for the readers; non-repo case for `resolve_repo_root`) cover each; `cargo test` passes.
  - Stop: hash input set would need to change to make tests pass; or refactoring `write_manifest_for_engine_dir` changes its output bytes.

- [x] T-002 [AC-02] [AC-03] [AC-04] [AC-05] Implement freeze core (write + check, idempotent)
  - Depends on: T-001
  - Files: `cli/crates/mochiflow-core/src/freeze.rs`
  - Done: `freeze(repo_root, check)` computes ALL desired content first — `engine/VERSION`, `engine/MANIFEST.json` via `build_manifest(engine_dir, workspace_version)` (NOT `write_manifest_for_engine_dir` directly, so the manifest version comes from the SSOT not on-disk `engine/VERSION`), and `contracts/contracts.lock` from the workspace version + frozen-surface hash — then in write mode writes only differing files, and in check mode returns stale entries without writing. Idempotent (second run on a clean fixture = no change). Unit tests run against a `tempfile` repo fixture (never the real tree) and cover write, idempotency, and staleness detection.
  - Stop: a frozen surface (`contracts/` or `tests/conformance/golden/`) is missing — refuse partial write; or a test would mutate the real repo tree.

- [x] T-003 [AC-01] [AC-07] Wire `mochiflow freeze` into the CLI and remove `engine manifest`
  - Depends on: T-002
  - Files: `cli/crates/mochiflow-cli/src/main.rs`
  - Done: `Commands::Freeze { check: bool }` added and routed to `freeze::freeze`, resolving repo root from cwd via `resolve_repo_root` (not `--config`); `Commands::Engine` + `EngineCommand` removed; `freeze` and `freeze --check` run end-to-end; CLI tests pin that freeze run from a fixture subdir succeeds and from a non-repo temp dir fails non-zero without writing; `cargo test` passes.
  - Stop: removing `engine` breaks another caller of `EngineCommand`.

- [x] T-004 [AC-08] Collapse the version gate to a single rule
  - Depends on: T-001
  - Files: `cli/crates/mochiflow-cli/tests/conformance.rs`, `cli/crates/mochiflow-core/src/freeze.rs`
  - Done: `version_gate_consistent` rewritten to assert `lock.hash == compute_contracts_hash()` AND `lock.version == workspace_version == engine/VERSION`, calling the core hash fn (passing `repo_root()` in); the `|| engine_version != lock_version` escape clause is deleted; `version_gate_hash_matches_committed_lock` calls the core fn. A negative-case test (in `freeze.rs` tests, against a `tempfile` fixture) asserts that a fixture with a mismatched version triple FAILS the gate — proving the escape-hatch removal is positively verified, not just happy-path. `cargo test` passes.
  - Stop: the strict rule fails on the current committed tree (would indicate real drift to fix first).

- [x] T-005 [P] [AC-09] Replace the literal version test with a dynamic one
  - Depends on: none
  - Files: `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: `version_is_1_1_3` replaced by a test asserting `--version` output equals `format!("mochiflow {}", env!("CARGO_PKG_VERSION"))`; `cargo test` passes.
  - Stop: none beyond shared.

- [x] T-006 [AC-10] Add `freeze --check` to CI
  - Depends on: T-003
  - Files: `.github/workflows/ci.yml`
  - Done: The `rust` job runs `cargo run --manifest-path cli/Cargo.toml -- freeze --check` and fails on staleness; step ordering keeps existing test/fmt/clippy steps intact.
  - Stop: CI step requires infra not available in the runner.

- [x] T-007 [AC-11] Update contributor docs and remove the drift warning
  - Depends on: T-003
  - Files: `contracts/VERSIONING.md`, `docs/versioning.md`, `CONTRIBUTING.md`, `.kiro/steering/release.md`, `.github/PULL_REQUEST_TEMPLATE.md`
  - Done: Every doc that currently states `engine/VERSION` is the version source of record is updated to the new SSOT (`cli/Cargo.toml` `[workspace.package].version`) + `mochiflow freeze` two-step (bump → freeze → commit). Confirmed old-SSOT prose to fix: `CONTRIBUTING.md:63` ("bump `engine/VERSION`"); `CONTRIBUTING.md:44` (regenerate via `mochiflow engine manifest` — a removed command, must drop / point to `mochiflow freeze`); `docs/versioning.md:13` ("`engine/VERSION` is the source ... semver"); `.kiro/steering/release.md` lines 18/30/34/40-42/85/103 (multi-file bump steps, "release version of record", and the "Known drift to watch for" warning, which is removed); `.github/PULL_REQUEST_TEMPLATE.md:30` ("`contracts.lock` was regenerated **and** `engine/VERSION` was bumped" → becomes "bumped `cli/Cargo.toml` version and ran `mochiflow freeze`"); `contracts/VERSIONING.md` contract-change protocol (regenerate via `mochiflow freeze`, not a test-only `compute_contracts_hash()`).
  - Stop: a referenced doc no longer matches the quoted line — re-grep `engine/VERSION` and `engine manifest` before editing.

- [x] T-008 [chore: full gate] Run the complete verification gate and self-freeze
  - Depends on: T-001, T-002, T-003, T-004, T-005, T-006, T-007
  - Files: (verification only)
  - Done: `cargo test --manifest-path cli/Cargo.toml`, `cargo fmt --manifest-path cli/Cargo.toml --all -- --check`, `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`, and `cargo run --manifest-path cli/Cargo.toml -- freeze --check` all pass on the final tree.
  - Stop: any gate fails and the fix is out of scope.
