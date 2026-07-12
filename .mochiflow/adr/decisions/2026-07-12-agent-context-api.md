---
id: 2026-07-12-agent-context-api
date: 2026-07-12
area: [cli]
spec: agent-context-api
status: active
---
## Keep agent context deterministic, read-only, and versioned

As of 2026-07-12, coding-agent context is published as one additive Draft
2020-12 JSON contract rather than extending the human status board or persisting
a generated context cache. Repository scope stays lightweight; detailed spec
scope carries structured health, observation quality, and the six lifecycle
eligibility results.

Natural-language intent remains solely owned by the engine router. The Rust CLI
answers only deterministic eligibility after a route is selected, so adapters
do not recreate lifecycle prerequisites and the CLI does not become a competing
router. Git/provider uncertainty remains explicit and never authorizes an
action. Status, index, and ready remain compatibility projections from the
shared snapshot rather than parallel state engines.

Rejected alternatives were overloading `status --json`, using the writable
`state/index.json` board as the agent contract, embedding Markdown content, and
adding a server, persistent cache, or prompt compiler.
