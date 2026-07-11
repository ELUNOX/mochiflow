# Codebase Audit Remediation

## Problem

A repository-wide audit identified thirteen high-confidence improvements across
MochiFlow's filesystem boundaries, adapter installation, lifecycle recovery,
engine integrity, metadata parsing, index performance, release security,
platform coverage, localization, conformance tests, contributor guidance, and
public documentation. Several findings can write or remove files, publish
release artifacts, or leave lifecycle state difficult to recover, so addressing
them independently would duplicate safety primitives and prolong exposure to
known failure modes.

The work should land as one coherent remediation because the findings share the
same Rust CLI surface, verification gate, release workflow, and filesystem
safety model. The change is intentionally classified as elevated rather than
critical at the maintainer's direction; planning must still give the security
and destructive-operation work explicit negative tests, stop conditions, and
rollback or recovery behavior.

## Appetite

One expanded, multi-day fix delivered through a single branch and pull request.
The plan should split implementation into dependency-ordered workstreams so the
shared path-safety foundation lands before adapter confinement, while
independent documentation, CI, performance, parser, and test-architecture work
can proceed without weakening the final integrated verification gate.

## Solution

Address all thirteen audited findings in one spec:

1. Validate every configured project path as non-empty, relative, and free of
   parent traversal. Before mutation, resolve existing ancestors and require the
   effective destination to remain within the repository. Permit symlinks only
   when their resolved target remains inside the repository.
2. Enforce the supported adapter allowlist and validate every adapter manifest
   template and output mapping. Templates must remain under their adapter
   directory; generated, candidate, and detach targets must remain under the
   repository. Generation and detach must share the checked resolver.
3. Reject malformed one-character or unterminated quoted YAML scalars with a
   normal metadata parse error instead of panicking.
4. Replace removed `mochiflow init --language` documentation with the current
   artifact/conversation language model and current CLI options.
5. Collect delivery state once per index generation and render both Markdown
   and JSON from the same immutable snapshot, avoiding duplicate Git and GitHub
   probes.
6. Harden release automation with least-privilege job permissions, full-SHA
   action pins, checksum verification for downloaded cargo-dist tooling, and no
   secrets or write token in pull-request planning. Run pull-request release
   planning only when release configuration or package metadata changes; run
   build and publication on release tags. Before publication, require the tag
   commit to be reachable from `origin/main` and the tag version to match the
   workspace package version.
7. Make `mochiflow accept` safely resumable after staging, hook, signing, or
   commit failure. Recognize the exact accepted-but-uncommitted close-out state,
   reject unrelated changes, and rerun lint, final verification, and staged-path
   validation before retrying the commit. Do not automatically roll back user
   edits or staging state.
8. Make contributor documentation name the complete default verification gate
   (tests, formatting, Clippy, and freeze check) while retaining the test-only
   command as a fast feedback loop.
9. Propagate recursive directory and entry errors during freeze manifest
   construction, aborting before any incomplete derived manifest is written.
10. Make engine replacement fail closed when an existing engine manifest is
    missing, malformed, or inconsistent unless the caller explicitly passes
    `--force`. Preserve and report the backup location when installation or
    rollback fails; never silently discard a rollback error.
11. Keep arbitrary BCP 47-style artifact and conversation language settings,
    but let the AI choose and persist a concrete artifact language during
    initialization based on repository evidence. Keep conversation language
    dynamic with `auto`. AI-authored specs, PR text, and QA follow the persisted
    artifact language; deterministic CLI text keeps a documented English
    fallback instead of attempting runtime AI translation.
12. Replace formatting-sensitive conformance checks with semantic parsing only
    for structural contracts: typed YAML frontmatter, router table entries, and
    required/conditional load declarations. Retain a small set of intentional
    prose assertions where wording itself is an agent-facing behavior contract.
13. Add a cost-bounded macOS test job that runs the Rust test suite only after
    relevant CLI, Cargo, or toolchain changes reach `main`, plus manual dispatch.
    Keep formatting, Clippy, freeze, and cargo-deny on Linux; do not add a
    scheduled run or per-pull-request macOS matrix.

Use the official GitHub Actions security guidance as the basis for workflow
permissions and immutable action references. Preserve existing cargo-dist
generation ownership rather than hand-maintaining generated workflow sections
when the upstream tool exposes the needed configuration.

## Rabbit Holes

- Do not treat lexical path checks as sufficient: symlink escapes must be
  covered without banning valid in-repository symlinks.
- Do not turn acceptance recovery into a broad rollback mechanism that can
  erase unrelated work or alter pre-existing staging state.
- Do not replace all prose conformance assertions mechanically; distinguish
  structural contracts from intentionally pinned agent-facing language.
- Do not hand-edit the vendored `.mochiflow/engine/`; source changes belong
  under repo-root `engine/` and must follow the required freeze, upgrade, and
  adapter drift sequence.
- Do not make macOS run on every pull request or on a schedule; the selected
  policy optimizes cost for a solo maintainer while still testing merged CLI
  changes before release.
- Do not weaken release validation merely to avoid a cargo-dist regeneration
  constraint. If upstream generation cannot express a required hardening,
  planning must identify the supported extension point or stop for a bounded
  decision rather than silently forking generated output.

## No-gos

- No instruction catalog, context-staleness feature, or non-GitHub reference PR
  driver; those direction ideas are not part of the thirteen remediation items.
- No third-language translation catalog or AI invocation from the Rust CLI at
  runtime.
- No automatic rollback of acceptance state, destructive cleanup of unrelated
  files, or silent migration of out-of-repository configured paths.
- No redesign of spec lifecycle states, approval gates, PR provider contracts,
  or the public adapter manifest format beyond the validation needed for safe
  existing mappings.
- No full rewrite of the conformance suite and no removal of wording assertions
  that protect observable agent behavior.
- No macOS architecture matrix and no release build on ordinary code pull
  requests.

## Alternatives Considered

- Split the findings into many specs: rejected because the path checks,
  destructive-operation safety, verification gate, and release hardening need a
  coordinated contract, and the maintainer explicitly chose one spec.
- Use lexical path validation only: rejected because symlink traversal can still
  escape the repository.
- Ban all symlinks: rejected because it would break legitimate repository-local
  layouts without improving containment over resolved-path enforcement.
- Roll back `accept` automatically after commit failure: rejected because it can
  overwrite user edits or disturb existing staged changes; exact-state resume is
  safer.
- Allow upgrade when manifest integrity is unknown: rejected because local
  engine changes cannot be distinguished from a legacy or corrupt install;
  explicit `--force` records overwrite intent.
- Restrict configured languages to Japanese and English: rejected because AI
  artifacts can support arbitrary configured languages and existing BCP 47
  values must remain valid.
- Run macOS tests on every pull request: rejected as unnecessary cost for a solo
  maintainer; relevant changes are tested on `main` with manual dispatch
  available.
- Run release automation only after a tag: rejected as too late for release
  configuration errors; path-filtered pull-request planning retains early
  feedback without charging ordinary code changes.

## Open Questions

None — ready for plan.
