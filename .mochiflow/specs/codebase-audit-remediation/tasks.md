# Codebase Audit Remediation — Tasks

Implementation Summary: Establish shared repository containment, then remediate the remaining parser, recovery, integrity, performance, language, conformance, release, documentation, and macOS findings behind one integrated verification gate.
risk: critical
Critical Stop Conditions:
- Stop if a valid shipped config or adapter cannot remain supported without changing the agreed containment contract.
- Stop if acceptance or engine recovery would require deleting, resetting, or overwriting unrelated user state.
- Stop if cargo-dist generation cannot coexist with least-privilege release hardening without an explicit, test-pinned ownership boundary.

## Defaults

- Verification: `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing
- Critical per-task gate: for every `T-###`, run its focused checks and the default verification, retain strong evidence as test output/logs or explicit human confirmation, commit the verified task, run the delegated `change-reviewer` against that task commit, fix and recommit any in-scope finding, and repeat review if the fix changes code. Before starting the next task, append the final `pass` or `pass-with-comments` result with `Reviewed through: <sha>` to `design.md ## Review Results` and append the required task entry to `design.md ## Integration Log`.
- Engine-edit rule: after any repo-root `engine/` edit, run `mochiflow freeze`, `mochiflow upgrade --source engine`, `mochiflow adapter generate`, and `mochiflow adapter generate --check` before the default verification; commit the regenerated `engine/MANIFEST.json` and configured tracked adapter outputs with that task.
- Frozen-contract rule: a `contracts/` or locked engine/schema edit must carry the workspace patch-version bump and coherent Cargo lock, engine version/manifest, contracts lock, changelog, README, and Japanese README references in the same task.

## Tasks

- [x] T-001 [AC-01] Enforce the repository-relative config path contract
  - Depends on: none
  - Files:
    - `cli/crates/mochiflow-core/src/config.rs`
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-core/src/join.rs`
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-core/src/adr.rs`
    - `cli/crates/mochiflow-core/src/accept.rs`
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-core/src/upgrade.rs`
    - `cli/crates/mochiflow-core/src/detach.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `contracts/config.schema.json`
    - `tests/conformance/fixtures/schema/config_good.json`
    - `tests/conformance/fixtures/schema/config_bad_absolute_path.json`
    - `tests/conformance/fixtures/schema/config_bad_parent_path.json`
    - `cli/Cargo.toml`
    - `cli/Cargo.lock`
    - `engine/VERSION`
    - `engine/MANIFEST.json`
    - `contracts/contracts.lock`
    - `CHANGELOG.md`
    - `README.md`
    - `README.ja.md`
    - `AGENTS.md`
    - `.kiro/steering/mochiflow.md`
    - `.kiro/agents/spec-change-reviewer.json`
    - `.kiro/agents/spec-plan-auditor.json`
  - Done: Every repository-owned configured artifact or directory path is lexically and canonically validated at config load, so inventoried read consumers cannot begin outside the repository. Every mutating consumer—including PR delivery scratch, doctor state, engine replacement, init, join, index, ADR, accept, and detach/purge cleanup of configured install/state directories—rechecks canonical containment immediately before mutation; outside symlinks/absolute/parent paths fail without outside writes and inside symlinks pass. Canonicalized paths are containment witnesses only: callers mutate the checked repository-root-joined operation path, and delete uses link-aware metadata so removing a valid local symlink preserves its sentinel target. Representative read consumers, state-backed PR scratch, detach/purge configured cleanup, local-link deletion, and outside-link rejection have regression coverage before T-001's critical review. A repository-wide accessor/consumer search is repeated and every hit is reconciled with the design inventory, while explicit `--body-file`, path-like PR request-directory, and `pr_driver` remain under their existing executable/caller-supplied contracts. The schema encodes the lexical contract and negative fixtures pass. Workspace/version artifacts are coherently bumped to the next patch version; after freeze and engine upgrade, configured tracked adapters are regenerated and check mode proves zero drift with no marker retaining the prior version. The shared `conformance.rs` remains green with all pre-existing structural/prose tests intact for later tasks.
  - Stop: Stop if containment requires rejecting every symlink or changing `schema_version`; report the valid layout or contract that cannot be represented.

