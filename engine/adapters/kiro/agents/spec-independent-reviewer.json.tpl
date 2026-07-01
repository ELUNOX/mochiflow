{
  "_generated": "{{marker}}",
  "name": "spec-independent-reviewer",
  "description": "mochiflow independent read-only grounded adversary. Runs S0-S4 plus Falsification from spec metadata, spec.md, design, tasks, repository reality, changed files, ADR records, and verification results.",
  "prompt": "file://../../{{engine}}/agents/independent-reviewer.md",
  "resources": [
    "file://{{engine}}/agents/independent-reviewer.md",
    "file://{{engine}}/reference/workflow.md",
    "file://{{engine}}/reference/language.md",
    "file://{{engine}}/reference/risk.md",
    "file://{{engine}}/reference/authoring.md",
    "file://{{engine}}/reference/git.md"
  ],
  "tools": ["read"],
  "model": "claude-opus-4.8"
}
