# Agent Context API — Design

## Design Decisions

### Keep intent routing and deterministic eligibility separate

`engine/router.md` remains the only authority that maps user language and
explicit commands to a workflow. A new Rust eligibility evaluator answers a
different question: whether a selected lifecycle action can run against the
currently observed repository state and, if not, which stable blocker codes
apply. Engine procedures consult that result and continue to own execution.
This avoids both extremes rejected in discuss: raw facts that every adapter must
reinterpret, and a second natural-language router embedded in the CLI.

### Publish one additive Draft 2020-12 contract

`contracts/agent-context.schema.json` is the public contract and uses JSON
Schema Draft 2020-12 `$defs`, `oneOf`, `const`, required properties, and
`additionalProperties: false` to distinguish repository, spec, partial, and
error documents. This follows the official Draft 2020-12 specification:
https://json-schema.org/draft/2020-12. The schema carries its own API
`schema_version: 1`; the product release moves to `1.3.0`, while consumer config
`schema_version` remains unchanged.

Stable machine identifiers are closed enums in the schema. Display messages are
not used for control flow and may follow the configured conversation language.
Positive fixtures cover repository, spec, degraded, partial, and error results;
negative fixtures prove missing required fields, unknown enum values, absolute
paths, and mismatched scope payloads are rejected.

### Build one immutable observation snapshot

Add a core `inspect` module that owns repository/spec discovery, observation
quality, active binding, eligibility, serialization inputs, and human rendering
inputs. Snapshot construction returns data and never prints or writes. `status`
and `index` project their existing board models from the same snapshot; `ready`
projects build eligibility and preserves its existing text and exit behavior.

`delivery.rs` retains the pure asserted-plus-observed precedence rule and next
action semantics, but per-spec process execution moves behind the batch
collector. Compatibility wrappers may remain temporarily only when an existing
single-spec caller cannot migrate in the same task; they must delegate to the
shared collector rather than restore independent logic.

### Use constant-bounded batch probes with injectable execution

The collector executes a bounded sequence independent of spec count:

1. current symbolic branch and worktree status;
2. all relevant local/remote refs and the subset merged into the configured base;
3. all `Spec:` trailers reachable from the base in one log traversal;
4. ignore classification for all concrete spec artifact paths through one
   stdin batch;
5. one GitHub PR list query for configured GitHub repositories.

Git 2.50.1 officially supports formatted ref iteration and trailer extraction:
https://git-scm.com/docs/git-for-each-ref and
https://git-scm.com/docs/pretty-formats. GitHub CLI 2.96.0 officially supports
`gh pr list --state all --limit ... --json headRefName,state`:
https://cli.github.com/manual/gh_pr_list. If the provider result count reaches
the chosen limit, unmatched provider facts become unknown and the response
adds `provider_result_truncated`; they are never treated as known false.

Process execution is injected at the collector boundary so unit tests can count
calls and supply spawn failures, non-zero exits, malformed UTF-8/JSON, and
sanitization witnesses without changing global PATH. The production runner
captures stdout for parsing, never copies raw stderr into API documents, and
records only safe reason codes.

### Preserve observation quality explicitly

Every fact that can fail independently is represented as known, unknown, or not
applicable. Known values carry the typed value; unknown facts carry a stable
reason code; not-applicable is used when configuration makes the fact
meaningless, such as provider PR state under `provider = none`.

Delivery precedence may use known true observations. An unknown higher-priority
signal makes the detailed delivery result and dependent eligibility uncertain
rather than asserting a lower state as definitive. Existing `status` and
`state/index.json` compatibility projections keep their current fail-soft
fallback semantics, while `inspect` exposes the uncertainty that those legacy
surfaces cannot represent.

### Keep repository and spec views intentionally asymmetric

Repository scope performs direct-child discovery, batch observation, and coarse
summaries. It retains malformed specs as error entries and returns `partial`
with exit 1, but does not lint every valid spec. Active binding uses the current
branch only when its conventional prefix/slug matches exactly one valid spec;
otherwise it reports unresolved or ambiguous candidates without modification
time heuristics.

Spec scope resolves one slug within the configured active/legacy boundaries,
runs structured lint and readiness checks, and reports every lifecycle action.
Eligibility is tri-state so an unavailable delivery fact does not become a
false blocker or permission. Suggested workflow precedence is deterministic:
`open` outranks a still-technically-resumable `build`, followed by `build` and
`plan`; `close` is suggested for observed merged cleanup. `discuss` and `update`
are never suggested automatically because they require idea or feedback intent.
Existing human delivery next actions remain a separate field.

### Reuse structured lint and ready logic

Refactor lint into a pure report API returning issues and a presentation wrapper
that preserves current output. Eligibility consumes stable issue categories,
not rendered messages. A shared readiness core matches the current `ready`
contract exactly: lint has no failures, asserted status is `approved`, and every
declared surface has a runnable default verification profile. The detailed
`build` lifecycle action composes that core with build-entry observations for
the expected branch and unrelated worktree dirt. Legacy `ready` projects only
the shared core, so those two entry-only blockers never change its output or
exit behavior; inspect and engine preconditions consume the full action result.

