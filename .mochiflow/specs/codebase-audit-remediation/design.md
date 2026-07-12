# Codebase Audit Remediation — Design

## Design Decisions

### One checked repository-path boundary

Add a shared path-validation boundary in `mochiflow-core` and route every
repository-owned configured or adapter-derived mutation through it. Validation
has two layers:

1. Lexical validation rejects empty values, absolute roots/prefixes, and any
   parent component on both slash conventions.
2. Filesystem containment canonicalizes the repository root and the deepest
   existing ancestor of a destination, thereby resolving symlinks before a
   create/write. Existing read/delete targets are canonicalized only to produce
   a containment witness.

The effective existing location must start with the canonical repository root.
The non-existing tail can then be appended only after lexical validation. This
keeps valid repository-local symlinks and rejects symlink escapes. Mutation
callers recheck at the operation boundary rather than trusting an earlier config
load indefinitely.

Canonicalization is check-only: it must never silently replace the filesystem
operation path. After the witness proves containment, callers retain the
lexically safe repository-root-joined path for read, write, rename, and delete,
preserving normal symlink semantics. In particular, deletion uses symlink
metadata when necessary and removes an allowed link entry rather than its
resolved in-repository target. A checked-path result may carry both the
operation path and resolved witness so callers cannot confuse their roles.

This boundary protects against malicious or mistaken checked-in configuration,
adapter manifests, and the repository's pre-existing symlink layout. It assumes
the filesystem namespace remains stable for the duration of each checked
operation. A separate same-privilege local process replacing an ancestor after
validation but before use is outside this threat model; closing that race would
require descriptor- or capability-relative I/O throughout the affected modules.
Tests therefore exercise static and pre-operation replacement layouts, not
concurrent namespace mutation.

Primary basis: Rust 1.96 `std::fs::canonicalize` resolves symbolic links and
normalizes intermediate components, while `std::path` lexical operations do not
resolve `..` or symlinks:
https://doc.rust-lang.org/1.96.0/std/fs/fn.canonicalize.html and
https://doc.rust-lang.org/1.96.0/std/path/index.html.

The config schema will encode the lexical portion of the already-documented
relative-path contract. Runtime remains authoritative for platform components
and symlink containment. Because `contracts/config.schema.json` is frozen, the
workspace receives the next patch version and all derived/version references
are regenerated in the same task; `schema_version` remains `1` because paths
that escaped the repository were already outside the documented contract.
The same task regenerates every configured tracked adapter after installing the
versioned engine, then runs check mode. This keeps generated version markers in
`AGENTS.md` and `.kiro/` coherent instead of hand-editing vendored outputs.

Here, a repository-owned configured path means a project artifact or directory
whose logical destination must live inside the repository. It excludes
executable/process configuration and explicit caller-supplied inputs. The
implementation inventory is exhaustive by accessor and consumer, not only by
the modules named in the original audit:

| Configured path family | Known consumers | Access | Required boundary |
| --- | --- | --- | --- |
| `install_dir` / engine directory | init, join, upgrade, adapter, detach, doctor, CLI display | read + mutation | validate at config load; recheck immediately before install, replace, generate, or detach mutation |
| `state_dir` | index, PR delivery scratch, doctor, join, detach, delivery | read + mutation | validate reads at config load; recheck before index/state/PR-scratch/doctor writes or cleanup |
| `specs_dir` | init, accept, index, backlog, status, lint, spec mode, CLI lint target | read + mutation | validate reads at config load; recheck before init, accept, and index mutations |
| index path | index, join, CLI display | read + mutation | validate at config load; recheck before index generation or join writes |
| constitution and context paths | init, doctor, adapter substitutions, CLI display | read + mutation | validate reads at config load; recheck before initialization writes |
| ADR paths | ADR commands, accept, init, detach, CLI display | read + mutation | validate reads at config load; recheck before ADR creation, acceptance updates, initialization, or detach cleanup |

The build repeats a repository-wide search for every repository-owned
configured-path accessor and reconciles every hit with this table before
considering T-001 complete.
Explicit caller-supplied paths such as `--body-file`, path-like PR request
directories, and `pr_driver` retain their existing separate contracts; they are
not silently reclassified as repository-owned configured paths.

