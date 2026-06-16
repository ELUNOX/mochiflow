# Legacy spec.yaml forms

The `documents:` nested form (pre-v2 engine) is **contract-external**.
It exists in archived specs (`_done/`) as read-only historical data.

The canonical `spec.yaml` schema is `spec.schema.json` (flat v1, `version: 1`).
No migration tooling is provided. Consumers encountering the legacy form
should treat it as opaque metadata not validated by the contract.
