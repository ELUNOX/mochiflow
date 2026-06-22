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
  - Files: `path/to/File.ext`
  - Done: {objective, checkable completion condition}
  - Stop: {task-specific stop condition; see Defaults for shared ones}
- [ ] T-002 [P] [AC-02] {task title}
  - Depends on: none
  - Files: `path/to/Other.ext`
  - Done: {objective, checkable completion condition}
  - Stop: {task-specific stop condition}

<!--
Task line: `- [ ] T-### [AC-01] title`. The reference in brackets is required and
must be an AC (`[AC-01]`), an NFR (`[NFR-01]`), or a chore reason (`[chore: ...]`).
Add `[P]` after the ID for a task that runs parallel to the previous `[P]` block;
never `[P]` two tasks that edit the same file. Each task needs Depends on / Files /
Done / Stop. `Depends on:` lists prior `T-###` IDs or `none`.
Create the ## AC Verification Matrix here during plan (one row per AC) so it is present at approval; record verification results during build.
-->