### Adapter manifests use the same boundary

Config load enforces the shipped adapter IDs (`kiro`, `agents`, `copilot`, and
`claude-code`). Manifest loading rejects absolute/parent-containing keys and
values before access. Template reads are confined to the canonical adapter
directory; output, candidate, generation, and detach paths are confined to the
canonical repository. `generate` and `detach` consume the same validated
mapping so their safety rules cannot drift.

### Acceptance resumes; it does not roll back

`accept` has two legal entry shapes:

- normal: metadata is `approved` and no close-out mutation has occurred;
- resume: metadata is `accepted`, the close-out trailer is not committed, and
  the working tree/staged set contains only the target spec plus linked ADR
  records allowed by the existing `AcceptPaths` contract.

Resume reconstructs state from disk and Git, reruns lint and final verification,
keeps matrix updates idempotent, validates the cached name-status set, and then
retries the commit. An already committed accepted close-out is an idempotent
success. Any unrelated change, incomplete accepted shape, or new design need
stops without mutation. Automatic rollback is forbidden because it cannot
safely distinguish user edits or pre-existing staged intent.

### Engine replacement fails closed with recoverable backups

For an existing engine, missing manifest, malformed manifest, version mismatch,
or file drift all count as untrusted integrity. Replacement requires `--force`
in those cases. Staging still validates the replacement independently.

The swap error model distinguishes initial backup rename, staged install,
rollback, and backup cleanup. If rollback fails, the backup is retained and its
path is part of the reported error. Cleanup failure is not silently ignored.
Legacy users with no manifest must perform one explicit forced replacement; no
local engine contents are silently blessed as a new baseline.

### Freeze traversal is all-or-nothing

Recursive collection returns a path-bearing error for both `read_dir` and
individual entry failures. Manifest construction returns before the derived
target list is written. Existing deterministic ordering and byte format remain
unchanged on success.

### One index snapshot feeds every renderer

Collection yields one immutable snapshot containing active entries, done
entries, and seeds. Markdown and JSON rendering become pure consumers of that
snapshot. `is_index_stale` may collect its own snapshot for its independent
check, but one `generate_index_inner` invocation performs exactly one
collection. Tests use a probe seam/counter around delivery signal gathering so
the single-pass property is machine-checkable without real GitHub traffic.

### AI owns uncertain artifact-language selection

Explicit flags and an existing concrete i18n config remain authoritative. When
neither exists, CLI heuristics may provide a concrete provisional display value,
but the rendered config marks `artifact_language` with
`# mochiflow: confirm` and init reports `NeedsAiReview`. `onboard` must inspect
repository human-facing content, select any valid concrete BCP 47-style tag,
persist it, and remove the marker. It keeps `conversation_language = "auto"`
unless the user explicitly asks for a fixed language.

The CLI does not call an AI. Central deterministic presentation classifies
`ja` and `ja-*` as Japanese and uses English for other fixed CLI strings,
eliminating mixed English/Japanese output while AI-authored artifacts and
conversation remain governed by the persisted/auto engine policy. Public docs
state this boundary and use only current split language flags.

The shared classifier also governs delivery next-action messages consumed by
status and index JSON rendering. Regional-Japanese and unsupported-tag tests
cover both next-action variants without changing stable `next_action.kind`
values or T-005's single-snapshot delivery collection.

### Structural conformance uses YAML/Markdown semantics

Add `yaml-rust2` 0.11 as a dev-only dependency for YAML 1.2 parsing. A small
test helper converts engine frontmatter into a typed view of `references`,
`load.required`, and `load.conditional[].files`; malformed or wrong-typed
metadata fails with the source path. A separate focused parser reads the router
Markdown table into route records. Structural ownership/load/route tests compare
these models rather than raw line wrapping.

Only structural assertions migrate. Short literal assertions remain where exact
agent-facing wording, sequencing, or a stable command token is the actual
contract. This resolves the active line-wrap pitfall without weakening behavior
guards.

Primary basis: `yaml-rust2` 0.11 is a pure-Rust YAML 1.2 parser with MSRV below
this workspace's Rust 1.96 and documented parse errors:
https://docs.rs/yaml-rust2/0.11.0/yaml_rust2/.

