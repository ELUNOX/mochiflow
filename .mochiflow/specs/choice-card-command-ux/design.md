# Clarify choice-card commands and numbered replies — Design

## Design Decisions

- Choice cards are the source of the numbered-reply mapping. A number is not a
  router trigger by itself; it is an inline alias resolved only when the agent has
  just displayed a clear choice card in the same conversation. This avoids adding
  persistent state or global numeric commands.
- User-facing labels are conversation-language actions. Stable internal tokens
  remain accepted for compatibility and for users who already know the workflow,
  but they are secondary to labels such as `計画を確定` and `PR準備を始める`.
- `計画を確定` is the visible name for the first delivery gate. It must describe
  the actual operation: set `status: approved`, re-check, and commit plan
  artifacts. It must also state that implementation does not start until the
  user chooses `実装を開始する` or an equivalent build trigger.
- Choice selection is the dispatch primitive: choosing a visible option by label
  or number invokes that option's action. For `計画を確定`, that action is the
  approve-to-build gate.
- `再開用プロンプトを作る` replaces visible `later` language only at high-value
  handoff points. The compatibility keyword `later` remains useful, but the label
  should name the artifact produced for the next session.
- PR title/body correction remains free-form feedback before the PR creation
  gate. There is no dedicated `PR本文を修正する` command because that would make a
  lightweight edit look like a workflow transition.

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
  - `計画を作る`
  - `再開用プロンプトを作る`
  - `計画を確定`
  - `レビューする`
  - `実装を開始する`
  - `PR準備を始める`
  - `PRを作成する`
  - `approve plan`
  - `resume`
  - `create pr`

## Error Handling

- If a bare number cannot be mapped to the most recent unambiguous choice card,
  the agent asks the user to choose again using the current card labels.
- If plan-confirmation language is ambiguous, the agent explains that
  `計画を確定` saves and commits the plan but does not start implementation.
- If the most recent card maps a number to `計画を確定`, the agent dispatches the
  same plan-confirmation action as the label form.
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

Build records the required elevated-risk reviewer result here after
implementation. Use `Reviewer mode: delegated | inline` and
`Verdict: pass | pass-with-comments | fail`.
