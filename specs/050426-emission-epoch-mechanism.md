# Specification: Emission and Epoch Mechanism

## Objective

Define the per-block coinbase emission pipeline and per-epoch Yuma Consensus distribution as implemented in the game-solver pallet. This spec covers the end-to-end flow from block emission through subnet token injection, epoch drain, consensus scoring, and final stake distribution to hotkeys.

## Evidence Status

| Claim | Source | Status |
|---|---|---|
| `run_coinbase` called per-block with `U96F32` emission | `coinbase/run_coinbase.rs:44` | Verified |
| Root network (netuid 0) excluded from emission | `run_coinbase.rs:51-54` filter on `NetUid::ROOT` | Verified |
| Epoch fires when `(block + netuid + 1) % (tempo + 1) == 0` | `run_coinbase.rs:1033-1051` | Verified |
| Tempo 0 disables epoch permanently (`blocks_until == u64::MAX`) | `run_coinbase.rs:1044-1046` | Verified |
| Swap stub is 1:1 identity, zero fees | `swap_stub.rs:96-98` `NoOpSwap` impl | Verified |
| `max_price` returns `B::max_value()` (slippage protection disabled) | `swap_stub.rs:101-105` | Verified |
| Sparse epoch (`epoch_mechanism`) is production path | `run_epoch.rs:570` | Verified |
| Dense epoch (`epoch_dense_mechanism`) retained for test parity | `run_epoch.rs:85-96`, `run_epoch.rs:155-157` comment | Verified |
| Yuma3 and traditional bond paths both present | `run_epoch.rs:801` `Yuma3On` branch | Verified |
| Owner-key miner emission is recycled or burned, not staked | `run_coinbase.rs:633-649` | Verified |
| Consistency check: duplicate hotkey detection before epoch | `run_epoch.rs:1599-1610` | Verified |
| `CoinbaseSummary` tracks server/validator/root/owner alpha | `run_coinbase.rs:19-28` | Verified |
| Miner/validator split is hardcoded 50/50 | `run_coinbase.rs:256-259` `asfloat!(0.5)` | Verified |
| Root proportion diverts validator alpha to root stakers | `run_coinbase.rs:245-252` | Verified |
| Weighted median uses deterministic mid-pivot | `math.rs` (per context) | Verified from context |
| `epoch_with_mechanisms` splits emission across MechIds | `subnets/mechanism.rs:292-312` | Verified |
| `calculate_dividend_distribution` has two cfg variants: `full-runtime` and not | `run_coinbase.rs:405-461, 463-565` | Verified |
| Zero-dividend fallback distributes by stake weight | `run_coinbase.rs:421-448` non-full-runtime path | Verified |

## Acceptance Criteria

### Coinbase pipeline (per-block)

- `run_coinbase(block_emission)` processes all non-root subnets each block
- `get_subnets_to_emit_to` filters subnets eligible for emission (registration, liveness)
- `get_subnet_block_emissions` distributes `block_emission` across subnets weighted by their TAO proportion
- `get_subnet_terms` computes per-subnet `tao_in`, `alpha_in`, `alpha_out`, `excess_tao` using the swap interface price (identity under `NoOpSwap`)
- `alpha_in` is capped at `min(alpha_emission, tao_block_emission)` to prevent unbounded alpha injection
- `inject_and_maybe_swap` injects TAO and Alpha into subnet state, swaps excess TAO for Alpha (recycled), and updates `TotalIssuance` and `TotalStake`
- Owner cut is deducted from `alpha_out` at `get_float_subnet_owner_cut()` rate and accumulated in `PendingOwnerCut`
- Server emission = 50% of remaining `alpha_out`, accumulated in `PendingServerEmission`
- Validator emission = remaining 50% minus root alpha, accumulated in `PendingValidatorEmission`
- Root alpha = `root_proportion * alpha_out * 0.5`, gated by `root_sell_flag` (total EMA price > 1); if flag is false, root alpha is recycled

### Epoch drain (at tempo boundary)

- Epoch fires for subnet `netuid` when `(current_block + netuid + 1) % (tempo + 1) == 0`
- `drain_pending` iterates all subnets, increments `BlocksSinceLastStep`, and for those at tempo boundary: drains `PendingServerEmission`, `PendingValidatorEmission`, `PendingRootAlphaDivs`, `PendingOwnerCut` to zero
- Epoch is skipped (no drain, no distribution) if `is_epoch_input_state_consistent` returns false (duplicate hotkeys in `Keys` map)
- `distribute_emission` calls `epoch_with_mechanisms` which splits emission across registered `MechId`s, runs `epoch_mechanism` for each, persists per-mechanism terms, then aggregates weighted by emission share

### Yuma Consensus (epoch_mechanism, sparse production path)

