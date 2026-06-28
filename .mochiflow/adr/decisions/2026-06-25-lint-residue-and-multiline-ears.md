---
id: 2026-06-25-lint-residue-and-multiline-ears
date: 2026-06-25
area: [cli]
spec: lint-residue-and-multiline-ears
status: active
---
## 2026-06-25 — lint-residue-and-multiline-ears: lint uses scoped Markdown masking instead of a parser

**Decision:** Template residue detection stays inside the existing structural
lint path and masks fenced code blocks / inline code before checking unfinished
template signals. Multi-line EARS validation treats each AC declaration plus its
continuation lines as a block, ending at the next AC declaration or Markdown
section heading.

**Why:** The approval gate needs mechanical protection against unfinished spec
templates, but a full Markdown parser would be disproportionate for the current
signals. Masking code regions prevents common false positives for examples that
legitimately contain `{...}` or `TBD`, while block-aware AC scanning fixes the
observed false warning without changing the artifact schema.

**Rejected:** Keeping residue detection as prose-only guidance; warning instead
of failing on unfinished template residue; requiring EARS keywords on the AC
declaration line; failing every brace pair regardless of code context.

**Consequence:** Conformance coverage now pins each residue class, code-context
false-positive avoidance, and AC block boundary behavior.
