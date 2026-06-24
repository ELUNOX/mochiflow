# Constitution (user-authored)

## Dogfood: engine source → vendored sync

When any file under `engine/` is edited, run these before the final verification:
1. `mochiflow freeze`
2. `mochiflow upgrade --source engine`
3. `mochiflow adapter generate --check`

