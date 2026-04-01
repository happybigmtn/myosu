# Implementation Playbook

## Goal

Deliver a code slice with verification and any necessary artifact updates.

## Inputs

- relevant plan file
- current crate or module source
- current tests and build commands

## Steps

1. Orient on the current code and plan.
2. Identify the smallest honest slice.
3. Implement the change.
4. Run targeted verification.
5. Update any affected docs or output artifacts.

## Proof

- code builds or tests as intended
- the targeted behavior is covered by a command or test
- docs do not over-claim beyond the shipped behavior

## Example lane

- `crates/myosu-games`
