---
id: 2026-07-10-reviewer-core-composition
date: 2026-07-10
area: [cli]
spec: engine-context-slimming-redesign
status: active
supersedes: 2026-07-01-reviewer-profile-split
---
## 2026-07-10 - Compose review profiles through a shared core

**Decision:** Retain `plan-auditor` and `change-reviewer` as the canonical
read-only profiles, but place their shared grounding, whole-tree impact, ADR
confrontation, falsification, finding shape, and verdict rules in
`reviewer-core.md`. Each profile keeps only its target-specific stages and
declared policy inputs. The legacy independent-reviewer wrapper is removed, and
generated Kiro resources point to the core plus the selected profile.

**Why:** The profile split remains useful, but duplicating the common adversarial
contract caused broad, drifting reviewer bootstrap context. Composition makes
the common safety model authoritative once while preserving the distinct plan and
implementation review responsibilities.

**Rejected:** restoring the legacy wrapper as a compatibility alias (the staged
engine upgrade already replaces obsolete internal paths); merging the two
profiles back into one mode-based reviewer (obscures their different targets);
allowing reviewers to mutate code or workflow state (breaks independent review).
