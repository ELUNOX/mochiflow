---
id: 2026-06-28-conformance-substring-line-wrap
date: 2026-06-28
area: [cli]
status: active
---
## Engine-doc conformance tests match literal substrings across newlines (2026-06-28)

**Applies to:** any change that edits `engine/**` Markdown whose wording is
pinned by `cli/crates/mochiflow-cli/tests/conformance.rs` (the doc-scanning
"engine prose drift guards"), and the tests authored alongside such a change.

**Signal:** a conformance assertion like `doc.contains("some required phrase")`
fails even though the phrase is visibly present in the engine doc — because the
phrase was wrapped across a line break (e.g. `no second\ntransport`), so it is
not a contiguous substring of the file (which includes newlines).

**Cause:** these tests read the whole file and do a literal `str::contains`. Hard
line wrapping inside a test-asserted phrase splits it; Markdown rendering hides
the break, so it is easy to miss. The same bites a rewrapped sentence whose
asserted phrase straddles the new wrap point (observed: `no-PR fast path branch
choice` broke an unrelated pre-existing test after a frontmatter rewrap).

**Guardrail:** keep any phrase a conformance test asserts on a **single
unwrapped line** in the engine doc. When writing the assertion, prefer a short,
unambiguous substring that is unlikely to wrap; if matching a longer idea, assert
two short contiguous tokens rather than one long phrase. After rewrapping any
engine prose, re-run the full conformance suite (not just the new test) because a
rewrap can break a pre-existing assertion on a neighboring phrase.

**Check:** `cargo test --manifest-path cli/Cargo.toml --test conformance` is
green after every engine-doc edit, including rewraps.

**Status:** Active.
