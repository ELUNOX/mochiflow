# {title} — Tasks

Implementation Summary: {one line}
risk: {standard|elevated|critical}
Critical Stop Conditions:
- {1-3 spec-specific stop conditions}

## Defaults

- Verification: {shared verification command}
- Shared stop conditions: out-of-scope change / new design decision needed / verification keeps failing

## Tasks

- [ ] T-001 [AC-01] {task title}
  - Depends on: none
  - Files:
    - `path/to/File.ext`
    - deleted: `path/to/RemovedFile.ext`
  - Done: {objective, checkable completion condition}
  - Stop: {task-specific stop condition; see Defaults for shared ones}
- [ ] T-002 [P] [AC-02, AC-03] {task title}
  - Depends on: none
  - Files:
    - `path/to/Other.ext`
  - Done: {objective, checkable completion condition}
  - Stop: {task-specific stop condition}

<!--
Task line: `- [ ] T-### [AC-01] title`. The reference in brackets is required and
must be one or more AC IDs (`[AC-01]`, `[AC-01, AC-02]`), an NFR (`[NFR-01]`),
or a chore reason (`[chore: ...]`). Use a compound AC reference when one task
naturally covers multiple related ACs; do not split tasks just to force one task
per AC.
Add `[P]` after the ID for a task that runs parallel to the previous `[P]` block;
never `[P]` two tasks that edit the same file. Each task needs Depends on / Files /
Done / Stop. `Depends on:` lists prior `T-###` IDs or `none`. Use normal `Files:`
paths for planned creates/edits and ``deleted: `path` `` for planned deletions.
The marker applies to every path parsed from that line; prefer one deleted path
per line for readability.
Create the ## AC Verification Matrix here during plan (one row per AC) so it is present at approval; record verification results during build.

## Verification Plan / AC Matrix

| AC | Scope | Verification method | Planned test/QA | Implementation | Result | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AC-01 | {surface} | automated | `command ...` | `path/File.ext` | UNVERIFIED | | |

-->
