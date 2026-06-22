# Version SSOT + freeze command — Design

## Design Decisions

- **SSOT = `cli/Cargo.toml` `[workspace.package].version`.** Every other version
  occurrence is derived from or validated against it. `--version` already derives
  from it via clap `CARGO_PKG_VERSION`; `engine/VERSION`,
  `engine/MANIFEST.json` `version`, and `contracts/contracts.lock` `version`
  become freeze-written derivations.
- **`freeze` reads `cli/Cargo.toml` at runtime**, not `env!("CARGO_PKG_VERSION")`.
  Reason: freeze is a repo-tree integrity tool; it must reflect the version
  declared in source, independent of the binary that runs it. The `toml` crate
  (with `preserve_order`) is already a dependency.
- **One command writes all derived files.** `freeze` writes, in one invocation:
  (1) `engine/VERSION` ← workspace version; (2) `engine/MANIFEST.json` ← full
  engine file-hash map + `version`; (3) `contracts/contracts.lock` ←
  `{version: workspace_version, hash: frozen_surface_hash}`. No partial
  regeneration subcommands.
- **`mochiflow engine manifest` is removed.** Its only job (regenerate
  `engine/MANIFEST.json`) is a strict subset of freeze. Removing it eliminates
  the "which command do I run?" ambiguity. The `EngineCommand` enum has only this
  one variant today, so the hidden `engine` parent command is removed entirely.
- **Hash logic has one home.** `compute_contracts_hash()` moves from
  `conformance.rs` (test-only) into `mochiflow-core` (e.g. `freeze.rs`) as a
  public function. The conformance version-gate test calls it instead of
  computing its own sha256. One implementation, two call sites (freeze + test).
- **Single version-gate rule, no escape hatch.** The gate becomes:
  `contracts.lock.hash == computed_hash` AND
  `contracts.lock.version == workspace_version == engine/VERSION`. The current
  drift-tolerating clause (`current == lock_hash || engine_version != lock_version`)
  is deleted — it was designed to tolerate drift rather than prevent it.
- **`contracts.lock.version` always equals the workspace version.** Semantics:
  "this frozen surface was verified correct at version X". freeze stamps the
  current version regardless of whether the surface changed.
- **Dynamic version test.** `version_is_1_1_3` (literal string compare) is
  replaced by a test asserting `--version` output equals
  `format!("mochiflow {}", env!("CARGO_PKG_VERSION"))`. Future bumps need zero
  test edits.

## Architecture

New module `cli/crates/mochiflow-core/src/freeze.rs` owns:

- `read_workspace_version(repo_root) -> Result<String>` — parse
  `cli/Cargo.toml` `[workspace.package].version`.
- `compute_contracts_hash(repo_root) -> Result<String>` — moved from
  `conformance.rs`; same input set (`contracts/*.json` sorted, then
  `tests/conformance/golden/**` sorted).
- `freeze(repo_root, check: bool) -> Result<FreezeReport>` — compute desired
  state of the three derived files; in write mode write any that differ, in check
  mode collect stale entries and return them without writing.

`engine/MANIFEST.json` generation must go through a pure builder. The existing
`upgrade::write_manifest_for_engine_dir()` (`upgrade.rs`) reads `engine/VERSION`
and immediately writes the file in one call — it derives the manifest `version`
from on-disk `engine/VERSION`, not from the workspace SSOT, and it forces a write
before freeze has computed all desired content. freeze must NOT call it directly.
Instead:

- Introduce a pure builder `build_manifest(engine_dir, version) -> String`
  (serialized JSON content) that takes the version explicitly (from the workspace
  SSOT) and performs no writes.
- Refactor `write_manifest_for_engine_dir()` to compute its version via
  `read_engine_version(engine_dir)` (unchanged), pass it into
  `build_manifest(engine_dir, version)`, and write the returned content — so the
  upgrade path is behavior-preserving by construction (same on-disk-derived
  version, same `files` map). The invariant the refactor must preserve is **same
  `version` + same `files` map** (the `BTreeMap` ordering and the
  `MANIFEST.json` / `__pycache__` exclusions), not literal byte identity:
  `doctor engine` compares the `version` field and the per-file hash map
  (`doctor.rs`), and `MANIFEST.json` excludes itself from its own map
  (`upgrade.rs`), so byte-level serialization of the committed file is not the
  gated invariant.
- freeze calls `build_manifest(engine_dir, workspace_version)` to obtain the
  desired manifest content as part of "compute all desired content first", then
  writes only if it differs (write mode) or reports drift (check mode).

