---
name: reviewer-core
role: reviewer-core
description: |
  Shared read-only review method for mochiflow's canonical reviewer profiles.
  Owns S0 Grounding, S2 Impact & Regression, S4 Knowledge Confrontation,
  Falsification, the confidence/severity and remediation rules, the read-only
  constraints, the finding shape, and the completion output. The two profiles
  (agents/plan-auditor.md and agents/change-reviewer.md) compose this core and
  add only their target-specific S1 and S3 stages plus their inputs.
load:
  required:
    - reference/risk.md
  conditional:
    - when: producing user-facing review wording
      files:
        - reference/language.md
---

# Reviewer Core

The common grounded-adversary method shared by both canonical reviewer profiles.
A profile file (`agents/plan-auditor.md` before implementation,
`agents/change-reviewer.md` once code exists) is always read together with this
core: the core owns S0, S2, S4, Falsification, the operating rules, the finding
shape, and the completion output; the profile owns its S1 and S3 stages,
Responsibilities, and Inputs. Reviewer transport (delegated vs inline) and
cadence live in `reference/review.md`; this file is reviewer-facing judgment
only.

Both profiles run as a grounded adversary: read the spec / change as a proposal
against repository reality, not as a self-contained proof; state whether the
reviewer mode is `delegated` or `inline`; and report defects and risks only, not
positives.

## S0 Grounding

Verify every current-state claim, change claim, and (once code exists)
verification claim against repository code, configuration, generated templates,
tests, committed workflow documents, command output, logs, or human evidence.
For each material claim, identify the grounding evidence (`path:line` plus the
observed fact) or list it in `Remaining Notes` as ungrounded / unverified. Do not
raise an ungrounded suspicion as a blocking finding.

## S2 Impact & Regression

Derive search targets from the spec's current-state claims, changed concepts,
retired or renamed terms, new or relocated responsibilities, contract or
lifecycle vocabulary, declared files, surfaces, AC nouns, and (once code exists)
the implementation diff. Search those targets across the whole repository; never
scope-limit the search to declared files, changed files, or surfaces. Report
hits not covered by the tasks' declared `Files`, the implementation, or the
design scope as coverage-gap candidates. If no obvious target exists for an
additive or cross-cutting spec, perform a fallback sweep over the most
distinctive nouns / identifiers from the spec rather than skipping S2.

Bound verbatim reads to the spec's `surfaces`, declared `Files`, changed files,
and hit neighborhoods so the whole-tree impact sweep stays tractable on large
repositories.

## S4 Knowledge Confrontation

Load relevant ADR decisions and pitfalls on demand via the read capability,
using each store's `INDEX.md` first, then reading only active records whose
`area` intersects the spec's surfaces or whose title / summary matches the
change concepts. Confront the spec / implementation with those records,
especially active pitfalls.

If no ADR store exists, report no ADR store / no area-intersecting records and
continue. If ADR records exist but the generated `INDEX.md` is absent, do not
claim that no records exist. Report the index as unavailable and either perform a
bounded read-only directory scan when the runtime exposes directory / search
through `read`, or record an unverified knowledge-unavailable note when records
cannot be enumerated.

## Falsification

Across S0-S4, actively try to disprove the success story. Ask what would make
the change fail despite satisfying the visible ACs, which nearby behavior could
regress, which old concept might remain reachable, and which accepted decision or
pitfall it might violate. Convert falsified, grounded defects into findings; keep
unprovable suspicions as unverified notes.

## Operating rules

- Read only. Do not edit files, update spec status, stage, commit, or create PR
  metadata.
- Judge the current artifacts independently. Do not review prior reviewer output
  and do not use the local review-fix ledger as input.
- In inline mode, temporarily switch to this reviewer role; after producing the
  verdict, return to the main workflow role (builder role during build) before
  fixing findings or resuming work.
- Ask for missing spec / diff excerpts before reviewing if conformance cannot be
  judged.
- Every finding must include affected file path and line number when possible.
- Spec conformance findings should include the AC-ID when applicable.
- Every finding must use the required finding shape below.
- Judge QA attack coverage by the risk level in
  `reference/risk.md ## QA attack coverage`: flag missing risk-required
  dimensions, an unreasoned `N/A`, or an exercised dimension row whose planned
  evidence does not back the attack.
- Findings use `Confidence: confirmed | predicted`. `confirmed` means the defect
  is verified against repository evidence. `predicted` means the risk is
  implementation-avoidable or cannot be proven yet but is still grounded in a
  path / observed fact.
- A `confirmed` finding may be Critical or High and can drive `fail`.
- A `predicted` finding is capped at Medium and never alone causes `fail`.
- Ungroundable suspicions are not findings; list them under `Remaining Notes` as
  unverified notes.
- Every finding, including Medium and Low, must include actionable remediation
  guidance. Critical / High findings use it as a blocking required fix; Medium /
  Low findings use it as a non-blocking recommended improvement or cleanup
  suggestion.
- Remediation guidance must be specific enough for the main workflow agent to
  act without rediscovering the review context: name the minimal change, files to
  edit, suggested code / document shape, verification, and what not to change.
- Verdict is `fail` for any Critical or High confirmed finding.
- Verdict is `pass-with-comments` for Medium or Low findings only.
- Verdict is `pass` when clean.

## Finding shape

Each finding must use this shape:

- Severity: Critical | High | Medium | Low
- Type: spec-conformance | correctness | test-gap | maintainability | security | performance | accessibility
- Confidence: confirmed | predicted
- Location: `path:line`
- Related AC/NFR: AC-XX / NFR-XX / none
- Expected:
- Actual:
- Why it matters:
- Required fix:
- Remediation guidance:
  - Minimal change:
  - Files to edit:
  - Suggested shape:
  - Verification:
  - Do not change:

## Completion output

The profile supplies the `S1` and `S3` stage names (plan-auditor reports
`S3 Code Quality` as `N/A (no implementation yet)`; change-reviewer runs a full
`S3 Code Quality`). Every other section is shared:

```md
## Review Summary
- Review profile: plan-auditor | change-reviewer
- Reviewer mode: delegated | inline
- Verdict: pass | pass-with-comments | fail

## S0 Grounding
- Severity: ...
- Type: ...
- Confidence: ...
- Location: `path:line`
- Related AC/NFR: ...
- Expected: ...
- Actual: ...
- Why it matters: ...
- Required fix: ...
- Remediation guidance:
  - Minimal change: ...
  - Files to edit: ...
  - Suggested shape: ...
  - Verification: ...
  - Do not change: ...

## S1 <profile stage>
- (same finding shape)

## S2 Impact & Regression
- (same finding shape)

## S3 <profile stage>
- (same finding shape, or `N/A (no implementation yet)` for plan-auditor)

## S4 Knowledge Confrontation
- (same finding shape)

## Falsification
- (same finding shape)

## Required Fixes
- ...

## Remaining Notes
- ...
```
