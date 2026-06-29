---
slug: "workflow-state-transition-scenario-tests"
title: "Add adversarial workflow state-transition scenario tests"
surface: "cli"
type_hint: "improvement"
maturity: "seed"
source: "conversation"
source_spec: "build-orchestrator-subagent-execution"
source_phase: "review"
created: "2026-06-29"
updated: "2026-06-29"
---

# Add adversarial workflow state-transition scenario tests

## Signal

The delegated build orchestrator review found multiple bugs that prose
conformance tests did not catch. The failures appeared at phase boundaries and
resume points, not in isolated wording checks.

## Why It Matters

MochiFlow's engine contracts are mostly prose, so string-based conformance can
prove that required phrases exist while missing impossible or stale lifecycle
states. State-transition scenarios would test the workflow as an executable
system contract and catch regressions where `build`, `open`, `update`, reviewer
freshness, task checkboxes, trailers, and AC Matrix settlement drift apart.

## Evidence

- Review of `build-orchestrator-subagent-execution` found issues around
  `approved` build versus `accepted` open/update rework, stale reviewer verdicts
  after code changes, and all-tasks-checked resume before final Matrix settlement.
- Useful adversarial scenarios include approved build, accepted open QA-`FAIL`
  rework, accepted update PR-feedback fix, and all tasks checked while the AC
  Matrix / final verification / reviewer verdict is still unsettled.
- Existing conformance tests heavily assert engine-doc wording; they need
  complementary scenario tests that construct repository states and verify the
  intended next action or stop condition.

## Open Questions

- Should these scenarios live as CLI conformance tests, unit tests over a
  workflow-state helper, or both?
- What is the minimal state fixture needed to model `approved`, `accepted`,
  `in_review`, task trailers, and reviewer freshness without building a full
  workflow runner?
- Should stale reviewer verdict detection become a deterministic lint/doctor
  check, or remain a scenario-tested procedural contract?
