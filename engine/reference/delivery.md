# Delivery Reference

Spec persistence modes, PR handoff, derived delivery state, and post-merge local
cleanup. Branch / commit / trailer / staging mechanics live in
`reference/git.md`; the living-spec fold and ADR mechanics live in
`reference/knowledge.md`; lifecycle states and gates live in
`reference/lifecycle.md`.

## Persistence modes

`.gitignore` is the single source of truth for whether specs are tracked.
`mochiflow accept {slug}` detects persistence from Git ignore behavior for the
concrete spec artifact: tracked mode means the spec artifact is not ignored;
local mode means `.mochiflow/` or the spec artifact path is ignored.

- **Tracked mode**: git includes the spec files. `mochiflow accept` stages the
  target spec directory and linked ADR record files with
  `git add {specs_dir}/{slug} {adr_record_paths...}`, validates the staged
  name-status output, and creates the accept close-out commit
  (`reference/git.md ## Accept close-out commit`).
- **Local mode**: `.mochiflow/` or the spec artifact path is ignored, so the
  worktree stays clean. `mochiflow accept` still runs final verification,
  validates lint / AC Matrix / reviewer verdict, updates local `spec.yaml` to
  `status: accepted`, then skips close-out commit, spec staging, and ADR staging
  with an explicit reason. It appends final verification evidence to
  already-`PASS` automated Matrix rows but does not convert `UNVERIFIED` rows to
  `PASS`, never stages `{index}` (`INDEX.md`), and never moves the spec. Local
  mode relies on local accepted state plus PR body evidence.

## PR

The PR title/body (per `templates/delivery/pr-description.md`: artifact
language, external-reviewer facing, no spec-internal references, no spec slug, no
AC IDs, no mochiflow vocabulary; includes verification evidence, review result,
and durable decision summary) are generated after human gate 2
(`reference/lifecycle.md`). Every spec depth uses `mochiflow pr` after acceptance
and approve-PR.

The open procedure uses **`mochiflow pr`** for PR handoff. The command runs
pre-flight, pushes the branch, resolves the backend, and reports its exit code
(`0` created, `10` manual handoff, `3` pre-flight FAIL, `1`/`2` failure).
Tracked mode pre-flight requires a clean tree, current branch as source,
source ≠ target, and a committed accepted spec at `{specs_dir}/{slug}/` with a
`Spec: {slug}` trailer. Local mode pre-flight does not require committed spec
artifacts or a `Spec:` trailer; it requires a clean tracked tree, current branch
as source, source ≠ target, source ahead of target, local accepted state, and
complete verification/review evidence.

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
manual handoff — the branch is pushed so the human can open the PR from it.

Duplicate-PR detection is provider-specific and is left to the driver / provider
CLI; `mochiflow pr`'s agnostic pre-flight does not perform it.

## Derived delivery state

Delivery state is observed, never stored. `mochiflow status` and the regenerated
`INDEX.md` compute it; `spec.yaml` keeps only the asserted states
`draft → approved → accepted` (`reference/lifecycle.md`).

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

## Delivery next actions

Delivery guidance is conversational and follows `[i18n].conversation_language`
(`reference/language.md` / `reference/presentation.md`). This covers the
PR-created next action (merge the PR, then report the merge in chat), the
in-review and `local cleanup pending` next-action hints in status / board
output, and the `close` start and completion wording. When
`conversation_language = auto`, resolve per the language rule; CLI-only output
(no live conversation context) falls back to `[i18n].artifact_language`
deterministically.

PR titles, PR descriptions, and other durable artifacts stay in
`[i18n].artifact_language`. The post-merge next action is therefore never written
into the PR body — it is local workflow guidance for the author, not review
material. Merge-report phrasings such as `merged` / `マージした` are illustrative
intent examples, not fixed trigger strings.

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
