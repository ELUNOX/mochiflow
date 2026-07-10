# Git Reference

Branch, commit, trailer, and staging mechanics during mochiflow. Reviewer
cadence lives in `reference/review.md`; PR handoff, derived delivery state, and
post-merge cleanup live in `reference/delivery.md`; the living-spec fold and ADR
mechanics live in `reference/knowledge.md`. Specs stay flat at
`{specs_dir}/{slug}/` for their whole life — there is no `_done/` move and no
committed board.

## Branch

- Branch name `{branch}` resolves to `{prefix}/{slug}`. `prefix` is derived from
  `spec.yaml` `type` via the mapping below; `slug` is used as-is.
- Prefix mapping (Conventional Commits alignment):
  - `feature` → `feat`
  - all other types (`fix`, `refactor`, `docs`, `chore`) → used as-is
- If the current branch is already `{branch}`, do not switch.
- "Unrelated changes" is precise: any uncommitted change **other than this
  spec's own `{specs_dir}/{slug}/**`**. During discuss, the matching raw seed
  deletion at `{specs_dir}/_backlog/{slug}.md` is also related because seed
  promotion is committed atomically. The spec files just authored by discuss /
  plan are related and expected to be present before the phase commit — they
  never block their own phase. Any other dirt → stop instead of switching.
- Discuss creates `{branch}` from `origin/{[git].base_branch}` when agreement is
  reached, and warns when the local base branch is behind
  `origin/{[git].base_branch}` (`commands/discuss.md`); never branch from a
  stale local base. Direct micro plan creates or switches `{branch}` from
  `origin/{[git].base_branch}` after metadata confirmation and before the draft
  micro commit. Other plan/build/open/update flows use the existing branch;
  build must error-stop if it cannot find or switch to `{branch}`. Every spec
  depth delivers through the feature branch + PR path.

## Commit

Conventional Commits, in `[i18n].artifact_language`.

```
type(scope): summary

body (optional)
```

- `type`: `feat|fix|refactor|docs|chore`, matching `spec.yaml` `type` (`feature` maps to `feat`).
- `scope`: `spec.yaml` `module` if present.
- Summary ≤ 50 chars. Body for large changes only.
- Summary: never write spec slug, AC IDs, or mochiflow vocabulary
  (external-reviewer view).
- Body: slug may appear as natural context (e.g. "implements the refresh logic
  from oauth-refresh-flow"). Avoid AC IDs and mochiflow vocabulary (`fold`,
  `open`, `build phase`, etc.) in the body. Body must not begin a line with `Spec:`
  (reserved for trailer parsing; see `## Trailers`).
- Trailers are metadata (same category as `Signed-off-by`); `Spec:` and `Task:`
  trailers are required per `## Trailers` below.

## Trailers

Git trailers provide machine-parseable traceability from commits to specs and
tasks. They go in the commit footer (after the body, separated by a blank line).

```
type(scope): summary

body (optional)

Spec: {slug}
Task: T-001
Task: T-002
```

### Rules

- `Spec: {slug}` — **required** on every spec-lane commit (discuss, plan, build,
  open, update). The value is the spec's `slug` from `spec.yaml`. A merge must
  leave the `Spec: {slug}` trailer reachable from the base branch: merge/rebase
  preserve it automatically; a squash merge must carry it into the squash commit
  (it is the local-git `merged` derivation signal).
- `Task: T-XXX` — **required** when `tasks.md` exists and the commit completes a
  specific task. Normal build commits complete one task and use one `Task:`
  line. Multiple `Task:` lines are kept for compatibility with existing history
  and exceptional reconciliation commits. **Optional** on the accept close-out
  commit (which bundles multiple concerns).
- `Spec:` and `Task:` keys are case-sensitive and use a single space after the
  colon.

### External-reviewer compatibility

Trailers do not appear in `git log --oneline`, `git shortlog`, or GitHub PR
subject views. They are visible in full commit messages and are useful metadata
for any reviewer.

## AI Git Log Recipes

Reusable git commands for querying spec/task traceability. Use `--grep` for
speed, then `%(trailers:...)` format for accurate extraction.

```bash
# All commits for a spec (fast grep + trailer display)
git log --grep="Spec: {slug}" \
  --format="%H %s%n  Spec: %(trailers:key=Spec,valueonly,separator=%x2C )%n  Task: %(trailers:key=Task,valueonly,separator=%x2C )"

# Last completed task for a spec
git log --grep="Spec: {slug}" \
  --format="%(trailers:key=Task,valueonly)" | grep -m1 .

# Recent changes to a file with spec context
git log --format="%s | %(trailers:key=Spec,valueonly)" -- path/to/file -5
```

## Auto-commit and staging

The AI auto-commits in all flows. Commit only after verification PASS; never
commit on verification FAIL. When `reference/review.md` requires a reviewer verdict,
reviewer PASS is a phase-completion / acceptance gate, not a pre-commit gate.
If reviewer FAIL findings require changes, fix them, verify, and commit the
follow-up before build completes.

- Stage files in the change plan / task plus tests added for verification.
- Stage this spec's own files under `{specs_dir}/{slug}/**` together with the
  change, as part of build's first task commit when `tasks.md` exists, or as
  part of the single logical-unit build commit for taskless / micro specs.
  `.gitignore` is the single source of truth for whether specs are tracked: when
  the project tracks specs, git includes these files; when the project gitignores
  `{specs_dir}/{slug}/`, git skips them and the worktree was already clean.
  Close-out ADR record files under `[adr]` (`decisions/` / `pitfalls/`) are
  committed only when they are linked to this spec's slug; each store's
  generated `INDEX.md` is not.
