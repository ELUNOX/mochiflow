---
id: 2026-07-01-grounded-independent-reviewer
date: 2026-07-01
area: [cli]
spec: independent-reviewer-grounded-redesign
status: active
---
## 2026-07-01 - Independent review is grounded against repository reality

**Decision:** The independent reviewer is a grounded adversary rather than a
spec-only proofreader. Every review now starts by checking the spec's
current-state and change claims against repository evidence, sweeps whole-tree
impact targets derived from the spec, confronts area-intersecting ADR decisions
and pitfalls, and then runs code-quality checks only when an implementation diff
exists. The public labels stay `plan-quality mode` and `post-implementation
mode`; internally they differ only by whether `S3 Code Quality` is present.

**Why:** A spec-only reviewer cannot catch conflicts that exist outside the spec
folder. The retire-patch work exposed this failure mode: workflow wording,
lint behavior, conformance-test pinning, and active pitfall records all mattered
to the correctness of the change, but none were visible to a reviewer that
judged only the authored spec artifacts. The reviewer should apply the same
code-is-source-of-truth rule that MochiFlow asks other agents to follow.

**Key sub-decisions:**
- Findings carry `Confidence: confirmed | predicted`. Confirmed findings are
  verified against repository evidence and may block; predicted findings are
  still grounded but capped so plan-time concerns do not over-block work that a
  correct implementation can avoid.
- ADR records are loaded on demand through the reviewer's read capability,
  using each store's derived `INDEX.md` first. ADR indexes are not static Kiro
  resources because they are project-local, gitignored caches that may be
  absent in consuming repositories.
- The Kiro reviewer keeps `tools: ["read"]`. The `read` category is enough for
  file reads, directory inspection, and search; adding fine-grained or write /
  shell tools would either be unknown to Kiro or widen the reviewer beyond its
  read-only role.

**Rejected:** keeping the old two-branch reviewer structure and bolting
grounding onto only plan-quality mode (preserves the spec-only blind spot);
requiring S3 for code-less specs (creates noisy `N/A` work instead of a useful
mode distinction); statically listing ADR `INDEX.md` files as Kiro resources
(couples generation to derived caches); allowing predicted findings to reach
High/Critical (turns implementation-avoidable risks into hard blocks).
