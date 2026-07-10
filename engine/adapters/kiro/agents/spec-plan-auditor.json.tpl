{
  "_generated": "{{marker}}",
  "name": "spec-plan-auditor",
  "description": "mochiflow read-only plan audit. Runs S0-S4 plus Falsification from spec metadata, spec.md, design, tasks, QA attack dimensions, repository reality, and ADR records before implementation.",
  "prompt": "file://../../{{engine}}/agents/plan-auditor.md",
  "resources": [
    "file://{{engine}}/agents/reviewer-core.md",
    "file://{{engine}}/agents/plan-auditor.md",
    "file://{{engine}}/reference/risk.md",
    "file://{{engine}}/reference/language.md"
  ],
  "tools": ["read"],
  "model": "claude-opus-4.8"
}
