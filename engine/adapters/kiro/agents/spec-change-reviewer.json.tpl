{
  "_generated": "{{marker}}",
  "name": "spec-change-reviewer",
  "description": "mochiflow read-only post-implementation review. Runs S0-S4 plus Falsification from implemented diff, changed files, verification evidence, AC Matrix, QA attack dimensions, repository reality, and ADR records.",
  "prompt": "file://../../{{engine}}/agents/change-reviewer.md",
  "resources": [
    "file://{{engine}}/agents/reviewer-core.md",
    "file://{{engine}}/agents/change-reviewer.md",
    "file://{{engine}}/reference/verification.md",
    "file://{{engine}}/reference/risk.md",
    "file://{{engine}}/reference/language.md"
  ],
  "tools": ["read"],
  "model": "claude-opus-4.8"
}
