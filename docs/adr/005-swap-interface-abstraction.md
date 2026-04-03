# ADR 005: Stage-0 Swap Interface Abstraction

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `ops/decision_log.md`, `AGENTS.md`, `THEORY.MD`, `specs/040226-02-single-token-emission-accounting.md`
- Informed: runtime, pallet, and operator contributors
- Related: `crates/myosu-chain/pallets/game-solver/src/lib.rs`, `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs`, `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/pallets/game-solver/src/staking/stake_utils.rs`

## Context

This is a retroactive record of the stage-0 swap seam now carried by the live
runtime and pallet.

The inherited subtensor fork was more deeply entangled with swap behavior than
the original plan assumed. Registration, staking, emission, and runtime APIs
all touched swap-oriented interfaces. Myosu's stage-0 product, however, does
not run an AMM. It only needs a consistent answer for pricing and conversion:
identity, zero fee, no liquidity.

Inlining those assumptions at every callsite would have made the chain harder
to reason about and harder to replace later. Keeping the full inherited swap
surface everywhere would have overstated what stage 0 really supports.

## Decision

Myosu keeps an explicit stage-0 swap abstraction and binds it to a noop runtime
implementation.

That decision includes:

- a pallet-local swap seam that exposes only the operations the carried stage-0
  pallet still needs
- a runtime `Stage0NoopSwap` implementation that returns identity conversions,
  zero fees, and inert liquidity operations
- stage-0 proofs that pin the outward runtime behavior at the surfaces callers
  actually observe

## Alternatives Considered

### Option A: Explicit stage-0 swap seam plus noop runtime implementation

This won because it isolates inherited AMM complexity while keeping the live
runtime contract explicit and replaceable.

### Option B: Keep the full inherited swap contract everywhere

This was rejected because it would keep stage-0 code and documentation looking
more liquidity-aware than the real runtime actually is.

### Option C: Delete the swap seam entirely and hardcode identity math inline

This was rejected because too many carried pallet paths still expect a swap-like
boundary, and removing it outright would have mixed cleanup with semantic
behavior changes.

## Consequences

### Positive

- Stage-0 runtime behavior is easy to state and test.
- Future pallet cleanup can target one seam instead of dozens of callsites.
- A real pricing mechanism can replace the noop runtime behind the same
  contract if stage-2 economics ever require it.

### Negative

- The repo still carries swap-shaped names even though the stage-0 runtime does
  not run an AMM.
- Test surfaces that still use broader mock runtimes can mislead contributors if
  they are treated as stage-0 truth without checking the real runtime.

### Follow-up

- Keep proving behavior at the runtime/API surface, not only in the local noop
  implementation.
- Continue removing dead liquidity-era reporting and helper baggage when a
  narrow slice can prove the reduction safely.

## Reversibility

Easy to moderate.

The seam itself is intentionally a bridge. Replacing the noop implementation
with a real pricing engine is straightforward compared with deleting the seam
entirely. Reversal is only hard if callers start depending on the current
identity behavior without guarding for a future change.

## Validation / Evidence

- `crates/myosu-chain/pallets/game-solver/src/lib.rs` defines the pallet-side
  stage-0 swap interface.
- `crates/myosu-chain/runtime/src/lib.rs` wires `type SwapInterface =
  Stage0NoopSwap`.
- `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` and the runtime's
  stage-0 swap tests prove the identity/zero-fee contract directly.
