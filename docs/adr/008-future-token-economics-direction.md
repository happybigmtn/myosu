# ADR 008: Future Token Economics Direction

- Status: Proposed
- Date: 2026-04-05
- Deciders: pending maintainer review required by `specs/050426-token-economics.md`
- Consulted: `specs/050426-token-economics.md`, `docs/adr/001-single-token-emission.md`, `docs/adr/005-swap-interface-abstraction.md`, `docs/adr/stage-2-roadmap.md`, `ops/decision_log.md`, `genesis/plans/013-token-economics-research.md`
- Informed: chain, runtime, pallet, and operator contributors
- Related: `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/pallets/swap-interface/src/lib.rs`, `crates/myosu-chain/pallets/swap/`, `crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs`, `crates/myosu-chain/pallets/game-solver/src/staking/stake_utils.rs`, `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`

## Context

Myosu's live runtime is deliberately simpler than the inherited subtensor
economics:

- `crates/myosu-chain/runtime/src/lib.rs` wires `Stage0NoopSwap`, which returns
  identity conversions, zero fees, and `C::MAX` price limits.
- `crates/myosu-chain/pallets/swap-interface/src/lib.rs` still defines a richer
  swap contract with quote, fee, and liquidity-shaped concepts.
- `crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs` routes
  `burned_register` through the swap seam before burning the returned alpha.
