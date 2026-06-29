{
  "_generated": "{{marker}}",
  "name": "spec-worker",
  "description": "mochiflow disposable per-task build worker. Implements one tasks.md task from a context pack (read repo-wide, write within the task's Files), runs the default verification, commits the task, and returns a compact report.",
  "prompt": "file://../../{{engine}}/agents/worker.md",
  "resources": [
    "file://{{engine}}/agents/worker.md",
    "file://{{engine}}/reference/workflow.md",
    "file://{{engine}}/reference/git.md",
    "file://{{engine}}/reference/engineering-standards.md",
    "file://{{engine}}/reference/risk.md",
    "file://{{engine}}/reference/language.md"
  ],
  "tools": ["read", "grep", "glob", "edit", "write", "bash"],
  "model": "claude-opus-4.8"
}
