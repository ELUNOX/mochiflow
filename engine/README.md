# Spec Flow Engine

Config-driven, vendored engine for spec-driven workflows. The engine is
project-agnostic: it knows verbs, status, risk, and the living-spec fold rules,
and resolves every project path/command from `config.toml` (one level up, at
`<install_dir>/config.toml`).

- Natural-language entry → `router.md`
- Lifecycle verbs → `commands/{discuss,plan,build,open,update,close}.md`
- Non-phase commands → `commands/{patch,review,refresh-context,onboard}.md`
- Cross-cutting rules → `reference/{workflow,risk,authoring,git,language}.md`
- Templates → `templates/`
- CLI → `mochiflow` (Rust binary; conformance suite at `cli/` `cargo test`)
- Adapter templates → `adapters/{tool}/`
- Read-only reviewer → `agents/independent-reviewer.md`

## Artifact model

- `spec.md` is the product contract: problem, goal, scope, ACs, QA scenarios,
  NFRs, and the verification plan.
- `design.md` is the technical contract when required: decisions, alternatives,
  interface contracts, failure modes, rollout / rollback, observability, and
  test strategy.
- `tasks.md` is the executable checklist when required: dependency-ordered
  `T-###` tasks with files, done criteria, and stop conditions.
- The AC Matrix lives in `spec.md` and tracks AC → implementation →
  verification → evidence → result.

Generated prose follows the project language. Machine-readable IDs and Matrix
results remain stable English tokens.

## Layout

```
<install_dir>/
  config.toml      # project-owned (never overwritten by upgrade)
  engine/          # this directory (vendored, tracked, replaced wholesale by upgrade)
    VERSION  MANIFEST.json  router.md  commands/ reference/ templates/ agents/ adapters/
  state/           # generated cache (index.json / doctor.json)
```

Specs, living-spec docs, and adapter target files live outside the engine, at
paths declared in `config.toml`.

## CLI

```
mochiflow config show | config validate
mochiflow guide
mochiflow lint [--spec SLUG]
mochiflow index
mochiflow doctor [config|specs|adapter|engine]
mochiflow adapter generate [--force] [--check]
mochiflow init [--target .] [--adapter TOOL] [--artifact-language TAG] [--conversation-language TAG|auto] [--force] [--dry-run]
mochiflow join [--target .] [--dry-run]
mochiflow detach [--target .] [--dry-run] [--purge]
mochiflow upgrade --source PATH [--force]
mochiflow pr --spec SLUG --title TITLE --body-file PATH [--draft] [--dry-run]
```

Engine files contain no project-specific values; those belong in `config.toml`.
`mochiflow init` is deterministic and writes a schema-valid skeleton with TODO
sentinels. Project-specific configuration is completed by asking your AI
assistant to run the onboarding command, for example "onboard MochiFlow" or
"mochiflow のオンボーディングをして".
