# Versioning Policy

MochiFlow uses a 2-axis versioning scheme.

## Axes

| Axis | Location | Meaning | Bump trigger |
|:-----|:---------|:--------|:-------------|
| **engine semver** | `engine/VERSION` | Release version (X.Y.Z) | Any release: X=breaking, Y=feature, Z=fix |
| **schema_version** | consumer `config.toml` field | File format compat signal to consumers | Only when config.toml/spec.yaml format breaks |

The installed engine is self-contained: `{install_dir}/engine/VERSION` is the
active engine version for that project, and `{install_dir}/engine/MANIFEST.json`
records the integrity baseline for that same installed copy. `config.toml` does
not own engine version. A legacy `engine_version` field may remain in old
configs, but tooling ignores it.

Users do not edit `schema_version` for normal product upgrades. After updating
the CLI, they run `mochiflow upgrade`; the command installs the engine bundled
with that CLI, regenerates the engine MANIFEST and adapters, and preserves
`config.toml`, specs, and living-spec files. `mochiflow upgrade --source` is a
development/dogfood override for testing a local engine source.

## contracts.lock

`contracts/contracts.lock` is a `{version, hash}` file committed to the repo.

- **hash** = sha256 over the frozen contract surfaces:
  - `contracts/*.json` (schemas, sorted by filename)
  - `tests/conformance/golden/**` (all golden files, sorted)
- **version** = the engine VERSION at the time the lock was last updated.

The frozen surface is deliberately limited to consumer-facing contracts
(schemas + golden output). Engine file contents (`engine/MANIFEST.json` files
map) are **not** part of this hash: editing engine prose is not a compatibility
change, and folding it in forced a lock regeneration on every doc tweak. Engine
file integrity is handled separately by `mochiflow doctor engine` MANIFEST
drift detection — an integrity check for vendored engines, not a version gate.

## Contract change protocol

When a frozen surface changes (schema edit or golden update):

1. Bump `engine/VERSION` (semver rules) if the change is consumer-facing.
2. Add a section to `CHANGELOG.md` for the new version.
3. Regenerate `contracts/contracts.lock` (re-run `compute_contracts_hash()`).
4. Conformance runner verifies all three are consistent.

Editing engine docs (`commands/**`, `reference/**`, templates) does **not** trip
the version gate; it only updates `engine/MANIFEST.json` (drift baseline). If any
step above is missing for a real frozen-surface change, the `cargo test`
version-gate check **fails**.

## Consumer drift detection

Consumers (projects using MochiFlow via init/upgrade) have a materialized
`engine/VERSION` and `engine/MANIFEST.json`. Running `mochiflow doctor engine`
checks that `MANIFEST.version` matches the installed `VERSION`, then compares
file hashes against the MANIFEST. Any hand-edit is reported as drift. If the
installed engine differs from the engine bundled in the current CLI,
`doctor engine` points users to `mochiflow upgrade`.
