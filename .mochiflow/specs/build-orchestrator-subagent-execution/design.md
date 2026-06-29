# Run build as an orchestrator dispatching disposable per-task workers — Design

> risk: elevated · integration: workflow · surface: cli. Decisions and contracts
> only; read source during build.

## Design Decisions

- **Shared transport, distinct roles (not a separate worker transport).** The
  delegation transport is the selection discipline "prefer a delegated subagent
  when the runtime supports it, else run inline". It already exists for review
  (`risk.md ## Review transport`). Generalize that section into a shared
  delegation transport that both roles use. The *role* (system prompt + tool
  scope + permissions) is what differs: the reviewer is read-only; the worker is
  write + verify + commit. Rejected a fully independent worker transport: it
  duplicates the selection / fallback / tool-agnostic logic (SSOT violation) and
  the clean-context separation that makes a generator-verifier loop work is
  already obtained by keeping the roles distinct under one transport.
- **Transcript isolation, repo-wide read, contract-bounded write.** The bloat
  being removed is the orchestrator's accumulated history, not filesystem access.
  A worker reads the repo freely (it must, to implement correctly) but may only
  write within its task's declared surface; an out-of-scope edit is a `blocked`
  return, reusing build's existing stop condition. Discovery (e.g. an `rg` sweep
  for adjacent files) runs inside the worker and is discarded on return.
- **git is the accumulator for review.** The orchestrator never holds the
  cumulative diff; the per-task commits do. The mandatory review reconstructs the
  diff from git and reads code from scratch; compact reports are never review
  evidence. This extends `risk.md`'s "never conversation history as evidence".
- **Principle 5 refined, not relaxed.** Split the fused sentence into the
  invariant (judgment / gates / integration / fold single-threaded) and the
  execution transport (inline or delegated). Reviewer is no longer "the only
  separated procedure"; review and per-task build execution are the two delegated
  procedures over one transport.
- **Delegation threshold decoupled from risk.** Orchestrator mode triggers on
  `>= 2` open tasks in `tasks.md`; `risk` keeps owning reviewer cadence only. A
  single-task elevated spec gains nothing from isolation.
- **No downgrade in v1.** Workers run on the top model. Quality equals today's
  inline build; the only change is where execution runs.
- **Worker-recoverability is an authoring rule, not a lint.** Whether a fact is
  recoverable from `design.md` + the task row + committed code cannot be decided
  mechanically (consistent with this repo's "leave non-mechanical checks to
  review" stance). It lives in `plan.md` / `authoring.md` and is enforced by
  reviewer Stage 1 judgment.
- **This supersedes the Kiro-adapter "exactly two files" decision.** ADR
  `2026-06-24-kiro-adapter-always-on-steering` states the Kiro adapter generates
  exactly two files (steering + the read-only reviewer agent). Adding the
  write-capable `spec-worker.json` (AC-12) intentionally makes it three. The
  conflict is deliberate and bounded: the new agent follows the same generated,
  per-call-permission model as the reviewer agent (no baked tool policy, no
  `toolsSettings` — the concern that motivated the original ADR), so the
  rationale of that ADR is preserved while its "exactly two" count is updated.
  `open`'s fold adds a new decision record under `[adr].decisions` with
  `supersedes: 2026-06-24-kiro-adapter-always-on-steering`, flipping that record
  to `superseded`; no engine doc is rewritten to hide the older rationale.

## Architecture

Build execution model when orchestrator mode is active:

```
build orchestrator (top model, main thread; holds plan/contract only)
  if tasks.md has >= 2 open tasks AND a subagent mechanism exists:
    for each open task T in dependency order (ONE AT A TIME):
      assemble context pack(T)
      dispatch worker(T) over the shared delegation transport:
        worker: fresh context = context pack; repo-wide read; contract-bounded write
        worker implements T, runs the default verification, marks the tasks.md
          checkbox, commits with one `Task:` trailer
        worker returns ONLY a compact report
      orchestrator records the report; settles the AC Matrix row(s) at build
        completion (not per task — see write ownership below)
      if risk == critical: run independent-reviewer on T's own git commit
        (the per-task cadence from risk.md) BEFORE advancing
      advance to next T
    run final verification once
    if risk == elevated: run the completion-gate review on the
      git-reconstructed full diff (`git diff origin/{base}...HEAD`)
      (critical's per-task reviews above are its entire cadence; standard has
       no reviewer)
  else:
    run today's inline build task loop unchanged
```

- The shared transport selection (`delegated` → `inline`) is reused verbatim from
  the reviewer path; only the dispatched role changes.
- Sequential only: one worker at a time on the single working tree; commit, then
  next. No `[P]` parallelism, no worktree.
- **Reviewer cadence is unchanged from `risk.md`.** `standard` runs no reviewer
  (the safeguard is the worker's deterministic verification plus the
  orchestrator's final `default` verification); `elevated` runs once after all
  tasks; `critical` runs after each task. Delegation only changes *where the diff
  comes from* (git), never *how often* review happens.
- **Write ownership (no double-write to `tasks.md`).** A worker owns its task's
  checkbox tick and the per-task code commit. The orchestrator owns the AC Matrix
  rows, which it records once at build completion (per `build.md` step 7), not per
  task. Because execution is sequential and the AC Matrix is settled at the end,
  the worker (checkbox) and the orchestrator (Matrix) never write `tasks.md` in
  the same step; the resume-reconciliation source stays `tasks.md` checkboxes +
  `Task:` trailers.

## Data Model / Interfaces

- **Context pack (orchestrator → worker), as a dispatch prompt — not a schema
  file):**
  - the relevant `design.md` slice (the shared contract) as the **start point**;
    the worker reads the full `design.md` via repo-wide read when it needs more,
  - the single `tasks.md` row for T (`Files` / `Done` / `Stop` / AC refs),
  - the surface's `default` verify command,
  - pointers to `[constitution]`, `reference/engineering-standards.md`, and
    relevant `[adr].pitfalls`.
  - Excludes: other tasks' transcripts, conversation history. `Files` is the
    write-scope anchor and reading start point, not a read jail.
- **Compact report (worker → orchestrator):**
  - `task`: T-### id,
  - `status`: `done` | `blocked`,
  - `files_changed`: list of paths,
  - `verify`: profile + `PASS` | `FAIL` + evidence pointer (command / output
    location),
  - `commit`: commit ref (present when `status = done`),
  - `reason`: required when `status = blocked` (out-of-scope / new design
    decision / verification keeps failing).
  - Excludes the implementation narrative. The orchestrator settles the AC Matrix
    row(s) for T from this report alone.
- **kiro worker agent (`.kiro/agents/spec-worker.json`):** modeled on
  `spec-independent-reviewer.json` but write-capable —
  `tools: ["read","grep","glob","edit","write","bash"]`, `model` = the top model
  used by the reviewer, `prompt` = `engine/agents/worker.md`, resources include
  `worker.md` + the workflow/git/language references it needs. Generated from a
  new `engine/adapters/kiro/agents/spec-worker.json.tpl` + a `manifest.toml`
  entry; `adapter.rs is_kiro_agent_json` is extended to treat it as a full-file
  managed agent.

## Error Handling

- **Inline fallback** when no subagent mechanism is available or dispatch fails
  for a runtime/tooling reason — identical to today's inline build (no behavior
  change, no data loss).