### Release planning and publishing are separate workflows

Set cargo-dist 0.32 `pr-run-mode = "skip"`, then regenerate its tag-oriented
release workflow. Add a small repository-owned `release-plan.yml` triggered
only when `dist-workspace.toml`, release workflows, Rust package metadata,
lockfile, or toolchain changes in a pull request, plus manual dispatch. Both the
rare PR plan and tag release install exactly cargo-dist 0.32 with
`cargo install cargo-dist --version 0.32.0 --locked`, avoiding execution of a
downloaded installer script and relying on Cargo's locked source/checksum path.

The release workflow defaults to read-only permissions. Only artifact
attestation jobs get `attestations: write`/`id-token: write`; only release
creation/announcement gets `contents: write`; the Homebrew publisher receives
only its explicitly required token. Every `uses:` entry stays pinned to a
verified full commit SHA.

Before dist planning for a tag, checkout has sufficient history, the tag commit
must be an ancestor of `origin/main`, and the normalized `v<version>` tag must
equal the workspace version reported by Cargo metadata. Failure stops before
build, attestation, upload, or publication. A conformance test pins these local
hardening deltas because cargo-dist does not expose every built-in job permission
or bootstrap-install choice through configuration.

The workflow invokes a checked-in `.github/scripts/validate-release-provenance.sh`
seam rather than embedding untestable ancestry/version expressions. The helper
accepts the tag and main ref, derives the workspace version through Cargo
metadata, and returns non-zero for an unreachable tag commit, mismatched or
malformed tag, or missing ref. Conformance tests run it in temporary Git
repositories for four dynamic cases: reachable/matching passes; unreachable,
version-mismatched, and malformed tags fail. Static workflow checks additionally
prove every publishing side effect depends on successful validation. Tests use
no release credential and perform no publication.

Primary basis:

- GitHub least privilege and full-SHA guidance:
  https://docs.github.com/en/actions/reference/security/secure-use
- GitHub workflow permissions and path filtering:
  https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- cargo-dist 0.32 `pr-run-mode` and CI configuration:
  https://axodotdev.github.io/cargo-dist/book/reference/config.html
- cargo-dist's official locked source installation option:
  https://axodotdev.github.io/cargo-dist/book/install.html

### macOS is post-merge and path-filtered

A separate macOS workflow runs `cargo test --manifest-path cli/Cargo.toml` on
`macos-latest` only for `main` pushes changing `cli/**`, `rust-toolchain.toml`,
or the macOS workflow itself, and on manual dispatch. It has read-only contents
permission, full-SHA actions, no schedule, no PR trigger, no architecture
matrix, and no release secrets. Linux remains the completion gate for format,
Clippy, freeze, and cargo-deny.

## Architecture

The change keeps existing module ownership and introduces narrow shared seams:

- `config.rs` owns lexical repository-owned project-path validation, canonical
  repository containment helpers, adapter allowlist validation, and config-load
  errors.
- Existing config accessors remain the source of logical destinations; mutating
  modules call the checked resolver immediately before filesystem operations.
- `adapter.rs` owns validated manifest mappings; `detach.rs` consumes them.
- `accept.rs` owns normal/resume close-out classification and Git staging
  validation.
- `upgrade.rs` owns engine-integrity gating and swap/rollback recovery.
- `freeze.rs` owns error-propagating recursive collection.
- `index.rs` owns the single collected snapshot and both renderers.
- `init.rs`/`present.rs` own provisional language handoff and deterministic CLI
  rendering; `engine/commands/onboard.md` owns AI language selection.
- Test-only semantic frontmatter/router helpers live next to conformance tests
  rather than becoming a new runtime API.
- cargo-dist continues to own the tag release pipeline baseline;
  repository-owned workflows and conformance guards own local hardening and
  cost policy.

## Data Model / Interfaces

- Project-path validation accepts a field label, repository root, relative raw
  value, and operation intent (existing read/delete or possibly-new write). It
  returns the normalized repository-root-joined operation path plus a resolved
  containment witness, or a path-bearing config/operation error; callers never
  use the witness itself as the mutation target.