Open eligibility additionally requires build completion evidence: all tasks are
checked when `tasks.md` exists, automated Matrix rows are settled, human rows
may remain `PENDING_HUMAN`, and the required elevated implementation review is
recorded. Update requires known `in_review`; close requires known merged state
and pending local cleanup. A prerequisite that cannot be determined is unknown,
not eligible true.

The closed eligibility table is:

| Action | Eligible | Ineligible blockers | Unknown propagation | Auto-suggestion |
| --- | --- | --- | --- | --- |
| `discuss` | A valid active spec is `draft`. Repository summaries may expose `discuss` as the coarse next candidate for a valid backlog seed, but slug detail remains spec-only. | Legacy archived target → `target_archived`. | `approved` or `accepted` requires the router's state/intent confirmation → `intent_confirmation_required`; malformed metadata is a target error, not an action result. | Never. |
| `plan` | Status is `draft` and either `pitch.md` exists or the existing `spec.md` satisfies the current pitchless-micro shape. | Other status → `status_not_draft`; neither plan input shape exists → `plan_input_missing`; structural lint failure → `lint_failed`. | Required worktree/spec observation unavailable → its stable observation code. | Yes when known eligible. |
| `build` | Status is `approved`, structured lint has no FAIL, every used surface has a configured non-TODO default verification profile, the expected spec branch exists, and no unrelated dirty path blocks entry. | `status_not_approved`, `lint_failed`, `verification_missing`, `verification_todo`, `branch_missing`, or `worktree_dirty`. | Worktree, branch, or required config observation unavailable → its stable observation code. | Yes only when `open` is not known eligible. |
| `open` | Status is `approved`; all tasks are checked when tasks exist; the AC Matrix exists; automated rows are settled to `PASS`, `FAIL`, or reasoned `N/A` with no automated `FAIL`; human rows may remain `PENDING_HUMAN`; the required elevated review is recorded through the current implementation; and no unrelated dirty path blocks entry. | `status_not_approved`, `tasks_incomplete`, `matrix_missing`, `automated_checks_unsettled`, `automated_checks_failed`, `review_result_missing`, `review_result_stale`, or `worktree_dirty`. | Worktree or review-freshness evidence unavailable → its stable observation code. | Yes and takes precedence over `build`. |
| `update` | Status is `accepted` and delivery state is known `in_review`; no unrelated dirty path blocks entry. | `status_not_accepted`, `delivery_not_in_review`, or `worktree_dirty`. | Provider/Git delivery or worktree observation unavailable → `delivery_unknown` or the corresponding stable observation code. | Never; feedback intent is external. |
| `close` | Delivery is known merged, local cleanup is pending, and the worktree is known clean. | `delivery_not_merged`, `cleanup_not_pending`, or `worktree_dirty`. | Merge, cleanup, or worktree observation unavailable → `delivery_unknown` or the corresponding stable observation code. | Yes when known eligible. |

Action results use `eligible`, `ineligible`, or `unknown`; observation values
may additionally be `not_applicable`. Multiple blockers are retained in stable
order instead of stopping at the first. A malformed requested spec produces the
top-level error contract and therefore never fabricates six unknown action
rows. T-001 schema enums and T-003 table-driven tests must be derived from this
table; changing a row after approval is a design change and returns to plan.
Compatibility tests pin dirty-worktree and missing-expected-branch fixtures in
which legacy `ready` still succeeds while detailed `build` eligibility reports
the corresponding entry blocker.

### Keep output safe and portable

All contract paths are slash-normalized and relative to the canonical repository
root. Conversion rejects paths that cannot be proven repository-local. Dirty
path lists may be returned because they explain eligibility, but no file body or
diff is read for presentation. Configured command bodies are reduced to
`configured`, `missing`, or `todo`; raw Git/provider stderr and absolute paths
never enter the response.

Human mode is a concise projection of the same document. JSON mode writes one
pretty-printed JSON value and a trailing newline to stdout for all results.
Progress or sanitized diagnostics go to stderr only when needed. Config loading
for `inspect --json` uses an inspect-specific dispatch path so invalid config can
still produce the schema-defined error document rather than exiting through the
current generic text-only loader.

## Architecture

The implementation follows this flow:

1. CLI parses optional slug, `--json`, and `--fetch`.
2. Inspect-specific config loading returns either a valid `Config` or a
   serializable contract error.
3. Optional fetch runs once before observation and contributes a warning on
   failure.
4. The collector discovers spec/seed paths and performs the constant-bounded Git
   and provider snapshot.
5. Repository mode builds summaries and visible parse-error entries; spec mode
   resolves one target, obtains a structured lint report, and evaluates actions.
6. A result classifier selects `ok`, `degraded`, `partial`, or `error` and the
   corresponding exit code.
7. JSON or human presentation renders from the immutable document.
8. Existing board consumers project their legacy shapes from the same snapshot.

Snapshot construction must not depend on generated `INDEX.md` or
`state/index.json`; both remain downstream projections. No snapshot survives the
process.

