# Agent Context Reference

`mochiflow inspect --json` is the read-only machine contract for deterministic repository facts. `mochiflow inspect <slug> --json` adds structured health, delivery observations, and eligibility for `discuss`, `plan`, `build`, `open`, `update`, and `close`.

The router remains the sole owner of natural-language intent and route selection. Eligibility never selects a route from prose. After the router has selected a lifecycle command, the procedure may consult the matching action row and treat `ineligible` or `unknown` blockers as a stop condition before executing. The procedure remains the sole owner of mutations and execution steps.

The JSON contract is frozen at `contracts/agent-context.schema.json` with `schema_version: 1`. Repository and spec inspection are read-only. `--fetch` explicitly refreshes origin once; a failure is represented by `fetch_failed` and otherwise usable local facts remain available. `ok` and `degraded` exit 0; `partial` and `error` exit 1; argument misuse retains the CLI usage exit.

Machine consumers branch on stable identifiers, never localized messages. Unknown Git or provider observations never authorize a lifecycle action.
