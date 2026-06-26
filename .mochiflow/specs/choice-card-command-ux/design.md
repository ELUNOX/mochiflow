# Clarify choice-card commands and numbered replies — Design

## Design Decisions

- Choice cards are the source of the numbered-reply mapping. A number is not a
  router trigger by itself; it is an inline alias resolved only when the agent has
  just displayed a clear choice card in the same conversation. This avoids adding
  persistent state or global numeric commands.
- User-facing labels are conversation-language actions. Stable internal tokens
  remain accepted for compatibility and for users who already know the workflow,
  but they are secondary to localized labels such as "confirm the plan" and
  "start PR preparation".
- The localized "confirm the plan" action is the visible name for the first
  delivery gate. It must describe the actual operation: set `status: approved`,
  re-check, and commit plan artifacts. It must also state that implementation
  does not start until the user chooses the localized "start implementation"
  action or an equivalent build trigger.
- Choice selection is the dispatch primitive: choosing a visible option by label
  or number invokes that option's action. For plan confirmation, that action is
  the approve-to-build gate.
- The localized "create a resume prompt" action replaces visible `later` language
  only at high-value handoff points. The compatibility keyword `later` remains
  useful, but the label should name the artifact produced for the next session.
- Build-completion resume prompts are generated inline from the active slug and
  spec path, pointing the next session to `{slug} ship`. This keeps the change
  documentation-only and avoids adding a second handoff template.
- PR title/body correction remains free-form feedback before the PR creation
  gate. There is no dedicated `PR本文を修正する` command because that would make a
  lightweight edit look like a workflow transition.
- Risk is `elevated` because the change updates behavior contracts across
  multiple command procedures and shared routing guidance; no data, migration, or
  security boundary is involved.

## Architecture

- Update repo-root `engine/` source files as the source of truth.
- Keep generated or vendored copies synchronized through the existing project
  commands after source edits:
  - `mochiflow freeze`
  - `mochiflow upgrade --source engine`
  - `mochiflow adapter generate --check`
- Do not add CLI code or a state file unless implementation proves the engine
  instructions cannot express the behavior consistently.

## Data Model / Interfaces

- No serialized data model changes.
- The user-facing interface is the phase-completion choice card text in engine
  command procedures and presentation rules.
- The compatibility interface is the existing trigger vocabulary plus the new
  explicit labels:
  - `review`
  - `build`
  - `ship`
  - `later`
  - `approved`
  - localized label: create the plan
  - localized label: create a resume prompt
  - localized label: confirm the plan
  - localized label: review
  - localized label: start implementation
  - localized label: start PR preparation
  - localized label: create the PR
  - `approve plan`
  - `resume`
  - `create pr`

## Error Handling

- If a bare number cannot be mapped to the most recent unambiguous choice card,
  the agent asks the user to choose again using the current card labels.
- If plan-confirmation language is ambiguous, the agent explains that
  confirming the plan saves and commits the plan but does not start
  implementation.
- If the most recent card maps a number to the localized plan-confirmation
  action, the agent dispatches the same plan-confirmation action as the label
  form.
- If ad-hoc review completes outside an approved implementation-ready context,
  the agent reports the review result and presents only actions valid for the
  current lifecycle state.
- If PR-body feedback is received before PR creation, the agent revises and
  re-presents the PR content rather than routing to the PR Feedback Loop.
- If a requested source edit would require CLI runtime behavior, stop and route
  back to plan before changing scope.

## Test Strategy

- Run `mochiflow lint --spec choice-card-command-ux` before approval.
- During build, manually inspect changed engine procedures against the acceptance
  criteria and QA scenarios.
- Run the configured CLI verification after implementation:
  `cargo test --manifest-path cli/Cargo.toml && cargo fmt --manifest-path cli/Cargo.toml --all -- --check && cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings && cargo run --manifest-path cli/Cargo.toml -- freeze --check`.
- Run constitution-required engine checks after editing `engine/`:
  `mochiflow freeze`, `mochiflow upgrade --source engine`, and
  `mochiflow adapter generate --check`.

## Review Results

- Reviewer mode: delegated
  - Reviewer: Avicenna subagent
  - Verdict: pass-with-comments
  - Finding: Medium test-gap on AC-11 evidence granularity in `tasks.md`; fixed
    by replacing summary-only evidence with command-output transcript details.
  - Note: reviewer report header self-reported `inline`, but the review was run
    through delegated subagent transport in this session.
