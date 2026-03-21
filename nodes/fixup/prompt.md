Goal: Implement the next approved `chain:pallet` slice.

Inputs:
- `pallet/spec.md`
- `pallet/review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `chain:pallet`

Required curated artifacts:
- `pallet/implementation.md`
- `pallet/verification.md`
- `pallet/quality.md`
- `pallet/promotion.md`
- `pallet/integration.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
test -f ./outputs/chain/pallet/implementation.md && test -f ./outputs/chain/pallet/verification.md && test -f ./outputs/chain/pallet/quality.md && test -f ./outputs/chain/pallet/promotion.md && test -f ./outputs/chain/pallet/integration.md
true`
  - Stdout: (empty)
  - Stderr: (empty)
- **implement**: fail
- **verify**: fail
  - Script: `test -f ./outputs/chain/pallet/implementation.md && test -f ./outputs/chain/pallet/verification.md && test -f ./outputs/chain/pallet/quality.md && test -f ./outputs/chain/pallet/promotion.md && test -f ./outputs/chain/pallet/integration.md`
  - Stdout: (empty)
  - Stderr: (empty)

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n>


# Pallet Restart Implementation Lane — Fixup

Fix only the current slice for `pallet-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
