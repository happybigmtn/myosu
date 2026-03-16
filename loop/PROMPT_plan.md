You are in PLANNING mode. Do NOT implement any code.

## Instructions

1. Read `ralph/SPEC.md` — the definitive format and process reference.
2. Read specs: ${SPEC_FILTER}
3. Read the current `ralph/IMPLEMENT.md` (if it exists).
4. Read relevant source code to understand current implementation state.
5. Perform gap analysis: what do the specs require that doesn't exist yet?
6. Create or update `ralph/IMPLEMENT.md` with a prioritized task list
   following the format in `ralph/SPEC.md` Part 2. DO NOT DELETE EXISTING ENTRIES,
   just update the plan for any additional entries you would create

## Entry Format (6 fields — all mandatory)

Each plan entry has exactly 6 fields per `ralph/SPEC.md` §2.3:

```
- [ ] **{PREFIX}-{NN}** — {Title}
  - Where: `{file paths}`
  - Tests: `{primary cargo test command}`
  - Blocking: {One sentence: why this must exist}
  - Verify: {Compressed pass/fail — key behaviors, semicolons between assertions}
  - Integration: `Trigger=...; Callsite=...; State=...; Persistence=...; Signal=...`
  - Rollback: {Compressed rollback condition}
```

- **Integration** is the compressed wiring contract. It prevents building
  isolated components that pass unit tests but never get called.
- Use `Integration: N/A (reason)` only for non-runtime tasks (docs, lint, CI).
- Checkbox states: `[ ]` pending, `[x]` complete, `[!]` blocked.

## Completion criteria (from SPEC.md §5.4)

A completed `[x]` entry MUST have all 5 wiring fields filled by the build
agent: Trigger, Callsite, State effect, Persistence effect, Observable signal.
Missing wiring fields = task remains `[ ]`.

## Rules

- Output a prioritized TODO list only. No implementation.
- Priority order: foundation dependencies first, then by unblock value.
- If the plan already exists and is mostly correct, update it — don't
  regenerate from scratch. DO NOT DELETE ANYTHING.
- Mark completed items `[x]`, blocked items `[!]`, pending `[ ]`.
- Group entries by stage with `Source spec:` references.
- Include the Spec Index table in the plan header.

## End

End with:
`RESULT: plan_update gap_analysis_complete none next_planning_pass`
