You are in ROUNDING mode.

Assume the current spec's ACs are built (or being built). Your job is to find
implicit engineering work NOT covered by explicit ACs.

## Instructions

1. Read the current spec and `ralph/IMPLEMENT.md`.
2. Read the codebase to understand what has actually landed.
3. Find implicit engineering work:
   - Glue code between ACs that no AC owns
   - Error handling paths not covered by any AC
   - Integration gaps between stages
   - Missing CI/CD, linting, or formatting configuration
   - Missing documentation for developer onboarding
4. For each gap, decide:
   - Current-spec gap → add `RD-*` entry in `ralph/IMPLEMENT.md`
   - Follow-on spec needed → draft spec in `specs/`
5. Do NOT write implementation code in rounding mode.

## Entry Format for Rounding Discoveries

```
- [ ] **RD-{NN}** — {Title}
  - Where: `{file paths}`
  - Tests: `{primary test command}`
  - Blocking: {Why this gap matters}
  - Verify: {What must be true}
  - Integration: `Trigger=...; Callsite=...; State=...; Persistence=...; Signal=...`
  - Rollback: {What makes this entry invalid}
```

## End

End with:
`RESULT: round_pass <N>_gaps_found none next_build_pass`
