# Review Playbook

## Goal

Decide whether a slice is actually complete.

## Inputs

- changed code
- changed docs or outputs
- verification command results

## Steps

1. Check the highest-risk surfaces first.
2. Verify proof commands against the actual acceptance claim.
3. Downgrade any compile-only or placeholder proof.
4. Record remaining risks plainly.

## Proof

- no claimed behavior lacks a matching verification path
- risks and gaps are explicit

## Example lane

- reviewing whether a bootstrap artifact is current enough to trust