- `crates/myosu-chain/pallets/game-solver/src/staking/stake_utils.rs` still
  carries `fee_paid`, `fee_to_block_author`, and price-dependent stake math.
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` still
  computes subnet emission terms through `stage0_current_alpha_price()` and
  `inject_and_maybe_swap()`, even though the live price is pinned to `1`.
- `crates/myosu-chain/pallets/swap/` contains the inherited AMM implementation,
  but the stage-0 runtime does not wire it in.

ADR 001 already records that the active runtime is single-token. ADR 005 records
that the swap seam is a compatibility boundary. Neither document answers the
future-facing question in `specs/050426-token-economics.md`: what should replace
the noop economics, if anything, after stage-0 proves the product loop.

The repo also already contains a strong architectural prior in
`docs/adr/stage-2-roadmap.md`: dual-token economics are the hardest stage-2
decision to reverse and should be last, not first. This ADR turns that roadmap
guidance into a concrete recommendation on the open design axes so future work
does not reactivate the dormant AMM by inertia.

## Decision

Myosu should keep the single-token MYOSU model as the active economics through
stage-1 and any early stage-2 work until concrete multi-subnet or operator
demand proves that single-token incentives are insufficient.

That recommendation includes these boundaries:

- Do not activate the inherited `crates/myosu-chain/pallets/swap/` AMM as the
  default next step after `Stage0NoopSwap`.
- Treat `Stage0NoopSwap` as a temporary compatibility bridge, not as a final
  public-network pricing mechanism.
- If Myosu later needs subnet-local economic separation, begin with a narrow
  protocol-controlled conversion surface rather than a continuous user-liquidity
  AMM.
- Keep the current zero-fee swap behavior while no real conversion exists.
- Keep the current stage-0 registration burn for bootstrap, but plan to replace
  it with bond-style admission controls before open admission becomes a product
  requirement.
- Keep the current stage-0 emission constants for now, but do not treat the
  inherited subtensor schedule as long-term doctrine.

## Design Axes

### Single Token Vs Dual Token

Recommendation: stay single-token until the network can point to a specific
single-token failure that routing, governance, or admission controls cannot
solve.

Rationale:

- Myosu's quality signal is deterministic exploitability, not subjective market
  sentiment. That removes the strongest argument for subnet-local alpha assets
  as a proxy for quality discovery.
- The live product loop is still proving one network can coordinate miners,
  validators, and gameplay around a single solver market.
- Reintroducing a second token would reopen storage, staking, runtime API, and
  operator complexity before the network has shown it needs subnet-local price
  discovery.

Revisit trigger:

- more than one subnet has durable operator demand and the network can show that
  a shared single-token budget is preventing credible subnet-local ownership,
  routing, or funding.

### Conversion Mechanism

Recommendation: keep the runtime on `Stage0NoopSwap` until a real conversion
need exists. If that need arrives, prefer a deterministic protocol-controlled
conversion model first, such as epoch-priced or governance-set quotes, rather
than a continuous LP AMM.

Rationale:

- The inherited AMM brings user liquidity, slippage, ordering, and fee-routing
  semantics that the current game-solving network does not need.
- A protocol-controlled conversion path preserves determinism and gives the
  chain a smaller attack surface than a live trading venue.
- The existing `SwapEngine<O: Order>` seam is useful as a migration bridge, but
  it should not force Myosu to inherit the full subtensor market model.

### Fee Model

Recommendation: keep conversion fees at zero while `Stage0NoopSwap` remains
live. If a non-identity conversion path is introduced, route fees to an
explicit protocol or subnet budget, not to block authors by default.

Rationale:

- The current `fee_to_block_author` path exists because the carried swap result
  type includes it, not because block production is the correct economic sink
  for solver-market conversion fees.
- A protocol-controlled budget aligns better with subnet funding, artifact
  maintenance, and future operator support than incidental block-author capture.
- If future policy wants a burn component, it should be explicit and reviewed as
  part of the economics design, not inherited accidentally from AMM plumbing.

### Registration Cost Model

Recommendation: keep the flat `burned_register` model during bootstrap, but the
target post-bootstrap admission control should be an occupancy-sensitive bond
with a cooldown release. Use dynamic burn only as an interim fallback if bond
accounting proves too invasive for the next stage.

Rationale:

- Registration cost is primarily an anti-spam and anti-churn control, not a
  market for price discovery.
- Deterministic quality scoring means low-quality participants can be filtered
  by scoring and pruning without requiring permanent capital destruction as the
  main defense.
- A bond keeps the cost of experimentation and honest entry lower than a pure
  burn while still making rapid churn expensive.

### Emission Schedule

Recommendation: keep the current stage-0 emission constants while the network
is still proving operator and multi-node truth, but do not carry the inherited
subtensor schedule forward as a product principle. When Myosu reopens emission
policy, prefer a flat or stepwise governance-tuned schedule over an AMM- or
scarcity-coupled curve.

Rationale:

- Game-solving incentives should track solver quality, operator cost, and
  network growth, not the scarcity story of a general compute token.
- The current schedule is acceptable as bootstrap scaffolding because the stage-0
  objective is proof of a deterministic incentive loop, not token-market design.
- Reopening emission policy should happen only after governance and operator
  release processes are strong enough to change economics safely.

### Swap Trait And Runtime Surface

Recommendation: keep the current `SwapEngine` and `SwapHandler` traits only as
the compatibility seam that lets stage-0 compile. If Myosu ships a non-identity
conversion model later, narrow the surface so quote semantics, execution
semantics, and liquidity administration are not coupled by inheritance.

Rationale:

- `crates/myosu-chain/pallets/swap-interface/src/lib.rs` still exposes
  `drop_fees`, `should_rollback`, `adjust_protocol_liquidity`, and
  `toggle_user_liquidity`, which are AMM-era concepts rather than clearly
  justified requirements for a solver network.
- A smaller conversion surface will be easier to reason about, test, and expose
  to operators than the inherited interface.

## Implementation Surface

| Option | Reuse | Replace |
|---|---|---|
| Recommended: single-token now, protocol-controlled conversion later if needed | Reuse the runtime `type SwapInterface` slot, the existing internal single-token operator model, and the carried `SwapResult`/order plumbing as a migration bridge. | Replace `Stage0NoopSwap` in `crates/myosu-chain/runtime/src/lib.rs`; replace the `price == 1`, `fee == 0`, and `max_price == C::MAX` assumptions in `subnets/registration.rs`, `staking/stake_utils.rs`, and `coinbase/run_coinbase.rs`; narrow the swap trait before exposing new economics publicly. |
| Rejected: activate inherited dual-token AMM | Reuse some currency newtypes and parts of the existing swap pallet implementation. | Replace stage-0 operator docs, emission proofs, staking expectations, CI gates, and runtime/operator mental model; add slippage, fee, liquidity, and migration behavior everywhere the current repo assumes identity pricing. |
| Rejected: keep the noop model as the final economics forever | Reuse almost all current stage-0 code and docs. | Replace no code immediately, but accept that subnet-local market signals, funding separation, and future admission models must be solved some other way. |

## Alternatives Considered

### Option A: Keep single-token economics and delay real conversion until the network proves it needs one

This wins because it matches the current repo truth, preserves deterministic
proof surfaces, and keeps the hardest-to-reverse economic choice behind actual
evidence instead of inheritance pressure.

### Option B: Reactivate the inherited subtensor dual-token AMM

This is rejected because it imports liquidity, fee, ordering, and migration
complexity that the current product loop has not justified. It also reopens too
many attack surfaces at once.

### Option C: Declare the noop model permanent and never design beyond it

This is rejected because it would turn today's bootstrap simplification into a
doctrine before Myosu has measured whether multi-subnet or third-party subnet
ownership need additional economic tools.

## Consequences

### Positive

- The repo now has a concrete recommendation instead of an open-ended research
  gap.
- Future token-economics work has a default answer: do not wire in the dormant
  AMM unless the explicit revisit trigger is met.
- Registration, emission, and swap follow-on work can be judged against a
  stated product principle rather than inherited subtensor assumptions.

### Negative

- This ADR does not fully design the future bond model, treasury model, or
  quote mechanism; those stay as follow-on decisions.
- The codebase will continue to carry Alpha/TAO-shaped compatibility names for
  longer than the ideal end state.
- If Myosu reaches real multi-subnet demand quickly, a second ADR will still be
  needed before implementation can begin.

### Follow-up

- Record maintainer review on this ADR before any work replaces `Stage0NoopSwap`
  in the runtime.
- If the revisit trigger fires, author a new ADR for the chosen conversion and
  admission model before touching runtime economics.
- Continue keeping operator docs explicit that the live network is single-token
  and does not run a public AMM.

## Reversibility

Moderate today, hard later.

Keeping the single-token model is easy to revisit while the network is still in
bootstrap proof mode. Introducing a second token or a live conversion venue
would be hard to undo once storage, operator expectations, and emission proofs
depend on it. This ADR should be reopened only when the network can show a
concrete single-token failure mode in live operator or multi-subnet use.

## Transition Risks And Required Tests

### 1. Slippage And Price-Limit Semantics

- Current risk surface: `crates/myosu-chain/runtime/src/lib.rs` returns `C::MAX`
  for `default_price_limit()` and `max_price()`.
- Required tests: quote-limit rejection tests for registration and staking, plus
  runtime tests showing successful execution never exceeds the quoted limit.

### 2. Fee Activation Paths

- Current risk surface: `crates/myosu-chain/pallets/game-solver/src/staking/stake_utils.rs`
  routes `fee_paid` and `fee_to_block_author` through both stake-add and
  stake-remove flows.
- Required tests: author-present and author-absent cases must preserve total
  issuance, user balances, and fee sink behavior under non-zero fees.

### 3. Price-Dependent Emission Math

- Current risk surface: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`
  uses `get_subnet_terms()` and `inject_and_maybe_swap()` even though stage-0
  price is pinned to `1`.
