# Versioning

MochiFlow has a few version numbers because the CLI, embedded engine, and config
file format have different compatibility surfaces.

## Product and CLI version

`mochiflow --version` reports the product / CLI version. This is the version
users normally care about when installing or reporting issues.

## Engine version

`cli/Cargo.toml` `[workspace.package].version` is the single source of truth
for the engine and contract-surface semver. `mochiflow freeze` derives
`engine/VERSION`, `engine/MANIFEST.json`, and `contracts/contracts.lock` from it.

Inside an initialized project, `.mochiflow/engine/VERSION` records the installed
engine copy used by adapters, `config show`, and `doctor engine`.

`.mochiflow/engine/MANIFEST.json` records the installed engine integrity
baseline. Its `version` matches the installed `VERSION`.

## Config schema version

`schema_version` in `.mochiflow/config.toml` is the config file-format
compatibility number. Users do not edit it during normal upgrades.

## Upgrading

After updating the installed CLI, run from the target project root:

```bash
mochiflow upgrade
mochiflow doctor
```

`upgrade` replaces the project's vendored engine with the engine bundled into
the installed CLI, regenerates adapters, and preserves project config, specs,
context, and ADR files.

For a fresh clone of an already-initialized team project, the tracked vendored
engine is already present. Use `mochiflow join` only when local runtime state,
adapters, `INDEX.md`, or a missing engine need repair.
