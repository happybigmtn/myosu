# Specification: Token Economics Model

Source: Genesis Plan 013 (Token Economics Research Gate), ASSESSMENT.md findings
Status: Draft
Depends-on: none (research gate; runs in parallel with implementation work)

## Objective

Document the stage-0 token model as implemented, identify the design questions
that must be answered before moving beyond stage-0, and define the research gate
that produces a decision document on future token economics. The stage-0 model
is simple, proven, and intentional. Future economics are an open design space
that this spec frames without prescribing.

## Evidence Status

| Claim | Status | Source |
|---|---|---|
| Single token (MYOSU) with TAO balance type | Verified | Cargo.toml, runtime config |
| All 37 swap callsites satisfied by `NoOpSwap<B>` | Verified | `swap_stub.rs` (189 lines) |
| `max_price` returns `u64::MAX` | Verified | `swap_stub.rs` |
| Registration burns tokens (`burned_register`) | Verified | pallet-game-solver coinbase |
| Staking uses share-pool model with direct locking | Verified | pallet-subtensor staking logic |
| `AlphaCurrency` / `TaoCurrency` types exist in code | Verified | coinbase and epoch modules |
| AMM path inherited but dormant behind no-op stub | Verified | `swap_stub.rs`, swap-interface |
| No slippage protection in stub | Verified | Intentional for stage-0 |
| `SwapEngine<O: Order>` trait defines swap interface | Verified | `swap-interface/src/lib.rs` (102 lines) |

## Stage-0 Model (Current, Implemented)

### Single-token identity

Stage-0 operates on a single token. The inherited dual-token architecture
(TAO + Alpha) is present in type signatures but both currency types resolve to
the same underlying balance. The `NoOpSwap<B>` stub in
`crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` implements all swap
traits as identity operations (1:1 conversion, zero fees), satisfying the 37
callsites that reference swap logic.

### Token flows

Three token flows exist in stage-0:

1. **Emission**: The coinbase distributes new tokens to neurons proportional to
   Yuma Consensus dividend weights. This is the primary incentive mechanism.

2. **Registration burn**: Miners burn tokens to register on a subnet via
   `burned_register`. This creates a cost barrier to sybil registration.

3. **Staking**: Token holders lock tokens to delegate stake to validators. The
   share-pool model tracks proportional ownership. Staked tokens participate in
   consensus weight but are not liquid.

### Design properties of the stub

The `NoOpSwap<B>` stub has specific properties that are correct for stage-0 but
must not carry forward:

- `max_price` returns `u64::MAX` — safe when all swaps are identity, but would
  allow unbounded slippage with a real AMM.
- No fee accounting — `fee_paid` and `fee_to_block_author` are zero.
- No price discovery — every swap is 1:1 regardless of supply or demand.
- No slippage protection — intentionally omitted since identity swaps cannot
  slip.

### Fixed-point arithmetic

All economic calculations use `U96F32` fixed-point types from
`substrate_fixed` v0.6.0 (encointer fork), with overflow protection via the
`safe_math` crate. This is load-bearing for determinism across validators and
must be preserved.

## Future Economics (Open Design Space)

### The core question

The inherited Bittensor model uses dual tokens (TAO as root currency, Alpha as
subnet-specific currency) with an AMM for conversion. This model was designed
for a general-purpose compute network. Myosu is a game-solving network where
quality measurement is deterministic and verifiable. The fundamental question is
whether this economic model fits, needs adaptation, or should be replaced.

### Design axes to evaluate

**Single vs. dual token**: Does a subnet-specific token serve any purpose in a
game-solving network? Potential arguments for: economic isolation between game
types, subnet-level price discovery for solver quality. Potential arguments
against: complexity without clear benefit, liquidity fragmentation, the identity
model already works.

**AMM vs. alternative conversion**: If dual tokens are adopted, the inherited
AMM (constant-product style) provides one conversion mechanism. Alternatives
include fixed-rate conversion, oracle-based pricing, or governance-set rates.
The choice depends on whether organic price discovery between game subnets has
value.

**Fee model**: Stage-0 has zero swap fees. A future model needs to determine
whether swap fees exist, who receives them (block authors, treasury, burn), and
how they interact with emission incentives.

**Registration cost model**: The current `burned_register` is a flat burn. Future
options include dynamic pricing (cost scales with subnet demand), staking-based
registration (locked rather than burned), or auction-based slot allocation.

**Emission schedule**: The current emission rate is inherited from Bittensor
parameters. Whether this schedule is appropriate for a game-solving network with
different growth dynamics is an open question.

### What the stub masks

The identity swap hides potential economic bugs that would surface with a real
AMM. Specifically:

- Arithmetic overflow in price calculation paths (currently bypassed)
- Slippage-related fund loss (currently impossible)
- Fee distribution logic (currently zeroed)
- Price manipulation via swap ordering (currently irrelevant)

These must be tested when any non-identity swap is introduced.

## Scope

In scope:
- Documenting the stage-0 token model as a baseline
- Defining the research questions for future economics
- Specifying the output format of the research gate (decision document)
- Identifying risks masked by the identity swap

Out of scope:
- Implementing any changes to the stage-0 model
- Choosing a specific future economic model
- AMM implementation or swap logic changes
- Emission schedule modifications
- Token launch or distribution planning

## Acceptance Criteria

- A decision document is produced that evaluates single vs. dual token for a
  game-solving network, with concrete arguments for and against.
- The decision document addresses AMM appropriateness, fee model, registration
  cost model, and emission schedule with rationale.
- The document identifies which inherited code paths are reusable under each
  option and which must be replaced.
- The document includes a risk assessment of moving from identity swap to any
  real swap mechanism, referencing the specific masked behaviors listed above.
- The research gate can proceed in parallel with all stage-0 implementation
  work; no blocking dependency exists.

## Verification

The research gate output is a written decision document, not code. Verification
is review-based:

- The decision document is reviewed by at least two contributors with context
  on both the Bittensor economic model and the Myosu game-solving domain.
- Each design axis (single/dual token, AMM model, fees, registration, emission)
  has a stated recommendation with rationale grounded in the game-solving use
  case, not inherited assumptions.
- The risk assessment references specific code locations (swap_stub.rs,
  swap-interface, coinbase) and identifies concrete test scenarios for the
  transition from identity swap to real economics.

## Open Questions

1. Is there a game-theoretic argument for subnet-specific tokens in a network
   where quality measurement is deterministic? (In Bittensor, Alpha tokens
   partially address the problem of subjective quality — does deterministic
   quality measurement eliminate this need?)

2. Should the registration burn rate be dynamic? If solver quality is
   measurable, registration cost could track subnet demand or historical
   quality metrics.

3. What emission schedule matches the growth dynamics of a game-solving network?
   The Bittensor halving schedule assumes compute-network growth patterns that
   may not apply.

4. Does the `SwapEngine<O: Order>` trait interface support the design options
   under consideration, or will the trait itself need revision?

5. When the research gate concludes, what is the migration path from identity
   swap to the chosen model? Is a phased rollout (e.g., real AMM on testnet
   before mainnet) required?