- Required tests: epoch tests where price is not `1`, `excess_tao` is non-zero,
  and three nodes still agree on emission storage bit-for-bit.

### 4. Registration Cost Conversion

- Current risk surface: `crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs`
  burns registration cost through `swap_tao_for_alpha(..., stage0_max_price())`.
- Required tests: registration must respect any future quote limit and account
  for unused input or refund behavior explicitly.

### 5. Cross-Node Determinism

- Current risk surface: all future price and fee math still needs to remain
  deterministic across validators and nodes.
- Required tests: validator agreement tests and multi-node storage agreement
  tests must be extended before any non-identity conversion ships.

## Migration Path

1. Keep `Stage0NoopSwap` as the live runtime behavior until this ADR is reviewed
   and a follow-on economics ADR is accepted.
2. When the revisit trigger is met, write a narrow spec for the chosen
   conversion and admission model before changing code.
3. Introduce quote and execution proofs in unit tests and multi-node tests
   before replacing the runtime swap implementation.
4. Ship the new economics on devnet or testnet first, with operator-facing
   migration notes and rollback steps.
5. Replace the runtime noop implementation only after the new model has
   deterministic proofs for registration, staking, emission, and operator flows.

## Validation / Evidence

- Repo evidence consulted: the token-economics spec, ADR 001, ADR 005, the
  stage-2 roadmap, the decision log, and the live runtime, registration,
  staking, and coinbase code paths listed above.
- This ADR is the repo-local decision document requested by `F-003`.
- The task is not fully complete until the review requirement from
  `specs/050426-token-economics.md` is met. No second-contributor review is
  recorded in-repo yet, so the ADR remains `Proposed`.
