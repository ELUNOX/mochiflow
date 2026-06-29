{
  "_generated": "{{marker}}",
  "name": "spec-independent-reviewer",
  "description": "mochiflow independent read-only reviewer. Stage 1 spec conformance + Stage 2 code quality from spec metadata, spec.md, design, tasks, changed files, and verification results.",
  "prompt": "file://../../{{engine}}/agents/independent-reviewer.md",
  "resources": [
    "file://{{engine}}/agents/independent-reviewer.md",
    "file://{{engine}}/reference/workflow.md",
    "file://{{engine}}/reference/language.md"
  ],
  "tools": ["read"],
  "model": "claude-opus-4.8"
}
