---
id: 2026-06-28-kiro-agent-tools-are-coarse-categories
date: 2026-06-28
area: [cli]
status: active
---
## Kiro agent `tools` are coarse categories, not fine tool names (2026-06-28)

**Applies to:** any generated `.kiro/agents/*.json` (the `tools` / `allowedTools`
arrays) produced by the Kiro adapter.

**Signal:** in the Kiro agent selector / config view, the agent's tools list
shows several entries as **"unknown"** (e.g. a worker declaring six tools shows
four "unknown"). The agent may also silently lack the capability it intended.

**Cause:** Kiro's agent `tools` field takes **coarse tool categories**, not
individual tool names. The recognized categories are `fs_read` / `fs_write` /
`execute_bash` / `use_aws`, with aliases `read` / `write` / `shell` / `aws`
(plus MCP server names). Finer-grained names such as `grep`, `glob`, `edit`, and
`bash` are **not** categories, so Kiro cannot resolve them and renders each as
"unknown". (`read` already covers file read + directory + search; `write` covers
create/edit; `shell` covers running commands incl. read-only git.)

**Guardrail:** declare only Kiro categories in any `*.json.tpl` `tools` array —
a read-only agent uses `["read"]`; a write+verify+commit worker uses
`["read","write","shell"]`. Never use `grep` / `glob` / `edit` / `bash`. After
changing a template, re-run `adapter generate` and check the agent in Kiro shows
no "unknown" tools.

**Check:** the generated `.kiro/agents/*.json` `tools` arrays are a subset of
`{read, write, shell, aws}` (+ MCP names); the conformance test asserts the
worker tools contain none of `grep/glob/edit/bash`.

**Status:** Active.
