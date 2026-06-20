# {title} — Tasks

<!--
This template is structural. When rendering a real artifact, translate
human-facing headings and prose to the configured project language.
Preserve machine-readable IDs and enum values such as AC-01, QA-01, T-001,
NFR-01, UNVERIFIED, PASS, PENDING_HUMAN, HUMAN_CONFIRMED, N/A: <reason>, FAIL.
Remove template-only Rules blocks unless the project intentionally keeps them.
-->

## Execution Rules

- `[x]` may only be set after implementation, verification, and required commit are complete.
- Do not mark a task `[x]` before verification passes.
- Each task must reference at least one AC, NFR, or chore reason.
- Each top-level task should be a reviewable commit unit when practical.
- `[P]` means the task is parallelizable within the same wave.
- Do not mark tasks `[P]` if they edit the same files.
- Do not mark tasks `[P]` if one depends on the other.
- If new design decisions are required, stop and update `design.md`.
- If scope changes are required, stop and update the spec through the proper flow.
- If verification repeatedly fails, stop and record the failure.

## Defaults

- Verify: {verify command}
- Common stop conditions:
  - scope outside this spec is required
  - AC changes are required
  - verification fails repeatedly
  - migration / rollback decision is missing
  - required human input is unavailable

## Checklist

### Wave 1 — Foundation

- [ ] T-001 [AC-01] {task title}
  - Type: implementation
  - Depends on: none
  - Files:
    - `path/to/file`
  - Done:
    - [ ] Existing pattern reviewed
    - [ ] Implementation completed
    - [ ] Tests added or updated
    - [ ] Verification passed
    - [ ] AC Matrix updated
  - Stop:
    - {task-specific stop condition}

- [ ] T-002 [P] [AC-02] {task title}
  - Type: test
  - Depends on: none
  - Files:
    - `path/to/test`
  - Done:
    - [ ] Failing case reproduced or added
    - [ ] Passing verification confirmed
    - [ ] AC Matrix updated
  - Stop:
    - {task-specific stop condition}

### Wave 2 — Integration

- [ ] T-003 [AC-01, AC-02] {task title}
  - Type: integration
  - Depends on: T-001, T-002
  - Files:
    - `path/to/file`
  - Done:
    - [ ] Integration behavior implemented
    - [ ] Regression coverage added or confirmed
    - [ ] Verification passed
    - [ ] AC Matrix updated
  - Stop:
    - {task-specific stop condition}

## Finalization

- [ ] T-900 [chore: verification] Complete AC Matrix
  - Type: verification
  - Depends on: all implementation tasks
  - Files:
    - `{spec path}/spec.md`
  - Done:
    - [ ] All AC rows have Implementation
    - [ ] All AC rows have Result
    - [ ] Evidence is recorded where required
    - [ ] No UNVERIFIED automated AC remains
  - Stop:
    - Matrix cannot be completed from available evidence.

<!-- Include T-901 only when risk is elevated/critical or reviewer policy requires review.

- [ ] T-901 [chore: review] Complete required review
  - Type: review
  - Depends on: T-900
  - Files:
    - `{spec path}/design.md`
  - Done:
    - [ ] Required reviewer result recorded
    - [ ] High/Critical findings resolved or explicitly blocked
  - Stop:
    - Required review cannot be completed.
-->

<!-- Authoring rules:

- Use task IDs `T-001`, `T-002`, etc.
- Use finalization IDs `T-900+` for close-out tasks.
- A task without AC/NFR must include a chore reason.
- Keep tasks small enough for one agent context, but not so small that the checklist becomes noisy.
- For critical specs, prefer smaller independently verifiable tasks.
- For standard specs, prefer coherent commit units.
-->
