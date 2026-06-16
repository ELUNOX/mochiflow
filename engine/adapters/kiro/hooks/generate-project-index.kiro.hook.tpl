{
  "_generated": "{{marker}}",
  "enabled": true,
  "name": "Generate Project Index",
  "description": "mochiflow: regenerate the spec dashboard and state/index.json",
  "version": "1",
  "when": {"type": "userTriggered"},
  "then": {
    "type": "runCommand",
    "command": "mochiflow index",
    "timeout": 15
  }
}
