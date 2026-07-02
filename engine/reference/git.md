# Git Reference

Branch / commit / PR / fold rules during mochiflow. Reviewer cadence and
integration-log requirements are defined in `risk.md`; this file owns the git
mechanics and the living-spec fold. Specs stay flat at `{specs_dir}/{slug}/` for
their whole life — there is no `_done/` move and no committed board.

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
commit on verification FAIL. When `risk.md` requires a reviewer verdict,
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

`open` produces one **close-out commit** on the feature branch, after human QA and
**before** `mochiflow pr`. When `open`'s step-4 context-refresh check ran (a
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

Use `mochiflow accept {slug}` for the deterministic mechanics: it stages the
target spec directory and linked ADR record files with
`git add {specs_dir}/{slug} {adr_record_paths...}`, validates the staged
name-status output, and creates the close-out commit. It appends final
verification evidence to already-`PASS` automated Matrix rows, but it does not
convert `UNVERIFIED` rows to `PASS`. It never stages `{index}` (`INDEX.md`) and
never moves the spec. If manual fallback is required, use the same stable pathspecs and validate with
`git diff --cached --name-status -z`; never stage `INDEX.md`. When specs are
gitignored there may be nothing to stage under `{specs_dir}`.

The message follows
the Commit convention above — Conventional Commits, artifact language, and **no
spec slug, no AC IDs, no mochiflow vocabulary** (never "fold" in the summary).
This keeps the judgment-bearing durable record (the fold) inside the PR, under
review, so it merges atomically with the code; `close` then does only local
hygiene (`## Post-merge local cleanup`).

## PR

The PR title/body (per `templates/delivery/pr-description.md`: artifact
language, external-reviewer facing, no spec-internal references, no spec slug, no
AC IDs, no mochiflow vocabulary) are generated after human gate 2
(`workflow.md`). Every spec depth uses `mochiflow pr` after acceptance and
approve-PR.

The open procedure uses **`mochiflow pr`** for PR handoff. The command runs
pre-flight (working tree clean / current branch is the source / source ≠ target /
the spec committed at `{specs_dir}/{slug}/` with `status: accepted` and a
`Spec: {slug}` trailer), pushes the branch, resolves the backend, and reports its
exit code (`0` created, `10` manual handoff, `3` pre-flight FAIL, `1`/`2`
failure).

`mochiflow pr` resolves the creation backend in precedence order:

- **`[git].pr_driver`** — a custom executable implementing the pr-request
  contract: invoked as `<driver> <request-dir>`, reads `pr-request.json`
  (the repo-level CLI contract at `contracts/pr-request.schema.json`), prints
  `{"url": "..."}`. For providers/auth not covered by a built-in (e.g. an
  enterprise provider + secret-store PAT).
  The request-dir is `{install_dir}/state/{slug}/` (gitignored), where
  `mochiflow pr` writes `pr-request.json` — only for this driver backend; the
  schema is unchanged, only its location moved out of the tracked spec tree.
- **`[git].provider` built-in** — a maintained provider integration. `github`
  shells out to `gh`. (gitlab / azure-devops are additive, not yet built in.)
- **legacy `[git].pr_command`** — a raw command string (deprecated). Run via the
  shell with `{spec_dir}` substituted, after `mochiflow pr` has already done
  pre-flight + push. Kept for backward compatibility; prefer `pr_driver`.
- **manual handoff** — nothing configured (the zero-config default). `mochiflow
  pr` still runs pre-flight and pushes the branch, then presents the PR content
  and hands off: the human creates the PR via their provider UI/CLI and reports
  the URL / merge. This is a first-class default, not an "incomplete" state.

Note: `git push` now happens inside `mochiflow pr` for **all** modes including
manual handoff — the branch is pushed so the human can open the PR from it. (This
supersedes the earlier rule that manual handoff performed no push.)

Duplicate-PR detection is provider-specific and is left to the driver / provider
CLI; `mochiflow pr`'s agnostic pre-flight does not perform it.

## Derived delivery state

Delivery state is observed, never stored. `mochiflow status` and the regenerated
`INDEX.md` compute it; `spec.yaml` keeps only the asserted states
`draft → approved → accepted`.

- `in_review` — a PR is open (provider reports it, or `provider = none` and the
  spec branch is pushed to `origin` and unmerged).