- [x] T-002 [AC-02] Confine adapter manifest reads, writes, candidates, and detach cleanup
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/config.rs`
    - `cli/crates/mochiflow-core/src/adapter.rs`
    - `cli/crates/mochiflow-core/src/detach.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: Config load enforces the shipped adapter allowlist; manifest keys/values are lexically safe; templates resolve inside their adapter directory; all generate/candidate/detach targets prove containment through the T-001 boundary while operating on the checked non-canonicalized path. Tests cover unknown IDs, absolute/parent mappings, escaping/local symlinks, all four shipped adapters, and detach of a repository-local symlink that removes the link without deleting its sentinel target. Later shared-file changes in `detach.rs` preserve the already-reviewed T-001 configured install/state cleanup boundary, and adapter generation/check/detach remain mutually consistent.
  - Stop: Stop if a shipped manifest relies on an escaping mapping or if generate and detach cannot consume one validated mapping representation.

- [x] T-003 [AC-03, AC-09] Make metadata and freeze traversal failures explicit
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/spec_meta.rs`
    - `cli/crates/mochiflow-core/src/freeze.rs`
    - `cli/crates/mochiflow-core/src/upgrade.rs`
    - `cli/crates/mochiflow-core/src/doctor.rs`
  - Done: Scalar parsing is fallible and returns a metadata parse error for lone/unterminated quotes while accepting valid empty quoted strings. Recursive manifest collection propagates directory and entry failures with paths; every caller propagates the error; successful manifest bytes/order are unchanged; failure tests prove freeze writes no incomplete authoritative artifact. Shared upgrade/doctor call sites compile with the new fallible manifest boundary and retain their pre-existing successful behavior for T-004.
  - Stop: Stop if preserving the successful manifest byte format would require swallowing traversal errors or if a platform cannot create a deterministic failure fixture without privileged filesystem mutation; use an injected iterator seam instead.

- [x] T-004 [AC-07, AC-10] Add exact acceptance resume and recoverable engine replacement
  - Depends on: T-001, T-003
  - Files:
    - `cli/crates/mochiflow-core/src/accept.rs`
    - `cli/crates/mochiflow-core/src/upgrade.rs`
    - `cli/crates/mochiflow-core/src/doctor.rs`
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-core/src/join.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
  - Done: `accept` distinguishes normal, exact resumable, already-committed, and invalid accepted states; a failing hook followed by retry succeeds only after lint/final verification/staged-path validation; unrelated staging blocks without rollback. Existing engine integrity failures require `--force`; swap/rollback/cleanup errors retain and report backup paths; init/join/upgrade tests cover missing, malformed, mismatched, drifted, forced, rollback-failed, and cleanup-failed cases. Shared `init.rs`, `conformance.rs`, and upgrade/doctor code retain all earlier path/parser/freeze guarantees.
  - Stop: Stop before any solution that resets the index/worktree, removes unrelated staging, deletes a failed backup, or treats an unknown manifest as clean.

- [x] T-005 [AC-05] Render index outputs from one collected delivery snapshot
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/index.rs`
    - `cli/crates/mochiflow-core/src/delivery.rs`
  - Done: One `generate_index_inner` call performs one delivery collection and passes the same immutable snapshot to Markdown and JSON renderers; a deterministic probe seam asserts one provider/Git signal pass per spec; golden Markdown, JSON next-action, stale-check, and ordering tests remain unchanged. Shared `index.rs` retains T-001 checked output paths and leaves those checks applied before both writes.
  - Stop: Stop if avoiding double collection would make Markdown and JSON use different filtering/order rules or introduce a cache that survives one command invocation.

