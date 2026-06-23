# Git Reference

Branch / commit / PR / fold / archive rules during mochiflow. Reviewer and
commit-granularity cadence are defined in `risk.md`; this file owns the git
mechanics and the living-spec fold.

## Branch

- Branch name `{branch}` resolves to `{prefix}/{slug}`. `prefix` is derived from
  `spec.yaml` `type` via the mapping below; `slug` is used as-is.
- Prefix mapping (Conventional Commits alignment):
  - `feature` → `feat`
  - all other types (`fix`, `refactor`, `docs`, `chore`) → used as-is
- If the current branch is already `{branch}`, do not switch.
- "Unrelated changes" is precise: any uncommitted change **other than this
  spec's own `{specs_dir}/{slug}/**`**. The spec files just authored by `plan`
  are *related* and expected to be present at build start — they never block.
  Exception: only when returning from `ship.md ## PR Feedback Loop`, the restore
  from the archived shipped spec is also related, so the allowed dirty paths are
  exactly `{specs_dir}/{slug}/**` and `{specs_dir}/_done/{slug}/**`. Other slugs
  under `_done/`, other specs, source changes, and `state/` files remain
  unrelated dirt. Any other dirt → stop instead of switching.
- Create from `origin/{[git].base_branch}` when the branch does not exist.
  Create the branch first and let `git switch -c` carry the uncommitted
  `{specs_dir}/{slug}/**` onto it (a fresh branch has no conflict; no stash
  needed), so the base branch HEAD is never dirtied by the spec scaffold.
- Trivial `risk: standard` changes MAY use the current branch with no new branch
  and no PR only when the user explicitly opts in (no-PR fast path). Default is
  a feature branch + PR. no-PR skips PR creation and the approve-PR gate, but it
  still runs `ship` acceptance and the close-out commit.

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
  from oauth-refresh-flow"). AC IDs and mochiflow vocabulary (`fold`, `ship`,
  `build phase`, etc.) remain forbidden. Body must not begin a line with `Spec:`
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
  ship). The value is the spec's `slug` from `spec.yaml`.
- `Task: T-XXX` — **required** when `tasks.md` exists and the commit completes a
  specific task. Use one `Task:` line per task (multiple lines for multi-task
  commits). **Optional** on ship close-out commits (which bundle multiple
  concerns).
- Patch lane commits have **no trailers** (no spec context exists).
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
reviewer PASS is a phase-completion / ship-acceptance gate, not a pre-commit gate.
If reviewer FAIL findings require changes, fix them, verify, and commit the
follow-up before build completes.

- Stage only files in the change plan / task plus tests added for verification.
- Stage this spec's own files under `{specs_dir}/{slug}/**` together with the
  change, as part of build's **first** commit (folded into the single commit for
  `standard`, or the first commit — `docs(scope): ...` or alongside task 1 — for
  `elevated` / `critical`). `.gitignore` is the single source of truth for
  whether specs are tracked: when the project tracks specs, git includes these
  files; when the project gitignores `{specs_dir}/{slug}/`, git skips them and
  the worktree was already clean. Never `git add -f` to override either way. The
  ADR under `[adr]` is committed regardless of this choice.
- `git add .` is forbidden — name files explicitly.
- `state/` is gitignored; never `git add -f` it. The vendored engine under the
  install dir is tracked by default.
- Do not stage `.env`, `.env.*`, or credential files; warn if encountered.

### Patch commit

Patch runs on the current branch with no new branch, no PR, no spec archive, and
no living-spec fold.

At patch start, run `git status --short` and identify the intended target files.
If any target file is already dirty, patch may edit it only without overwriting
the existing changes, and auto-commit is disabled for the patch. Unrelated dirty
files are ignored and never staged.

After verification PASS, stage only the patch's changed files explicitly and
create one Conventional Commit per `## Commit`. Do not stage spec files,
`[constitution]`, `[context]`, `[adr]`, `{index}`, `{install_dir}/state/**`, `.env*`, or
unrelated dirty files. If verification could not be run, or any target file was
dirty before the patch, do not commit; report the files and verification result.

### Ship close-out commit

`ship` produces one **close-out commit** on the feature branch, after human QA and
**before** `mochiflow pr`. It bundles, in a single commit:

- `spec.yaml` `status: done` (+ `updated`);
- the AC Matrix rows added at ship (build already recorded the rest);
- the ADR fold (`[adr].decisions` / `[adr].pitfalls`);
- the archive move `{specs_dir}/{slug}/` → `{specs_dir}/_done/{slug}/` (use
  `git mv {specs_dir}/{slug}/ {specs_dir}/_done/{slug}/` so both the deletion and
  the addition are staged as a rename; when specs are gitignored there is nothing
  to stage);