- `merged` — derived in priority order: the provider API when configured and
  available, else a tracked-mode `Spec: {slug}` trailer reachable from
  `origin/{[git].base_branch}`, else for local mode only the local source branch
  tip reachable from `origin/{[git].base_branch}`. The human merge report only
  initiates `close` locally and is never persisted as a merged signal.
  Provider-none local mode has one limitation: if the source branch is deleted
  before `close`, the branch-tip signal is gone and Done may no longer be
  derivable without provider state.

## Living-spec fold (on the feature branch, before `mochiflow pr`)

The fold happens **on the feature branch as part of the single close-out commit**
(see `## Auto-commit and staging`), created before `mochiflow pr` — never as a
post-merge push to the base branch. This keeps the judgment-bearing durable
record inside the PR, under review. Fold only knowledge that **code cannot
reproduce**, as dated historical records — never as a "current state" description
(current state is always derived from code):

- The *why* behind design decisions / contracts (why a new type, schema shape,
  ownership, registry rule, or persistence model was chosen, and which
  alternatives were rejected) → add a new per-file record under `[adr].decisions`
  named `{YYYY-MM-DD}-{slug}.md` with front-matter (`id`, `date`, `area`
  defaulting to the spec's `surfaces`, `spec: {slug}`, `status: active`). Write it as a
  fact *as of that date*; never rewrite an existing record. When a decision
  overrides an earlier one, add the new record with `supersedes: <id>` and flip
  the old record to `status: superseded` with the reciprocal `superseded_by:
  <id>` (status/link change only — its body stays immutable).
- Operational pitfalls found during implementation (to prevent recurrence) →
  a new per-file record under `[adr].pitfalls`, using `Applies to`, `Signal`,
  `Cause`, `Guardrail`, `Check`, and `Status`. Resolved pitfalls flip to
  `status: resolved` rather than being deleted.

Each store keeps a generated, gitignored `INDEX.md` content catalog; regenerate
it after adding a record and **never stage it** (consistent with the board
`INDEX.md`). `mochiflow accept {slug}` stages only ADR record files linked to
that slug, plus reciprocal supersession records; the per-store `INDEX.md` files
are not staged.

Do not fold prose that describes current state ("how the system is put together
now", "where things live"). The context layer (`[context].product` /
`[context].structure` / `[context].tech`) is **not** a fold target — it is a current-state
orientation map regenerated from code via onboard / `refresh-context`, never
appended to during fold. For coarse code-layout changes (new module,
responsibility move, technology/verification change) detected during `open`, run
`refresh-context` (`commands/refresh-context.md`) on the feature branch under
human confirmation and ship the regenerated context **inside the PR** as a
separate `docs(context)` commit placed after the fold/context-check and before
the accept close-out commit (see `## Auto-commit and staging`); code remains the
source of truth and the refresh is never folded. Running the refresh in-branch
before the PR is the primary path — never a post-merge base-branch edit. Context
staleness discovered only **at or after merge** is the fallback: route it to a
follow-up (a `fix` spec when it carries a code change, or a backlog seed for
later `discuss`) rather than a base-branch edit.

Fold is skipped when the change yields no new rationale or pitfall (e.g. a trivial
display fix). Do not create the close-out commit until the fold (or the decision
that none is needed) is done.

Knowledge discovered **at or after merge** is not appended to the merged spec
(that would re-introduce an unreviewed base-branch edit). Route it to a
follow-up: a small `fix` spec when it carries a code change, or a backlog seed
when it is pure rationale/pitfall for a later `discuss`.

## Post-merge local cleanup

When the human confirms merge (「完了」/「マージ済み」/「merged」), in the same
session — **local git hygiene only; no content commit or push to the base
branch** (the fold and the spec were already merged via the PR's close-out
commit):

1. `git status --short` clean — else stop.
2. `git switch {[git].base_branch}`
3. `git pull --ff-only origin {[git].base_branch}` — stop if ff-only fails (divergent local).
4. `git branch -d {prefix}/{slug}` (safe delete; fails if unmerged → leave it, ask human). Resolve `prefix` from `type`: `feature` → `feat`; all other types use `type` as-is. For provider-none local mode, delete the branch only here, after the merge report, because its tip is the local merge signal.
5. Remote branch cleanup is outside post-merge local cleanup.
6. Remove the spec's ephemeral delivery scratch: `rm -rf {install_dir}/state/{slug}/` (gitignored — PR body / `pr-request.json` are not archived).
7. Regenerate the board (`mochiflow index`); `INDEX.md` is gitignored and never staged.

Nothing is committed or pushed to the base branch here — the fold and the spec
already merged via the PR's close-out commit, so `close` is local hygiene only.
The spec is never moved into `_done/`; its merged state is observed (derived),
not written.