## Data Model / Interfaces

The public contract has these conceptual layers:

- Common envelope: schema version, scope, result, observation timestamp,
  degraded flag, warnings, and exactly one scope payload.
- Repository payload: Git branch/base summary, active resolution, spec summary
  entries, backlog summaries, and malformed-entry errors.
- Spec payload: metadata, persistence, related paths, worktree relation,
  structured health, observed delivery facts, derived state quality, six action
  evaluations, suggested workflow, and human next action.
- Observation: known typed value, unknown with a stable reason, or not
  applicable with a stable reason.
- Diagnostic: stable code, optional safe localized message, and zero or more
  repository-relative paths.
- Action evaluation: stable lifecycle action, tri-state eligibility, and
  blocker diagnostics.

Stable initial codes include categories for invalid config/spec, missing or
ambiguous spec, status mismatch, missing pitch, lint failure, missing/TODO
verification, incomplete tasks or Matrix, missing review result, provider/Git
unavailability, fetch failure, provider truncation, delivery not in review,
delivery not merged, and cleanup not pending. The schema owns the exact enum;
callers never branch on message text.

The internal snapshot API accepts the resolved config, scope, refresh outcome,
and injected command runner. It returns a document plus process exit code and
does not access stdout/stderr directly. Existing CLI wrappers receive derived
board or build-readiness projections rather than reaching into collector
internals.

## Error Handling

- Config load failure in JSON mode produces `result: error`, exit 1, and one
  sanitized `config_invalid` diagnostic. Human mode retains a concise failure.
- Missing, path-like, ambiguous, or malformed requested targets produce a spec
  error document and exit 1; no fallback target is guessed.
- Malformed entries during repository discovery remain in the response and make
  the overall result `partial`, exit 1. Other valid entries remain usable.
- Provider or non-authoritative Git observation failure produces unknown facts,
  `result: degraded`, warnings, and exit 0 when repository integrity is intact.
- A failed explicit fetch adds `fetch_failed` and continues from existing refs;
  it never deletes or resets refs.
- Provider truncation marks unmatched PR facts unknown. Matched returned facts
  remain known.
- Serialization failure uses a minimal schema-valid internal-error document when
  possible; it never falls back to a mixed text/JSON stdout stream.
- Existing human status/index behavior continues its documented fail-soft
  projection, even when inspect exposes richer uncertainty.

## Test Strategy

- Schema conformance tests compile the Draft 2020-12 schema, accept all positive
  response variants, and reject incomplete, unknown-code, absolute-path, and
  cross-scope fixtures.
- Pure unit tests cover delivery precedence, observation quality propagation,
  action eligibility, suggestion precedence, active branch binding, result/exit
  classification, sanitization, and repository-relative path conversion.
- Fake-runner tests count Git/provider invocations across one and many specs and
  inject spawn, exit, output, truncation, and malformed JSON failures.
- CLI integration tests cover human/JSON output, invalid config before normal
  dispatch, stdout purity, localized messages with stable IDs, missing/invalid
  targets, and explicit fetch behavior.
- Filesystem/Git witnesses hash or snapshot the worktree and refs before and
  after ordinary inspect to prove no mutation.
- Compatibility tests preserve `status` output shape, index golden, index JSON
  next-action contract, and every current `ready` success/failure message.
- Engine conformance tests pin the router/eligibility responsibility boundary
  and confirm all six lifecycle procedures consult the shared API without
  duplicating natural-language routing.
- Release verification runs the complete default profile, cargo-deny, freeze,
  source-engine dogfood synchronization, adapter drift check, spec lint, and the
  required final elevated-risk change review.

## Integration Contract

- Contract owner: `contracts/agent-context.schema.json`, implemented by the
  Rust CLI inspect module and consumed by engine routing/procedure guidance and
  external adapters.
- Request: optional positional slug plus optional `--json` and `--fetch`; slug
  values use the existing spec identifier contract and are not arbitrary paths.
- Response: one repository or spec document with `schema_version: 1` in JSON
  mode; human mode is non-contractual localized presentation of the same data.
- Result/exit: `ok` and `degraded` exit 0; `partial` and `error` exit 1; Clap
  argument misuse retains exit 2.
- Error transport: structured codes and safe paths in JSON; no raw subprocess
  stderr, absolute path, file content, diff, or configured shell command body.
- Authentication: no new credential handling. Provider reads reuse the existing
  GitHub CLI session; unavailable authentication degrades to unknown.
- Compatibility: existing commands, config/spec formats, lifecycle statuses,
  index golden, and `state/index.json` remain compatible. This additive frozen
  contract triggers product version `1.3.0`, not config `schema_version`.
- Failure handling: local facts survive provider/Git partial failure; integrity
  failures remain visible and non-zero; unknown never silently authorizes an
  action.
- Verification: schema fixtures, behavioral CLI tests, probe-count tests,
  existing compatibility suites, version/freeze gates, and engine conformance.

## Review Results

No implementation review has run. Build must record the required elevated-risk
`change-reviewer` result here through the final code-changing commit.
