{
  "_generated": "{{marker}}",
  "name": "spec-builder",
  "description": "Spec-flow lifecycle agent: discuss / plan / build / ship inline; build implements inline and folds durable knowledge into the living spec at ship; review uses the read-only independent-reviewer transport.",
  "prompt": "file://../../{{engine}}/router.md",
  "includeMcpJson": true,
  "tools": ["read", "grep", "glob", "write", "shell", "code", "subagent", "todo", "web_search", "web_fetch", "@cupertino"],
  "allowedTools": ["read", "grep", "glob", "write", "shell", "code", "todo", "web_search", "web_fetch", "@cupertino"],
  "resources": [
    "file://{{engine}}/commands/discuss.md",
    "file://{{engine}}/commands/plan.md",
    "file://{{engine}}/commands/build.md",
    "file://{{engine}}/commands/ship.md",
    "file://{{engine}}/reference/workflow.md",
    "file://{{engine}}/reference/risk.md",
    "file://{{engine}}/reference/authoring.md",
    "file://{{engine}}/reference/git.md",
    "file://{{engine}}/reference/language.md",
    "file://{{engine}}/templates/spec/spec.yaml",
    "file://{{engine}}/templates/spec/spec.md",
    "file://{{engine}}/templates/spec/design.md",
    "file://{{engine}}/templates/spec/tasks.md",
    "file://{{engine}}/templates/delivery/qa-instructions.md",
    "file://{{engine}}/templates/delivery/pr-description.md",
    "file://{{constitution.project}}",
    "file://{{constitution.local}}",
    "file://{{context.product}}",
    "file://{{context.structure}}",
    "file://{{context.tech}}"
  ],
  "toolsSettings": {
    "write": {
      "allowedPaths": {{allow_json}},
      "deniedPaths": {{deny_json}}
    },
    "shell": {
      "autoAllowReadonly": true,
      "allowedCommands": [".*"],
      "deniedCommands": [
        ".*git push.*--force.*",
        ".*git push.* -f( .*)?",
        ".*git reset --hard.*",
        ".*git clean .*-[^ ]*f.*",
        ".*git branch -D.*",
        ".*rm -rf /( .*)?",
        ".*rm -rf /\\*.*",
        ".*sudo .*",
        ".*chmod -R 777.*",
        ".*chown -R .*",
        ".*gh pr create.*",
        ".*gh pr merge.*",
        ".*glab mr create.*",
        ".*az repos pr create.*"
      ]
    },
    "web_fetch": {"blocked": [".*pastebin\\.com.*"]}
  },
  "subagent": {
    "availableAgents": ["spec-independent-reviewer"],
    "trustedAgents": ["spec-independent-reviewer"]
  },
  "model": "auto"
}