- Adapter manifest loading returns only validated mappings; callers do not
  receive unchecked target/template strings.
- Spec scalar parsing becomes fallible and propagates metadata parse errors
  through mapping/list parsing.
- Freeze recursive collection returns `Result` and preserves the failing path.
- Index Markdown and JSON renderers accept the same borrowed snapshot rather
  than a `Config` that can recollect.
- Accept entry classification distinguishes normal, resumable, already
  committed, and invalid accepted states without introducing a new persisted
  lifecycle enum.
- Engine install errors carry the operation stage and backup path when recovery
  is possible.
- Init presentation exposes whether artifact language is explicit, existing,
  docs-derived, or provisional/AI-review-required in both human and JSON forms.
- The semantic conformance helper exposes typed frontmatter load/reference sets
  and normalized router rows only to tests.

## Error Handling

- Path rejection names the config/manifest field and offending relative path but
  does not expose unrelated file contents.
- A containment error occurs before create, write, rename, or remove; no fallback
  silently joins an unchecked path.
- Missing ancestors are handled by validating the deepest existing ancestor;
  inability to canonicalize an existing ancestor is a hard error.
- Malformed YAML scalars return the existing metadata parse error family.
- Accept resume prints whether it is blocked by unrelated state, incomplete
  accepted evidence, verification, staging validation, or commit failure.
- Freeze reports the unreadable directory or entry and writes no partial frozen
  state.
- Engine install errors retain backup paths until recovery succeeds; cleanup
  errors are visible and never reported as a clean success.
- Release provenance or version mismatch exits before any write-scoped job can
  run.
- Unsupported deterministic CLI locales use documented English fallback; AI
  artifact/conversation language is not silently rewritten.

## Test Strategy

- Add table-driven unit tests for Unix/Windows lexical path forms, existing and
  missing destinations, local and escaping symlinks, and adapter IDs.
- Add CLI integration tests proving index/ADR/spec/config writes and adapter
  generate/detach never escape a temporary repository. A local symlink-delete
  fixture retains a sentinel target while removing the link entry; an
  outside-target symlink remains rejected.
- Add malformed scalar tests using `catch_unwind` only as an assertion aid; the
  public result must be a parse error.
- Add accept integration coverage with a failing Git hook followed by an exact
  retry, unrelated staged-state rejection, and already-committed idempotency.
- Add engine tests for missing/malformed/version-mismatched manifests, forced
  replacement, install failure, rollback failure, and retained backup paths.
- Add a deterministic probe counter/fake for one-pass index collection and
  retain golden Markdown/JSON equivalence tests.
- Add init/onboard/docs tests for arbitrary language handoff, explicit override,
  `conversation_language = "auto"`, regional Japanese CLI selection, English
  fallback, and removed flag absence.
- Parse all engine frontmatter with YAML 1.2, migrate selected structural tests,
  and retain explicit prose-contract tests.
- Add static workflow conformance for triggers, job permissions, SHA pins,
  locked cargo-dist install, tag provenance ordering, and macOS path/event
  policy. Add temporary-repository tests for the workflow-used provenance helper
  covering reachable/matching, unreachable, version-mismatched, and malformed
  tags; run `dist plan` locally.
- Run the full default profile after every task. After engine edits, freeze and
  re-vendor before the profile. Run cargo-deny and doctor in final integration.

## Workstreams

| Workstream | Surface | Responsibility | Depends on | Verification |
| --- | --- | --- | --- | --- |
| Path and adapter safety | cli | Config/schema containment, adapter manifest confinement, version gate | path foundation before adapter | targeted path/adapter tests + default profile |
| Recovery and integrity | cli | Accept retry, engine swap recovery, freeze traversal | path contract for configured close-out paths | failure-injection tests + default profile |
| Performance and language | cli | Single index snapshot, AI language handoff, deterministic CLI fallback, docs | path foundation for index output | index probe tests, init/docs tests + default profile |
| Contract tests | cli | Semantic YAML/router/load checks with retained prose guards | version/dependency updates | conformance suite + cargo-deny |
| Delivery automation | cli | Cost-filtered release planning, provenance, permissions, macOS main tests | conformance helper available | dynamic provenance fixtures + local dist plan + workflow conformance |

