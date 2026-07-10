---
id: user-instructions-directory-fixed-opaque-paths
date: 2026-07-10
area: cli
spec: user-instructions-directory
status: active
---

# User instruction directories are fixed opaque paths

As of 2026-07-10, MochiFlow provides `.mochiflow/instructions/` for shared
user-authored Markdown and `.mochiflow/instructions.local/` for local-only
Markdown, but deliberately keeps both directories outside the runtime model.

The paths are fixed install-relative scaffolding rather than config fields,
adapter inputs, indexes, manifests, or engine-owned resources. `init` may create
the directories, seed the shared README once, and ensure the local ignore rule;
after creation, file contents remain user-owned and are used only when a user
explicitly names a path.

This avoids turning occasional operating notes into always-loaded standing
context and keeps join, upgrade, doctor, freeze, adapter generation, and drift
checks from acquiring accidental ownership of arbitrary user Markdown. The
rejected alternatives were a configurable instruction path, automatic discovery
or indexing, and extending the constitution contract; each would make optional
notes part of MochiFlow's managed state or standing context.
