# Planning Playbook

## Goal

Turn a broad objective into a sequence of executable, verifiable slices.

## Inputs

- active plan files
- current repo state
- known blockers

## Steps

1. Compare the plan to the repo.
2. Remove assumptions that are no longer true.
3. Order the work by dependency and verification value.
4. Name artifact gates for each phase.

## Proof

- the sequence matches real dependencies
- each step has a concrete output or verification command

## Example lane

- revising Phase 0 to focus on truthfulness before chain restoration