- **STOP / blocked**: a worker that hits a build stop condition returns
  `blocked: <reason>` and does not improvise; the orchestrator stops the loop and
  routes back to plan (preserves judgment-single-threaded at runtime).
- **Report over-claim**: the worker runs deterministic verification, and the
  safeguard scales with `risk`. For `standard` (no reviewer), the net is the
  worker's verification plus the orchestrator's final `default` verification.
  For `risk >= elevated`, the mandatory review additionally reads the real diff
  from git — so a mistaken/over-claimed PASS is caught there, never trusted from
  the report.
- **Resume mid-orchestration**: state is `tasks.md` checkboxes + `Task:` trailers
  in git; the existing build resume reconciliation applies unchanged.

## Test Strategy

- Conformance tests over engine doc content: router principle 5 refined wording;
  build.md orchestrator/worker procedure incl. the `>= 2` threshold, sequential
  rule, context pack, compact report, worker commit cadence, inline fallback;
  `risk.md` shared transport + reports-not-evidence + full-diff-from-git;
  worker.md existence and role contract; open/update reuse + close-none;
  plan.md/authoring.md worker-recoverability rule.
- `adapter.rs` unit tests: kiro generates `spec-worker.json` as a full-file
  managed agent; reviewer agent still generated; `adapter generate --check`
  green.
- Frozen-surface integrity: `freeze --check` green after per-task freeze; final
  `freeze` + `upgrade --source engine` + `adapter generate --check`.
- Full `default` verification: `cargo test` + `cargo fmt --check` +
  `cargo clippy -D warnings` + `freeze --check`.

## Integration Contract

This is a workflow contract change across verbs; the delegation unit is exactly
one thing: a verified code-change task.

- **Contract owner:** `build` owns the orchestrator + per-task worker mechanism
  and the shared transport generalization.
- **Reused by:** `open` for its QA-`FAIL` rework loop only; `update` for the
  PR-feedback code change only. Both reuse the build worker mechanism rather than
  defining their own delegation.
- **Not used by:** `close` (deterministic local hygiene; nothing to delegate);
  `patch` (no spec context).
- **Stays inline (must not be dispersed):** acceptance / human QA round-trip,
  fold / ADR authoring, PR-body synthesis, the two delivery approval gates, and
  integration judgment.
- **Compatibility:** purely additive to behavior — inline build is preserved as
  the fallback, the `risk.md` reviewer cadence is unchanged, and no CLI contract
  (`pr` / `ready` / `status`) changes. Failure mode recovers by running inline.
- **Verification:** the surface `cli` `default` profile.

## Review Results

Recorded during build — a mandatory `independent-reviewer` run is required for
`risk: elevated` (`Reviewer mode: delegated | inline`, `Verdict: pass |
pass-with-comments | fail`).
