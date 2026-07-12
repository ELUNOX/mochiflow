---
id: 2026-07-11-codebase-audit-remediation
date: 2026-07-11
area: [cli]
spec: codebase-audit-remediation
status: active
---
## 2026-07-11 - Use checked witnesses and a pinned CI hardening boundary

**Decision:** Repository-owned configured and adapter paths use one two-part
boundary: lexical validation plus a canonical containment witness. Filesystem
operations retain the original repository-root-joined path after validation so
valid local symlinks keep normal read, write, and deletion semantics.

Release automation keeps cargo-dist 0.32 as the generated workflow owner while
the repository maintains a small, conformance-pinned security delta for
least-privilege permissions, locked installation, and tag provenance. The
official `allow-dirty = ["ci"]` setting makes that ownership split explicit.

**Why:** Lexical checks alone cannot detect symlink escapes, while replacing
operation paths with canonical targets would change deletion behavior. For CI,
fully forking generated output would lose upstream maintenance, but accepting
generator defaults would retain broader permissions and an unsafe installer
path. These bounded seams preserve compatibility while failing closed.

**Rejected:** banning every symlink; using canonical paths for mutation;
silently trusting missing or malformed engine manifests; hand-owning the whole
release workflow; weakening provenance checks to keep generated CI pristine.
