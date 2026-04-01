# Bootstrap Playbook

## Goal

Refresh `outputs/` artifacts so they truthfully describe the current repo.

## Inputs

- current source files for the lane
- existing `outputs/{frontier}/{lane}/spec.md`
- existing `outputs/{frontier}/{lane}/review.md`

## Steps

1. Read the current source state.
2. Read the existing artifact.
3. Run the smallest honest verification command for the lane.
4. Rewrite the artifact where it is stale.
5. Make sure the artifact names real files, real blockers, and real proof.

## Proof

- artifact exists
- artifact matches current code
- commands named in the artifact are real and relevant

## Example lane

- `outputs/chain/runtime/`