- `state/` is gitignored, and the generated board `{index}` (`INDEX.md`) plus
  each ADR store's `INDEX.md` are gitignored and **never staged or committed** by
  any command (the bare `INDEX.md` ignore pattern matches `adr/**/INDEX.md`). The
  vendored engine under the install dir is tracked by default.

### Spec-lane lifecycle commits

| phase | branch action | commit content |
| --- | --- | --- |
| discuss | create/switch `{prefix}/{slug}` from `origin/{base_branch}` | `spec.yaml (draft)`, `pitch.md`, optional `_backlog/{slug}.md` deletion |
| plan | use existing `{prefix}/{slug}`; direct micro may create/switch `{prefix}/{slug}` from `origin/{base_branch}` before the draft commit | `spec.yaml (approved)`, `spec.md`, optional `design.md` / `tasks.md`, optional corrected `pitch.md`; direct micro first commits `spec.yaml (draft)` + `spec.md` |
| build | verify/switch existing `{prefix}/{slug}`; never create it | implementation, tests, task checkbox updates, AC Matrix updates |
| open | use existing `{prefix}/{slug}` | optional `docs(context)` commit (regenerated `[context]` files) when a structural shift was detected, then the close-out commit: `status: accepted`, final AC Matrix, ADR fold (flat spec, no `_done/` move, no `INDEX` write) |
| update | use existing `{prefix}/{slug}` | PR-feedback fixes as bounded inline code changes; the fold revised when a decision changes |
| close | local hygiene on base | nothing committed/pushed to the base branch |

Discuss and plan use `docs(spec): ...` commit subjects plus the required
`Spec: {slug}` trailer. Build uses the spec's Conventional Commit type and
`Task:` trailers when tasks complete. When `open` detects a coarse structural
shift, it makes a separate `docs(context): ...` commit (regenerated `[context]`
files only, with the `Spec: {slug}` trailer — a spec-lane commit) **before** the
accept close-out commit, so the refresh ships in the PR while the close-out stays
the single final state commit. open then follows `### Accept close-out commit`.

**open QA-`FAIL` rework / update PR-feedback commits** (the bounded inline
code changes in `commands/open.md` step 3e and `commands/update.md` step 2) are
ordinary feature-branch fix commits: a Conventional Commit subject describing the
fix, the required `Spec: {slug}` trailer, and **no** `Task:` trailer (build is
complete, so there is no `tasks.md` task to reference) and no checkbox tick. They
are separate from open's single accept close-out commit (which carries
`status: accepted` + the fold) and are not amended into it.

Every spec-lane procedure commit step (discuss / plan / build / open / update /
close) regenerates the board via `mochiflow index` so the gitignored `INDEX.md`
stays fresh between CLI commands.

### Accept close-out commit

`open` produces one **close-out commit** on the feature branch in tracked mode,
after human QA and **before** `mochiflow pr`. When `open`'s step-4
context-refresh check ran (a
coarse structural shift was detected and the human confirmed the regenerated
context), a separate `docs(context)` commit — carrying only the `[context]` files
with the `Spec: {slug}` trailer — is created **first**, after the
fold/context-check and **before** this close-out commit; the close-out commit
remains the single final state commit and `mochiflow pr` pre-flight still sees a
clean tree. The close-out commit bundles, in a single commit:

- `spec.yaml` `status: accepted` (+ `updated`); never `done`, never `completed`;
- the human QA Matrix rows added at open and final verification evidence appended
  by `mochiflow accept` to already-`PASS` automated rows (build already recorded
  final automated results);
- the selected ADR fold records (`[adr].decisions` / `[adr].pitfalls`) linked
  to this spec's slug.

The spec stays flat: there is no `_done/` move and no `INDEX` regeneration in the
commit.

Use `mochiflow accept {slug}` for the deterministic mechanics. The command
detects persistence from Git ignore behavior for the concrete spec artifact:
tracked mode means the spec artifact is not ignored; local mode means
`.mochiflow/` or the spec artifact path is ignored. In tracked mode it stages the
target spec directory and linked ADR record files with
`git add {specs_dir}/{slug} {adr_record_paths...}`, validates the staged
name-status output, and creates the close-out commit. In local mode it still runs
final verification, validates lint / AC Matrix / reviewer verdict, updates local
`spec.yaml` to `status: accepted`, then skips close-out commit, spec staging, and
ADR staging with an explicit reason. It appends final verification evidence to
already-`PASS` automated Matrix rows, but it does not convert `UNVERIFIED` rows
to `PASS`. It never stages `{index}` (`INDEX.md`) and never moves the spec. If
manual fallback is required, use the same stable pathspecs only in tracked mode
and validate with `git diff --cached --name-status -z`; never stage `INDEX.md`
and never force-add ignored `.mochiflow/` artifacts.

The message follows
the Commit convention above — Conventional Commits, artifact language, and **no
spec slug, no AC IDs, no mochiflow vocabulary** (never "fold" in the summary).
This keeps the judgment-bearing durable record (the fold) inside the PR, under
review, so it merges atomically with the code; `close` then does only local
hygiene (`reference/delivery.md ## Post-merge local cleanup`).
