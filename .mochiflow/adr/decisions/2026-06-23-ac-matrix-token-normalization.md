---
id: 2026-06-23-ac-matrix-token-normalization
date: 2026-06-23
area: [cli]
spec: ac-matrix-token-normalization
status: active
---
## 2026-06-23 — ac-matrix-token-normalization: ASCII canonical tokens for AC Matrix

**Decision:** `CONFIRMED` replaces `人間確認済み` and `N/A: <reason>` replaces
`対象外（<reason>）` as the canonical done-eligible AC Matrix result tokens.
Old Japanese tokens are permanent deprecated aliases accepted by lint.

**Why:** Machine-readable stable identifiers should be locale-independent.
Japanese tokens forced non-Japanese users to input non-ASCII in an otherwise
English workflow, inconsistent with language.md's own design principle.

**Rejected:** `HUMAN_CONFIRMED` / `NOT_APPLICABLE(<reason>)` (too long, poor
column fit); migrating `_done/` specs (rewrites archived history for no user
benefit); removing deprecated aliases after a transition (would break archived
specs that are lint-checked).

**Consequence:** lint.rs `is_canonical_matrix_result` and `is_done_matrix_result`
accept both old and new tokens permanently. Engine docs (workflow.md, ship.md,
build.md, language.md) and templates use ASCII tokens only. Error messages show
new tokens as primary with deprecated aliases noted as "also accepted".
