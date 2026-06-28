# Constitution (user-authored)

## Dogfood: engine source → vendored sync

The implementation target for the engine is the repo-root `engine/...`. The
`.mochiflow/engine/` copy is a gitignored, vendored, regenerated copy — never
edit it directly and never treat it as the source.

When any file under repo-root `engine/` is edited, run these before the final
verification:
1. `mochiflow freeze`
2. `mochiflow upgrade --source engine`
3. `mochiflow adapter generate --check`

