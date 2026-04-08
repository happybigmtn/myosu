# ADR 011: Close Stage-0 Emission Split Dust in the Validator Bucket

- Status: Accepted
- Date: 2026-04-08
- Deciders: Myosu maintainers
- Consulted: `IMPLEMENTATION_PLAN.md`, `WORKLIST.md`, `specs/070426-emission-yuma-consensus.md`, `specs/050426-emission-epoch-mechanism.md`
- Informed: chain, validator, operator, and release-gate contributors
- Related: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`, `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`, `tests/e2e/emission_flow.sh`

## Context

Stage-0 coinbase already computes an integer `alpha_created` budget for each
emitting subnet, but the live split path used independent `U96F32 -> u64`
floors for owner, server, and validator writes. That leaked up to 2 rao per
accrued block from the owner/server/validator split, which showed up as:

- `cargo test -p pallet-game-solver -- truncation --quiet` measuring a
  worst-case 6-rao gap over the default tempo-2 epoch
- `bash tests/e2e/emission_flow.sh` reporting `distribution_rounding_loss=6`
- `try_state` tolerating a 1_000-rao `TotalIssuance` delta even though the
  measured stage-0 drift was much smaller

The queue task `EMIT-001` required a real dust policy instead of leaving that
gap as an alert-only threshold. The authoritative emission spec names three
candidate families: accept the loss, accumulate dust in a separate account, or
rotate the residual across recipients.

Doing nothing would keep a correctness gap in the no-ship emission accounting
story and force `try_state` to stay far looser than the live code path needs.

## Decision

Myosu closes the stage-0 emission split remainder deterministically inside the
coinbase path:

- owner, server, and root buckets still use the existing fixed-point floor
  conversion
- validator alpha becomes the residual integer bucket:
  `alpha_created - owner - server - root`
- `try_state` tightens to a 1-rao alert delta because stage-0 no longer relies
  on silent per-block dust loss

This decision only covers the stage-0 coinbase split in
`pallet-game-solver`. It does not introduce a treasury, does not add new
storage, and does not change Yuma math, swap behavior, or subnet epoch
mechanics.

## Alternatives Considered

### Option A: Close the split remainder in the final integer bucket

This won because the code already knows the exact integer emission budget per
block. Reassigning the remainder keeps the budget closed without inventing a
new account, a rotation schedule, or migration state. The validator bucket is
the natural residual bucket in the live split: owner is carved out first, root
is carved out from the validator half, and validator alpha is whatever remains.

### Option B: Accept the loss and only tighten the alert threshold

This was rejected because the threshold would still be masking known,
intentional token destruction. A smaller alert number would be better than
1_000 rao, but it would still leave the core stage-0 emission invariant
"almost true" instead of true.

### Option C: Accumulate dust in a dedicated treasury or dust account

This was rejected for stage-0 because it would add new storage, accounting
surfaces, and operator explanation burden just to preserve a tiny remainder the
current split can already assign deterministically.

### Option D: Round-robin the residual across recipients

This was rejected because it needs rotation state and makes the proof surface
harder to reason about. Stage-0 benefits more from a static, derivable rule
than from micro-fairness across 1-2 rao remainders.

## Consequences

### Positive

- Stage-0 emission accounting now closes exactly on the owned pallet and E2E
  proof surfaces.
- `try_state` can alert on real accounting drift instead of carrying a large
  dust allowance.
- The policy stays deterministic and consensus-friendly without new storage or
  hidden balances.

### Negative

- Validators receive the tiny integer remainder from the split instead of the
  loss disappearing or being routed elsewhere.
- If Myosu later changes the emission split structure materially, this rule
  should be revisited rather than assumed to generalize automatically.

### Follow-up

- Keep `tests/e2e/emission_flow.sh` and the pallet accounting tests aligned
  with the exact-budget contract.
- Reopen the policy if stage-1 introduces non-zero root-alpha selling, a real
  treasury, or multi-token accounting that makes the residual bucket ambiguous.

## Reversibility

Easy.

The change is local to the coinbase split. Reversal would mean changing one
integer-allocation rule plus the associated tests and ADR. A future treasury or
rotation policy can supersede this ADR cleanly if new token-economics work
creates a better home for residual emission.

## Validation / Evidence

- `cargo test -p pallet-game-solver --quiet -- truncation`
- `cargo test -p pallet-game-solver --quiet -- stage_0_coinbase_emission_accounting_matches_accrued_epoch_budget`
- `bash tests/e2e/emission_flow.sh`