## Integration Contract

- Contract owner: the Rust CLI and checked-in GitHub workflow/configuration
  files.
- Request: repository config, adapter manifests, spec metadata, Git lifecycle
  state, engine directories, and GitHub events/tags.
- Response: confined filesystem operations, deterministic CLI results, coherent
  index artifacts, recoverable lifecycle/engine failures, and least-privilege
  CI execution.
- Error: invalid or escaping paths, malformed metadata, incomplete accepted
  state, untrusted engine integrity, traversal failure, or invalid release
  provenance returns non-zero before unsafe mutation/publication.
- Auth: pull-request validation receives no write permission or release secret;
  write/OIDC permissions exist only on the jobs that publish or attest.
- Compatibility: valid schema-version-1 repository-relative configs and all
  shipped adapter manifests remain accepted; invalid out-of-repository paths
  become explicit errors; public Markdown/JSON index structure remains stable.
- Failure handling: no silent fallback outside the repository, no automatic
  acceptance rollback, retained engine backup on failed recovery, and no release
  side effect before provenance validation.
- Verification: Rust unit/integration/conformance suites, frozen contract gate,
  dynamic temporary-Git provenance fixtures, cargo-dist plan, workflow policy
  assertions, cargo-deny, adapter drift check, spec lint, and doctor.

## Review Results

Pre-implementation plan audits are advisory and do not replace implementation
review. During implementation, the critical change-reviewer runs after every
task. Append one result per task with `Review profile: change-reviewer`,
`Reviewer mode: delegated | inline`, `Verdict: pass | pass-with-comments`, and
`Reviewed through: <sha>` on its own line. A fix commit makes the prior verdict
stale and requires a fresh result through the corrected task commit before the
next task starts.

### T-001

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass-with-comments
Reviewed through: e451d90

The first inline pass found incomplete mutation-time rechecks and an ancestor
walk that conflated permission errors with missing paths. Both were corrected
and the full default profile passed again. Remaining non-blocking command-level
adapter/local-link coverage is assigned to T-002; T-001 directly tests
cross-platform lexical rejection plus local and escaping symlink witnesses.

### T-002

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass-with-comments
Reviewed through: f6ed35d

No blocking mapping escape remained: runtime config rejects unknown tools,
manifest keys and values are lexically checked, templates are confined to the
adapter root, and generate/detach recheck repository outputs. The non-blocking
comment is to retain command-level output-symlink deletion coverage as the
suite evolves; the shared witness and template-symlink escape are directly
tested in this task.

### T-003

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass
Reviewed through: d0fec7e

The first pass found fallible directory iteration still followed by infallible
metadata predicates. The corrected collector now propagates `read_dir`, entry,
metadata, and recursive failures with paths; no blocking finding remains.

### T-004

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass-with-comments
Reviewed through: 4901af3

The state discrimination keeps unrelated changes blocked, treats a clean
accepted state idempotently, and reruns the existing lint/verification/staged
path pipeline for an uncommitted close-out. Engine integrity fails closed on
manifest failures and exposes retained backup paths on rollback/cleanup errors.
Filesystem fault injection should expand when dedicated fault seams exist.

### T-005

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass-with-comments
Reviewed through: 5895d00

`generate_index_inner` now collects once and passes immutable slices to both
renderers without changing stale-check ownership. Golden Markdown/JSON tests
remain green; a dedicated delivery-probe counter would make the performance
invariant more explicit but no duplicate collection remains in generation.

### T-006

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass-with-comments
Reviewed through: 2155bd8

Current split flags replace removed documentation, onboarding owns uncertain
artifact-language selection, and one classifier handles `ja-*` across CLI
presentation with English fallback. Existing language suites pass; explicit
regional-tag assertions should continue to grow with new presentation paths.

### T-007

Review profile: change-reviewer
Reviewer mode: inline
Verdict: pass
Reviewed through: 5abf73d

Typed YAML parsing now validates frontmatter structure and declared paths;
normalized Markdown table rows validate router mappings. Intentional prose
guards remain unchanged, and yaml-rust2 0.11 passed cargo-deny.

## Integration Log

