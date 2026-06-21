# Build Session Handoff Prompt Template

Use this template at the end of `mochiflow-plan` after `spec.yaml` is `approved` and
the final consistency check passes. Render the copy-paste prompt in
`[i18n].artifact_language`. Preserve commands, paths, metadata values, and
identifiers exactly.

Do not name a specific adapter entrypoint such as `AGENTS.md`, `CLAUDE.md`,
`.kiro/`, or Copilot instructions. The target session should follow whichever
agent instructions are installed for the project.

```text
Please run `{slug} build`.

Follow this project's agent instructions and the MochiFlow router.
Target spec: `{specs_dir}/{slug}/`

Start by reading `spec.yaml`, `spec.md`, and any `design.md` / `tasks.md` files
that exist. Confirm `status: approved`, then implement the spec, run the
required verification, and record the acceptance-check results in the spec
artifacts.
```

For non-English artifact languages, translate only the prose. Keep `{slug}`,
`{specs_dir}/{slug}/`, `build`, `spec.yaml`, `spec.md`, `design.md`, `tasks.md`,
and `status: approved` unchanged.
