# ADR 001: Single-Token Emission And Staking Model

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `ops/decision_log.md`, `AGENTS.md`, `specs/040226-02-single-token-emission-accounting.md`
- Informed: chain, miner, validator, and operator contributors
- Related: `ops/decision_log.md`, `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs`, `crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs`, `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`

## Context

This is a retroactive record of the single-token decision captured in the
2026-03-17 decision log and now reflected throughout the live stage-0 runtime.

Subtensor's inherited model assumes subnet-specific alpha assets, AMM-backed
price discovery, root-network stake weighting, and swap-era liquidity
housekeeping. Myosu's stage-0 loop does not need any of that to prove its core
product claim: one chain can register a poker subnet, let miners serve
strategies, let validators score them, and distribute emissions deterministically.

Keeping the dual-token model would preserve a large amount of storage, pricing,
and coinbase complexity that does not create stage-0 value. At the same time,
the fork still inherits many `TaoCurrency` and `AlphaCurrency` type names, so
the chosen model needed to simplify economics without forcing a full rewrite of
every inherited code path at once.

## Decision

Myosu uses a single token for staking and emissions in stage 0.

Concretely:

- the runtime uses a noop swap surface with identity conversion, unit price, and
  zero fees
- stage-0 coinbase logic treats root-network weighting as inactive and keeps
  rewards subnet-local
- inherited alpha-vs-TAO naming is tolerated only as a compatibility layer
  while the live economics remain single-token
- stage-0 operator and proof surfaces must describe the system as single-token,
  not as a dormant dual-token network

## Alternatives Considered

### Option A: Single-token stage-0 model

This won because it removes unused AMM complexity while preserving a truthful,
workable emission and staking path for the stage-0 bootstrap loop.

### Option B: Keep the inherited dual Alpha/TAO model

This was rejected because it carries swap pools, pricing state, and root
weighting that Myosu does not need to prove solver incentives in stage 0.

### Option C: Rewrite every inherited currency type and storage shape immediately

This was rejected because it would have mixed a real economic simplification
with a high-risk fork-wide rename and migration program.

## Consequences

### Positive

- The live runtime economics are far simpler to reason about and validate.
- Emission proofs no longer depend on root-network stake weighting.
- Swap calls remain callable only as a compatibility seam, not as live market
  behavior.

### Negative

- The codebase still carries some inherited alpha/TAO terminology, which can
  mislead readers unless the single-token contract is stated explicitly.
- Reintroducing dual-token behavior later would require a real migration, not a
  flag flip.

### Follow-up

- Keep stage-0 proofs focused on single-token behavior, especially emission
  accounting and swap-surface outputs.
- Continue shrinking inherited dual-token naming and payload baggage when a
  slice can do so truthfully.

## Reversibility

Moderate to hard to reverse.

The compatibility type names make a future dual-token design technically
possible, but restoring real alpha/TAO economics would require storage,
coinbase, staking, runtime API, and operator migration work. The decision
should only be reopened if a later-stage spec demonstrates concrete product
value that the single-token model cannot support.

## Validation / Evidence

- `crates/myosu-chain/runtime/src/lib.rs` implements `Stage0NoopSwap` as
  identity conversion with zero fees.
- `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` documents and tests
  the stage-0 single-token swap contract.
- `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs` proves
  root proportion and pending root dividend behavior stay zeroed or folded back
  into subnet-local accounting.
