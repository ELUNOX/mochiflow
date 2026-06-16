# Git Reference

Branch / commit / PR / fold / archive rules during mochiflow. Reviewer and
commit-granularity cadence are defined in `risk.md`; this file owns the git
mechanics and the living-spec fold.

## Branch

- Branch name `{prefix}/{slug}`; `prefix` is derived from `spec.yaml` `type` via
  the mapping below. `slug` is used as-is.
- Prefix mapping (Conventional Commits alignment):
  - `feature` → `feat`
  - all other types (`fix`, `refactor`, `docs`, `chore`) → used as-is
- If the current branch is already the target, do not switch.
- "Unrelated changes" is precise: any uncommitted change **other than this
  spec's own `{specs_dir}/{slug}/**`**. The spec files just authored by `plan`
  are *related* and expected to be present at build start — they never block.
  Any other dirt → stop instead of switching.
- Create from `origin/{[git].base_branch}` when the branch does not exist.
  Create the branch first and let `git switch -c` carry the uncommitted
  `{specs_dir}/{slug}/**` onto it (a fresh branch has no conflict; no stash
  needed), so the base branch HEAD is never dirtied by the spec scaffold.
- Trivial `risk: standard` changes MAY commit on the current branch with no new
  branch and no PR, only when the user opts in (no-PR fast path). Default is a
  feature branch + PR.

## Commit

Conventional Commits, in the project language.

```
type(scope): summary

body (optional)
```

- `type`: `feat|fix|refactor|docs|chore`, matching `spec.yaml` `type` (`feature` maps to `feat`).
- `scope`: `spec.yaml` `module` if present.
- Summary ≤ 50 chars. Body for large changes only. Never write spec slug, AC IDs, or
  mochiflow vocabulary (external-reviewer view).

## Auto-commit and staging

The AI auto-commits in all flows. Commit only after verification PASS (and
reviewer PASS when `risk.md` requires it); never commit on FAIL.

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
- The install dir and `state/` are typically gitignored; never `git add -f` them.
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
- the AC Verification Matrix rows added at ship (build already recorded the rest);
- the ADR fold (`[adr].decisions` / `[adr].pitfalls`);
- the archive move `{specs_dir}/{slug}/` → `{specs_dir}/_done/{slug}/`;
- the regenerated `{index}`.

Stage exactly these paths (`git add .` is still forbidden). The message follows
the Commit convention above — Conventional Commits, project language, and **no
spec slug, no AC IDs, no mochiflow vocabulary** (never "fold" / "archive" in the
summary). This relocates what was formerly a post-merge base-branch push into the
PR, so the durable record is reviewed; post-merge then does only local hygiene
(`## Post-merge local cleanup`). The no-PR fast path makes the same close-out
commit on the current branch, with no push.

## PR

The PR title/body (per `templates/delivery/pr-description.md`: project language,
external-reviewer facing, no spec-internal references, no spec slug, no AC IDs,
no mochiflow vocabulary) are always generated after human gate 2 (`workflow.md`).

PR creation goes through **`mochiflow pr`** — the single command that owns
pre-flight (working tree clean / current branch is the source / source ≠ target),
the one `git push`, and backend resolution. The AI never calls `git push` / `gh`
/ `az` directly; it runs `mochiflow pr` and reads the exit code (`0` created,
`10` manual handoff, `3` pre-flight FAIL, `1`/`2` failure).

`mochiflow pr` resolves the creation backend in precedence order:

- **`[git].pr_driver`** — a custom executable implementing the pr-request
  contract: invoked as `<driver> <request-dir>`, reads `pr-request.json`
  (`contracts/pr-request.schema.json`), prints `{"url": "..."}`. For providers/
  auth not covered by a built-in (e.g. an enterprise provider + secret-store PAT).
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
responsibility move, technology/verification change), run `refresh-context` (`commands/refresh-context.md`)
instead of editing it inline; code remains the source of truth.

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
4. `git branch -d {type}/{slug}` (safe delete; fails if unmerged → leave it, ask human).
5. Do not touch the remote branch.
6. Remove the spec's ephemeral delivery scratch: `rm -rf {install_dir}/state/{slug}/` (gitignored — PR body / `pr-request.json` / `qa-instructions.md` are not archived).

The fold + archive (`_done` move + `INDEX`) are **not** performed here — they are
part of the feature branch's close-out commit (`## Living-spec fold`,
`## Auto-commit and staging`). The no-PR fast path commits the fold + archive
locally on the current branch right after verification, with no base-branch push.

## Safety

- One git command per call; no `&&` / `;` / `||` / `|` chaining.
- `git push --force` / `-f` forbidden. `git reset --hard` / `git clean -f` /
  `git branch -D` require human judgement.
- Keep pre-commit hooks; `--no-verify` only on explicit human instruction.
- Do not change `git config`. Amend only your own un-pushed commits.