The manifest file-map format (`{"version", "files": {path: "sha256:..."}}`) and
the `MANIFEST.json` / `__pycache__` exclusion rules are unchanged.

`main.rs`:
- Add `Commands::Freeze { check: bool }` and route it to
  `freeze::freeze(repo_root, check)`.
- Remove `Commands::Engine` and `EngineCommand` (the `Manifest` handler).

Repo-root resolution (single rule). freeze is a developer repo-tree tool, not an
installed-project tool. It resolves the repo root by walking up from the current
working directory to the first ancestor containing both `cli/Cargo.toml` (with
`[workspace.package].version`) and `engine/VERSION`. The `--config` flag is not
used by freeze. If no such ancestor exists (run from outside the repo, or in an
installed `.mochiflow/`-only tree), freeze fails with a clear
`FAIL: not a mochiflow source repo (no cli/Cargo.toml + engine/ found)` and
writes nothing. CI invokes it from the repo root via
`cargo run --manifest-path cli/Cargo.toml -- freeze --check`, which satisfies the
rule. This resolution and its failure mode are pinned by CLI tests
(run-from-subdir succeeds; run-from-non-repo fails non-zero without writing).

CI (`.github/workflows/ci.yml`): add a step running the built binary's
`freeze --check` in the `rust` job after the build/test steps (it needs the
compiled binary; run via `cargo run --manifest-path cli/Cargo.toml -- freeze --check`).

## Data Model / Interfaces

- `FreezeReport { changed: Vec<PathBuf>, stale: Vec<StaleEntry> }` (or
  equivalent) — drives both human output and the `--check` non-zero exit.
- `contracts.lock` shape unchanged: `{"version": "<x.y.z>", "hash": "<sha256-hex>"}`.
- `engine/MANIFEST.json` shape unchanged: `{"version": "<x.y.z>", "files": {...}}`.
- `engine/VERSION` shape unchanged: bare `x.y.z` + trailing newline.

No public schema (`contracts/*.json`) changes; the frozen surface is byte-stable
except where a real schema/golden edit intentionally changes it.

## Error Handling

- Missing/unparseable `[workspace.package].version` → return an error; `main`
  prints `FAIL: ...` and exits non-zero; nothing is written.
- Missing `contracts/` or `tests/conformance/golden/` → error (freeze is a
  repo-maintenance tool; refuse to write a partial lock).
- `--check` staleness is not an error type per se but yields a non-zero exit with
  a per-file stale report.
- Write failures (I/O) propagate as errors; partial writes are avoided by
  computing all desired content before writing.

## Test Strategy

**Test isolation (mandatory).** No freeze write/check test may mutate the real
repo tree. `read_workspace_version` and `compute_contracts_hash` may read the
real repo read-only (pure reads). Every test exercising freeze write mode,
idempotency, staleness/`--check`, or a version bump / `engine/VERSION` hand-edit
MUST run against a temporary fixture under `tempfile::tempdir()` — a minimal repo
subset (a `cli/Cargo.toml` with `[workspace.package].version`, an `engine/` with
`VERSION` + a couple of files, `contracts/*.json`, and
`tests/conformance/golden/**`) constructed or copied from the real tree. Tests
mutate only the fixture. The only check against the real tree is the final-gate
`mochiflow freeze --check`, which is read-only by definition.

- Unit (in `freeze.rs` / core tests): `read_workspace_version` parses the real
  `cli/Cargo.toml` (read-only); `compute_contracts_hash` equals the committed
  lock hash (read-only); `build_manifest(engine_dir, version)` is pure (no write,
  version comes from the argument); freeze write mode produces consistent files
  in a temp fixture; freeze idempotency (second run = no change) in the fixture;
  `--check` detects a hand-introduced stale file in the fixture and writes
  nothing; repo-root resolution succeeds from a fixture subdir and fails non-zero
  in a non-repo temp dir without writing.
- Conformance (`conformance.rs`): `version_gate_consistent` refactored to the
  single-rule form and to call the core hash fn;
  `version_gate_hash_matches_committed_lock` calls the core fn. (Read-only.)
- CLI (`cli.rs`): dynamic version test replaces `version_is_1_1_3`; a test that
  `mochiflow engine manifest` is rejected (unknown subcommand).
- Full gate: `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`,
  and `mochiflow freeze --check` on the final tree.

## Review Results

<!-- Reviewer mode: delegated | inline / Verdict: pass | pass-with-comments | fail -->
- Pending build (risk elevated → independent-reviewer once, after all tasks).
