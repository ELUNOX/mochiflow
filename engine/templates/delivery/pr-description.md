# PR Description Template

Write the PR description for external reviewers in the configured artifact
language. Do not include spec files, internal IDs (`slug`, `AC-01`, etc.), or
MochiFlow vocabulary (`risk: standard`, etc.).

```markdown
## Summary

{Purpose of this change in 1-3 lines, including why it is needed.}

## Changes

- {Logical change unit: what changed and how.}

## Verification

- {Verification command and result.}
- {One-line summary of human confirmation when applicable.}

## Risk

- {Natural artifact-language description of reversibility, schema/contract impact, and affected area in 1-2 lines.}

## Testing

- {For each human-operated or visual QA scenario in spec.md QA Scenarios, list: scenario name, concrete reproduction steps, and expected result. Omit internal IDs (QA-XX, AC-XX). If all QA is automated, state "Automated verification only — see Verification above."}
```