- Inputs: neuron hotkeys from `Keys`, last_update timestamps, block_at_registration, activity_cutoff, stake weights, validator permits, weight matrix
- Activity filter: neuron inactive if `last_update + activity_cutoff < current_block`
- Stake filter: neurons below `get_stake_threshold()` have stake zeroed
- Validator permits: top-k by stake where `k = max_allowed_validators`; non-permitted validators have weights masked
- Weight processing: mask non-validators, remove self-weights (except subnet owner), remove outdated weights (updated <= registered), apply commit-reveal mask if enabled, row-normalize
- Consensus: stake-weighted median per column with kappa threshold
- Clipped weights: element-wise min of weight and consensus
- Validator trust: row-sum of clipped weights
- Ranks/incentive: `matmul_sparse(clipped_weights, active_stake)`, normalized
- Bonds: Yuma3 path uses `compute_bonds` with fixed-proportion EMA; traditional path uses `row_hadamard(weights, stake)` with standard EMA. Both column-normalized.
- Dividends (Yuma3): `vec_mul(row_sum(mat_vec_mul(ema_bonds_norm, incentive)), active_stake)`, normalized
- Dividends (traditional): `matmul_transpose(ema_bonds, incentive)`, normalized
- Emission scoring: if `emission_sum == 0`, validators get emission proportional to (active) stake; otherwise server_emission = incentive share, validator_emission = dividend share
- Output: `EpochOutput<T>` maps `AccountId -> EpochTerms` with uid, dividend, incentive, server_emission, validator_emission, stake_weight, active, emission, consensus, validator_trust, new_validator_permit, bond, stake

### Distribution (post-epoch)

- `distribute_emission` recomputes dividend allocation from epoch output using `calculate_dividend_and_incentive_distribution`
- If incentive sum is zero, validators receive both server and validator alpha pools
- `calculate_dividend_distribution` (non-full-runtime): proportional to epoch dividends, or falls back to weighted stake if total dividends are zero
- `calculate_dividend_distribution` (full-runtime): splits dividends into alpha and root components proportional to each hotkey's local vs root stake (weighted by `tao_weight`)
- Owner cut: staked to subnet owner hotkey+coldkey
- Miner incentives: staked to miner hotkey owner, except owner-associated hotkeys whose emission is recycled or burned per `RecycleOrBurn` config
- Validator alpha dividends: validator take deducted first, remainder distributed to nominators via `increase_stake_for_hotkey_on_subnet`
- Root alpha dividends: validator take deducted, remainder goes to `increase_root_claimable_for_hotkey_and_subnet`

### Invariants

- Total emission distributed per epoch must equal `sum(PendingServer + PendingValidator + PendingRoot + PendingOwnerCut)` that was drained -- no creation or destruction of tokens during distribution
- Under `NoOpSwap`: `amount_in == amount_out`, `fee == 0` for all swap paths; `current_alpha_price == 1`
- `blocks_until_next_epoch` with `tempo == 0` returns `u64::MAX` (epoch never runs)
- Sparse and dense epoch implementations must produce identical results for the same inputs (test-only verification)

## Verification

| Check | Method | Location |
|---|---|---|
| Coinbase summary fields populated | Unit test | `tests/stage_0_flow.rs` |
| Identity swap correctness | Unit test (8 cases) | `swap_stub.rs:123-188` |
| Yuma consensus scoring | Unit tests | `tests/epoch.rs` |
| Mechanism-level emission split | Unit tests | `tests/mechanism.rs` |
| E2E emission flow | Shell script | `emission_flow.sh` (thin assertions) |
| Emission conservation invariant | Partial -- `sum(distributions) == block_emission * epochs` asserted in stage_0_flow but not for all edge cases |
| Dense/sparse parity | Not currently tested in CI as a property |
| Zero-stake subnet epoch | Not tested -- `drain_pending` will fire but epoch output is undefined when all stakes are zero |
| Duplicate hotkey guard | `is_epoch_input_state_consistent` check before epoch; no test that verifies the skip path |

## Open Questions

1. **Emission conservation proof gap**: The invariant `sum(all distributed alpha) == total drained pending` is asserted in `stage_0_flow.rs` but only for the happy path. Fixed-point truncation in `U96F32 -> u64` conversions (via `tou64!`) loses fractional rao each block. Over many epochs this could accumulate. Is the truncation loss acceptable, or should it be tracked as dust?

2. **Dead coinbase paths under NoOpSwap**: `inject_and_maybe_swap` calls `swap_tao_for_alpha` and `recycle_subnet_alpha` on excess TAO. Under the identity stub, `excess_tao` is typically zero (price = 1, so `alpha_in == tao_emission / 1 == tao_emission`). These paths are unreachable in stage-0 but carry ~50 lines of logic. Should they be gated behind a feature flag per Plan 003?

3. **Dense epoch retention**: `epoch_dense_mechanism` (lines 157-555 of `run_epoch.rs`) duplicates the full consensus algorithm with dense matrices for test use. No CI job verifies dense/sparse parity. Should dense be removed, or should a property test enforce parity?

4. **50/50 miner/validator split hardcoded**: The server/validator emission ratio is `asfloat!(0.5)` at `run_coinbase.rs:256`. This is not configurable per-subnet. Is this intentional for stage-0, and when should it become a parameter?

5. **Zero-dividend fallback behavior**: When `total_dividends == 0` in the non-full-runtime path, emission is distributed by weighted stake. This means a subnet with no weight-setting activity still distributes validator emission. Is this the desired behavior, or should emission be recycled?

6. **Root sell flag threshold**: `get_network_root_sell_flag` sums EMA prices across all emitting subnets and compares to 1. With a single subnet at identity price (1), this returns false, meaning root alpha is always recycled in stage-0. This is correct but undocumented.

7. **Commit-reveal column mask**: The sparse epoch applies an additional weight mask when `commit_reveal_weights_enabled` is true. This filters weights where the commit block predates the target neuron's registration. Is this feature active in any stage-0 subnet configuration?

8. **`is_epoch_input_state_consistent` scope**: The check only detects duplicate hotkeys. It does not verify that `SubnetworkN` matches the actual key count, or that UIDs are contiguous. Are there other consistency invariants that should gate epoch execution?