- the regenerated `{index}`.

Stage exactly these paths (`git add .` is still forbidden). The message follows
the Commit convention above — Conventional Commits, artifact language, and **no
spec slug, no AC IDs, no mochiflow vocabulary** (never "fold" / "archive" in the
summary). This relocates what was formerly a post-merge base-branch push into the
PR, so the durable record is reviewed; post-merge then does only local hygiene
(`## Post-merge local cleanup`). The no-PR fast path makes the same close-out
commit on the current branch, with no push.

## PR

On the normal PR path, the PR title/body (per
`templates/delivery/pr-description.md`: artifact language, external-reviewer
facing, no spec-internal references, no spec slug, no AC IDs, no mochiflow
vocabulary) are generated after human gate 2 (`workflow.md`). On the explicit
no-PR fast path, skip PR title/body generation and `mochiflow pr`; `ship`
acceptance and the close-out commit still happen.

PR creation goes through **`mochiflow pr`** — the single command that owns
pre-flight (working tree clean / current branch is the source / source ≠ target),
the one `git push`, and backend resolution. The AI never calls `git push` / `gh`
/ `az` directly; it runs `mochiflow pr` and reads the exit code (`0` created,
`10` manual handoff, `3` pre-flight FAIL, `1`/`2` failure).

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

## Living-spec fold (on the feature branch, before `mochiflow pr`)

The fold happens **on the feature branch as part of the single close-out commit**
(see `## Auto-commit and staging`), created before `mochiflow pr` — never as a
post-merge push to the base branch. This keeps the judgment-bearing durable
record inside the PR, under review. Fold only knowledge that **code cannot
reproduce**, as dated historical records — never as a "current state" description
(current state is always derived from code):

- The *why* behind design decisions / contracts (why a new type, schema shape,
  ownership, registry rule, or persistence model was chosen, and which
  alternatives were rejected) → append to the Decisions Log in
  `[adr].decisions` as `### {YYYY-MM-DD} {slug}`. Write it as a fact *as of
  that date*; never rewrite existing entries.
- Operational pitfalls found during implementation (to prevent recurrence) →
  `[adr].pitfalls`, using `Applies to`, `Signal`, `Cause`, `Guardrail`, `Check`,
  and `Status`.

Do not fold prose that describes current state ("how the system is put together
now", "where things live"). The context layer (`[context].product` /
`[context].structure` / `[context].tech`) is **not** a fold target — it is a current-state
orientation map regenerated from code via onboard / `refresh-context`, never
appended to during fold. For coarse code-layout changes (new module,
responsibility move, technology/verification change), flag a post-ship
`refresh-context` (`commands/refresh-context.md`) follow-up instead of editing it
inline or running it during close-out; code remains the source of truth. Context
refresh is separate work after PR creation / merge unless the human explicitly
runs and commits it as a separate change later.

Fold is skipped when the change yields no new rationale or pitfall (e.g. a trivial
display fix). Do not archive until the fold (or the decision that none is needed)
is done.

Knowledge discovered **at or after merge** is not appended to the already-archived
spec (that would re-introduce an unreviewed base-branch edit). Route it to a
follow-up: a small `fix` spec when it carries a code change, or a backlog seed
when it is pure rationale/pitfall for a later `discuss`.

## Post-merge local cleanup

When the human confirms merge (「完了」/「マージ済み」/「merged」), in the same
session — **local git hygiene only; no content commit or push to the base
branch** (the fold + archive `_done` move + `INDEX` were already merged via the
PR's close-out commit):

1. `git status --short` clean — else stop.
2. `git switch {[git].base_branch}`
3. `git pull --ff-only origin {[git].base_branch}` — stop if ff-only fails (divergent local).
4. `git branch -d {prefix}/{slug}` (safe delete; fails if unmerged → leave it, ask human). Resolve `prefix` from `type`: `feature` → `feat`; all other types use `type` as-is.
5. Do not touch the remote branch.
6. Remove the spec's ephemeral delivery scratch: `rm -rf {install_dir}/state/{slug}/` (gitignored — PR body / `pr-request.json` are not archived).

The fold + archive (`_done` move + `INDEX`) are **not** performed here — they are
part of the feature branch's close-out commit (`## Living-spec fold`,
`## Auto-commit and staging`). The no-PR fast path makes that same close-out
commit locally on the current branch after `ship` acceptance, with no
base-branch push.

## Safety

- One git command per call; no `&&` / `;` / `||` / `|` chaining.
- `git push --force` / `-f` forbidden. `git reset --hard` / `git clean -f` /
  `git branch -D` require human judgement.
- Keep pre-commit hooks; `--no-verify` only on explicit human instruction.
- Do not change `git config`. Amend only your own un-pushed commits.
