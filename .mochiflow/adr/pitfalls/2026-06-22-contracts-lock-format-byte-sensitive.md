---
id: 2026-06-22-contracts-lock-format-byte-sensitive
date: 2026-06-22
area: [cli]
status: active
---
## contracts.lock format is byte-sensitive (2026-06-22)

`freeze --check` compares the lock file byte-for-byte. The committed format is
single-line JSON: `{"version": "X.Y.Z", "hash": "..."}\n`. Using
`serde_json::to_string_pretty` or changing key order will make the check fail.
Always use the canonical `format!` in `freeze.rs` for lock serialization.
