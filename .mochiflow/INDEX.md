# 📋 Spec Dashboard

> updated: 2026-06-26 23:43 UTC

## Pipeline

| stage | count |
|:------|------:|
| 🌱 backlog seed | 9 |
| 📝 draft | 0 |
| 🟢 approved | 1 |
| ✅ done | 15 |

## Backlog seeds

| Slug | Title | Maturity | Source |
|:-----|:------|:---------|:-------|
| ["active-spec-open-directory"](specs/_backlog/"active-spec-open-directory".md) | "Introduce an _open directory for active specs" | ❓ "seed" | "conversation" |
| ["develop-branch-workflow"](specs/_backlog/"develop-branch-workflow".md) | "Introduce develop branch to reduce PR ceremony for non-code changes" | ❓ "seed" | "conversation" |
| ["diagnostics-command-surfaces-hardening"](specs/_backlog/"diagnostics-command-surfaces-hardening".md) | "Harden diagnostics command surfaces and source-repo guidance" | ❓ "seed" | "conversation" |
| ["freeze-hardening"](specs/_backlog/"freeze-hardening".md) | "Freeze module hardening: error types, format stability, visibility" | ❓ "seed" | "conversation" |
| ["frozen-surface-ssot"](specs/_backlog/"frozen-surface-ssot".md) | "Single source for frozen-surface input set definition and test performance" | ❓ "seed" | "conversation" |
| ["parallel-spec-context-switch"](specs/_backlog/"parallel-spec-context-switch".md) | "Support parallel spec work with explicit context switching" | ❓ "seed" | "conversation" |
| ["phase-completion-guidance"](specs/_backlog/"phase-completion-guidance".md) | "Every phase should present the next action clearly" | ❓ "seed" | "conversation" |
| ["ship-handoff-recovery-and-cleanup"](specs/_backlog/"ship-handoff-recovery-and-cleanup".md) | "Improve ship handoff recovery and post-merge cleanup" | ❓ "seed" | "conversation" |
| ["spec-contract-lint-hardening"](specs/_backlog/"spec-contract-lint-hardening".md) | "Harden lint checks for approved task scope and verification evidence" | ❓ "seed" | "conversation" |

## Active specs

| Spec | Status | Risk | Docs | Module |
|:-----|:-------|:-----|:-----|:-------|
| [lint-deleted-files-in-tasks](specs/lint-deleted-files-in-tasks/) | 🟢 approved | elevated | spec+design+tasks | — |

## Done (chronological)

### 2026-06

| Updated | Slug | Title | Type |
|:--------|:-----|:------|:-----|
| 2026-06-26 | [choice-card-command-ux](specs/_done/choice-card-command-ux/) | Clarify choice-card commands and numbered replies | docs |
| 2026-06-26 | [manifest-test-isolation](specs/_done/manifest-test-isolation/) | Isolate MANIFEST integrity check from functional conformance tests | refactor |
| 2026-06-25 | [qa-attack-matrix](specs/_done/qa-attack-matrix/) | Add a QA attack matrix to plan and reviewer flows | feature |
| 2026-06-25 | [lint-residue-and-multiline-ears](specs/_done/lint-residue-and-multiline-ears/) | Lint: detect template residue and multi-line EARS ACs | feature |
| 2026-06-25 | [doctor-freeze-coherence](specs/_done/doctor-freeze-coherence/) | Clarify doctor/freeze coherence and context freshness | feature |
| 2026-06-24 | [prevent-build-phase-spec-mutation](specs/_done/prevent-build-phase-spec-mutation/) | Prevent build-phase spec mutation | fix |
| 2026-06-24 | [verify-ci-parity](specs/_done/verify-ci-parity/) | Verify profile should cover CI lint checks | fix |
| 2026-06-24 | [kiro-adapter-default-agent](specs/_done/kiro-adapter-default-agent/) | Kiro adapter: drop dedicated agent, single always-on steering, delegate permissions | refactor |
| 2026-06-23 | [commit-lifecycle-unification](specs/_done/commit-lifecycle-unification/) | Unify commit timing across discuss/plan/build/ship on a single branch | refactor |
| 2026-06-23 | [ac-matrix-token-normalization](specs/_done/ac-matrix-token-normalization/) | Normalize AC Matrix result tokens to ASCII-only | refactor |
| 2026-06-23 | [commit-trailer-traceability](specs/_done/commit-trailer-traceability/) | Add git trailers for spec/task traceability and AI log reading recipes | docs |
| 2026-06-23 | [plan-to-build-transition-ux](specs/_done/plan-to-build-transition-ux/) | Present next-step choices after plan approval | docs |
| 2026-06-23 | [ship-qa-experience](specs/_done/ship-qa-experience/) | Unify QA experience in ship: single-source scenarios, round-trip protocol, PR Testing section | feature |
| 2026-06-23 | [build-completion-guidance](specs/_done/build-completion-guidance/) | Build phase should announce next step on completion | docs |
| 2026-06-22 | [version-ssot-freeze](specs/_done/version-ssot-freeze/) | Version SSOT + freeze command | refactor |

> done total: 15