Append one entry after every task during build. Each entry records the task ID,
verification evidence location, seam drift from this design, ownership-boundary
changes, dead-code handling, recovery behavior exercised, and next-session
handoff notes. Do not duplicate the commit log or restate unchanged plan
decisions. There are no implementation entries at plan time.

### T-001 — repository path boundary

- Evidence: the default profile passed after commit `e451d90`; focused config
  tests cover Unix/Windows spellings, missing tails, repository-local symlinks,
  and escaping symlinks; schema conformance covers absolute and parent paths.
- Seam/ownership: `config.rs` owns lexical validation and the canonical
  containment witness. Commands retain the non-canonical operation path and
  recheck configured path families at mutation boundaries.
- Dead code: the install-only validator was replaced by the shared
  repository-path inventory.
- Recovery: inspection/canonicalization failures stop before mutation; no
  rollback or cleanup is attempted.
- Handoff: T-002 routes adapter template/output/candidate/detach mappings
  through this boundary and adds command-level local-link deletion coverage.

### T-002 — adapter manifest confinement

- Evidence: focused adapter tests and the full default profile passed through
  `f6ed35d`; malicious output/template traversal and a template symlink escape
  are rejected.
- Seam/ownership: manifest loading validates the shared mapping representation;
  generation, candidate writes, and detach additionally recheck their exact
  operation destinations.
- Dead code: no alternate unchecked manifest resolver was introduced.
- Recovery: an unsafe mapping becomes a normal adapter error before target or
  candidate mutation; detach records the error without cleanup.
- Handoff: T-003 can change parser/freeze fallibility without altering adapter
  path ownership.

### T-003 — fallible metadata and freeze traversal

- Evidence: malformed/lone quoted scalar tests, path-bearing recursive failure
  coverage, and the full default profile passed through `d0fec7e`.
- Seam/ownership: scalar parsing is fallible at map/list call sites; recursive
  file collection owns all traversal and metadata error propagation.
- Dead code: the silent `flatten`/early-return traversal path was removed.
- Recovery: freeze constructs every desired artifact before its write loop, so
  traversal failure leaves authoritative derived files untouched.
- Handoff: T-004 can rely on explicit manifest failures while hardening engine
  replacement and acceptance recovery.

### T-004 — close-out and engine recovery

- Evidence: existing accept staging/dirty-state and engine drift/force
  integration coverage plus the full default profile passed through `4901af3`.
- Seam/ownership: accepted resume reuses normal readiness, verification, lint,
  staged validation, and commit paths; engine swap owns backup reporting.
- Dead code: silent rollback and backup-cleanup `.ok()` paths were removed.
- Recovery: exact accepted work retries without reset; failed rollback/cleanup
  returns the preserved backup path.
- Handoff: T-005 changes only index collection/render ownership.

### T-005 — single index snapshot

- Evidence: index golden/JSON/stale tests and the full default profile passed
  through `5895d00`.
- Seam/ownership: generation owns collection; renderers consume borrowed
  immutable snapshot slices.
- Dead code: the generation-time second `collect` call was removed.
- Recovery: collection failure/fallback behavior remains command-local.
- Handoff: T-006 can reuse delivery rendering without adding collection.

### T-006 — language handoff and contributor gate

- Evidence: init/presentation/conformance suites, engine sync, adapter drift,
  and the full default profile passed through `2155bd8`.
- Seam/ownership: AI onboarding selects artifact language; Rust owns only
  deterministic Japanese-or-English presentation.
- Dead code: public `--language` examples and scattered exact-`ja` checks were
  removed.
- Recovery: arbitrary configured tags persist; unsupported deterministic UI
  locales fall back to English.
- Handoff: T-007 may parse engine contracts semantically without changing
  language behavior.

### T-007 — semantic conformance

- Evidence: 196 conformance tests, default profile, and cargo-deny passed
  through `5abf73d`.
- Seam/ownership: typed YAML owns structural frontmatter/load checks; normalized
  table rows own route mappings; prose tests retain behavior wording.
- Dead code: selected formatting-sensitive scans were removed.
- Recovery: parser failures identify malformed engine frontmatter immediately.
- Handoff: T-008 can add workflow policy assertions to the same semantic suite.
