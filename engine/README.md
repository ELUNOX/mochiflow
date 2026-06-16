# Spec Flow Engine

Config-driven, vendored engine for spec-driven workflows. The engine is
project-agnostic: it knows verbs, status, risk, and the living-spec fold rules,
and resolves every project path/command from `config.toml` (one level up, at
`<install_dir>/config.toml`).

- Natural-language entry → `router.md`
- Verb procedures → `commands/{discuss,plan,build,ship}.md`
- Cross-cutting rules → `reference/{workflow,risk,authoring,git,language}.md`
- Templates → `templates/`
- CLI → `mochiflow` (Rust binary; conformance suite at `cli/` `cargo test`)
- Adapter templates → `adapters/{tool}/`
- Read-only reviewer → `agents/independent-reviewer.md`

## Layout

```
<install_dir>/
  config.toml      # project-owned (never overwritten by upgrade)
  engine/          # this directory (vendored, replaced wholesale by upgrade)
    VERSION  MANIFEST.json  router.md  commands/ reference/ templates/ agents/ adapters/
  state/           # generated cache (index.json / doctor.json)
```

Specs, living-spec docs, and adapter target files live outside the engine, at
paths declared in `config.toml`.

## CLI

```
mochiflow config show | config validate
mochiflow lint [--spec SLUG]
mochiflow index
mochiflow doctor [config|specs|adapter|engine]
mochiflow adapter generate [--force] [--check]
mochiflow init [--target .] [--adapter TOOL] [--language LANG] [--force] [--dry-run]
mochiflow upgrade --source PATH [--force]
```

Engine files contain no project-specific values; those belong in `config.toml`.
`mochiflow init` is deterministic and writes a schema-valid skeleton with TODO
sentinels. Project-specific configuration is completed by asking your AI
assistant to run the onboarding command, for example "onboard MochiFlow" or
"mochiflow のオンボーディングをして".
