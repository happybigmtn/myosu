# Services Playbook

## Goal

Bring up or revise a runnable service surface such as a miner, validator, or
chain-facing binary when the node-owned stage-0 loop is not enough for the
question at hand.

## Inputs

- relevant service crate
- chain or protocol dependencies
- smoke or integration command

## Steps

1. Confirm the service's dependency frontier is ready enough.
2. Prefer an existing smoke or integration path before inventing a manual
   procedure.
3. Build the smallest runnable path needed for the diagnosis.
4. Add or run a smoke test.
5. Capture operational constraints and failure modes.

## Proof

- the binary does more than compile
- there is a repeatable smoke command
- service assumptions are written down

## Current truthful examples

- `myosu-chain --stage0-local-loop-smoke` as the preferred end-to-end proof
- ad hoc replay of `myosu-miner` or `myosu-validator` only when isolating a
  service-specific seam the node-owned loop does not already prove