- [x] T-006 [AC-04, AC-08, AC-11] Complete the AI language handoff and align public verification guidance
  - Depends on: T-001
  - Files:
    - `cli/crates/mochiflow-core/src/init.rs`
    - `cli/crates/mochiflow-core/src/present.rs`
    - `cli/crates/mochiflow-core/src/config.rs`
    - `cli/crates/mochiflow-core/src/status.rs`
    - `cli/crates/mochiflow-core/src/pr.rs`
    - `cli/crates/mochiflow-core/src/delivery.rs`
    - `cli/crates/mochiflow-cli/tests/cli.rs`
    - `cli/crates/mochiflow-cli/tests/first_run.rs`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `engine/commands/onboard.md`
    - `engine/reference/language.md`
    - `engine/MANIFEST.json`
    - `docs/getting-started.md`
    - `docs/configuration.md`
    - `CONTRIBUTING.md`
    - `CONTRIBUTING.ja.md`
    - `.github/PULL_REQUEST_TEMPLATE.md`
    - `README.md`
    - `README.ja.md`
  - Done: Explicit/existing languages remain authoritative; otherwise config carries a concrete provisional artifact language plus a confirmation marker and init reports AI review. Onboard selects any valid concrete BCP 47-style artifact language from repository evidence, removes the marker, and preserves conversation `auto` by default. Deterministic CLI rendering recognizes `ja-*`, otherwise falls back consistently to English with no mixed status output. The shared classifier covers both delivery next-action variants as rendered through status and index JSON; `ja-JP` and unsupported fixed tags have regression assertions without changing `next_action.kind`, and shared `delivery.rs` preserves T-005's single-snapshot/probe behavior. Public docs use only current split flags. Focused conformance assertions inspect both contributor guides and the pull-request template for the complete default test/format/Clippy/freeze gate; both guides separately label the test-only command as a fast loop. Engine source is frozen/re-vendored, shared README version references from T-001 remain correct, and shared conformance tests retain all prior guarantees.
  - Stop: Stop if the design requires storing `artifact_language = "auto"`, calling an AI from Rust, or adding translations beyond the agreed Japanese/English deterministic CLI boundary.

- [ ] T-007 [AC-12] Migrate structural engine conformance to semantic parsing
  - Depends on: T-001, T-006
  - Files:
    - `cli/Cargo.toml`
    - `cli/Cargo.lock`
    - `cli/crates/mochiflow-cli/Cargo.toml`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `cli/deny.toml`
  - Done: `yaml-rust2` 0.11 is dev-only and passes cargo-deny; helpers parse typed frontmatter references/load contracts and normalized router rows with source-bearing failures; structural tests no longer depend on line wrapping; intentionally behavioral prose assertions remain short and explicit. The shared `conformance.rs` still covers every prior contract and all T-001/T-004/T-006 additions, while a reflow fixture proves structural checks survive harmless wrapping.
  - Stop: Stop if the dependency violates MSRV/license policy, if migration weakens a user-visible behavior assertion, or if a general-purpose Markdown/YAML rewrite is needed beyond the selected structural checks.

- [ ] T-008 [AC-06] Harden and cost-filter release planning and publication
  - Depends on: T-007
  - Files:
    - `dist-workspace.toml`
    - `.github/workflows/release.yml`
    - `.github/workflows/release-plan.yml`
    - `.github/scripts/validate-release-provenance.sh`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `docs/release-verification.md`
  - Done: cargo-dist config skips its built-in PR trigger and regenerates a tag-oriented baseline; the separate plan workflow runs only for release/package/toolchain metadata paths or manual dispatch; both workflows install exactly cargo-dist 0.32.0 through locked Cargo sources rather than piping a downloaded script. Default permissions are read-only, write/OIDC scopes are job-minimal, actions use full SHAs, and no PR job receives release secrets. The release workflow calls the checked-in provenance helper before any side effect; it derives the workspace version from Cargo metadata and rejects missing refs, unreachable tag commits, mismatched versions, and malformed tags. Temporary-Git conformance fixtures prove reachable/matching passes plus unreachable, version-mismatched, and malformed cases fail without credentials or publication; static checks prove every publishing job depends on validation. `dist plan`, dynamic provenance tests, workflow conformance, and the full default profile pass; shared conformance tests retain semantic and earlier remediation coverage. Any manual hardening delta from generated output is explicitly documented and test-pinned.
  - Stop: Stop if a write-scoped token/secret must reach PR code, provenance cannot be checked before publication, or regeneration silently removes an unguarded hardening delta.

- [ ] T-009 [AC-13] Add the cost-bounded post-merge macOS test workflow
  - Depends on: T-008
  - Files:
    - `.github/workflows/macos.yml`
    - `cli/crates/mochiflow-cli/tests/conformance.rs`
    - `docs/release-verification.md`
  - Done: A read-only, full-SHA workflow runs only `cargo test --manifest-path cli/Cargo.toml` on `macos-latest` for relevant `main` path changes or manual dispatch, with concurrency cancellation and no PR/schedule/matrix/secrets. Static event-policy tests reject trigger broadening; release docs explain Linux completion versus post-merge macOS coverage. Run final `dist plan`, cargo-deny, engine freeze/upgrade/adapter check, default verification, spec lint, and doctor; shared `conformance.rs` remains one coherent suite covering all thirteen findings.
  - Stop: Stop if GitHub workflow semantics would run macOS for ordinary PRs/unrelated changes or if making it a pre-merge acceptance gate is required.
