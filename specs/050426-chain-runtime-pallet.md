# Specification: Chain Runtime and Pallet

## Objective

Define the contract for the Myosu chain runtime and its core `pallet-game-solver` pallet at stage-0 maturity. The runtime is a Substrate node (`myosu-chain`) using Aura/GRANDPA consensus. The pallet is a reduced fork of Bittensor's `pallet-subtensor` that provides subnet registration, neuron registration, staking, weight submission, axon serving, epoch processing (Yuma Consensus), and per-block coinbase emission.

This spec establishes what the stage-0 surface looks like today, what is verified, and what acceptance criteria must hold as the codebase evolves.

## Evidence Status

### Verified facts (from code)

- Runtime entry point: `crates/myosu-chain/node/src/main.rs` (17 lines, delegates to `command::run()`)
- Runtime definition: `crates/myosu-chain/runtime/src/lib.rs`
- Pallet source: `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- Swap stub: `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` (188 lines)
- Consensus: Aura (index 3) + GRANDPA (index 4) in `construct_runtime!`
- `pallet-game-solver` is registered as `SubtensorModule` at runtime index 7 (aliased from `pallet_subtensor`)
- Two `construct_runtime!` blocks exist: `full-runtime` (15 pallets) and default stage-0 (7 pallets: System=0, Timestamp=2, Aura=3, Grandpa=4, Balances=5, TransactionPayment=6, SubtensorModule=7, Utility=11, AdminUtils=19)
- Stage-0 default build exposes 25 extrinsics (not behind `full-runtime` gate): `set_weights`, `commit_weights`, `reveal_weights`, `add_stake`, `serve_axon`, `burned_register`, `set_childkey_take`, `register_network`, `faucet`, `swap_stake`, `swap_stake_limit`, `try_associate_hotkey`, `start_call`, `associate_evm_key`, `recycle_alpha`, `burn_alpha`, `update_symbol`, `claim_root`, `set_root_claim_type`, `sudo_set_root_claim_threshold`, `announce_coldkey_swap`, `swap_coldkey_announced`, `dispute_coldkey_swap`, `reset_coldkey_swap`, `add_stake_burn`
- Full-runtime adds 38 additional extrinsics (63 total)
- 193 `#[pallet::storage]` items exist in `pallet-game-solver`
- `NoOpSwap<B>` implements identity 1:1 TAO-to-Alpha conversion with zero fees across all 37 swap callsites
- `max_price` returns `B::max_value()` (u64::MAX for u64), disabling slippage protection — explicitly documented as stage-0 only
- `default_price_limit()` also returns `B::max_value()`
- `is_user_liquidity_enabled` always returns `false`
- Swap stub has 9 unit tests covering identity swap, zero fees, price, liquidity ops, and engine
- Clippy lints enforced at workspace level: `arithmetic_side_effects = "deny"`, `expect_used = "deny"`, `indexing_slicing = "deny"`, `unwrap_used = "deny"`
- Runtime `lib.rs` blanket-allows `clippy::arithmetic_side_effects` with documented justification (QUAL-001) for macro-generated code
- `spec_version: 385`, `impl_version: 1`
- Edition 2024, package name `pallet-game-solver`
- Workspace uses opentensor polkadot-sdk fork at rev `71629fd`
- `substrate-fixed` from encointer fork at tag `v0.6.0`
- Legacy pallets still exist as source directories: admin-utils, crowdloan, drand, proxy, registry, subtensor, swap, swap-interface, transaction-fee, utility
- `pallet-subtensor` (the original Bittensor copy) still exists at `crates/myosu-chain/pallets/subtensor/` as a parallel directory

### Recommendations (from planning corpus)

- Plan 002: Remove duplicate `pallet-subtensor` directory (~150K lines of dead code)
- Plan 005: Audit and reduce storage items from ~193 to ~80 for stage-0
- Plan 014: Research migration from opentensor polkadot-sdk fork to upstream

### Hypotheses (unverified)

- Yuma Consensus epoch logic may be simplifiable for single-subnet stage-0 (no root subnet, no cross-subnet weighting)
- The 25 stage-0 extrinsics may be reducible further — several (coldkey swap announce/dispute/reset, EVM key association, root claims) may not be needed for initial launch
- Migration to upstream polkadot-sdk may be feasible if the opentensor fork's changes are limited to pallet-subtensor-specific APIs

## Acceptance Criteria

- The default build (no `full-runtime` feature) compiles and produces a working WASM runtime with exactly the pallets listed in the stage-0 `construct_runtime!` block (indices 0, 2, 3, 4, 5, 6, 7, 11, 19)
- `pallet-game-solver` at index 7 exposes no more than 25 extrinsics in the default build
- All 25 stage-0 extrinsics are callable from a devnet node without enabling `full-runtime`
- `NoOpSwap<B>` satisfies `SwapHandler`, `SwapEngine<D>`, and `DefaultPriceLimit` for any `B: SwapBalance` and any direction `D`
- `NoOpSwap` invariants hold: `swap(n, x) == x`, `sim_swap(n, x) == x`, `approx_fee_amount(n, x) == 0`, `current_alpha_price(n) == 1`, `max_price(n) == B::max_value()`, `is_user_liquidity_enabled(n) == false`
- All 9 swap stub unit tests pass (`cargo test -p pallet-game-solver swap_stub`)
- Clippy passes with workspace-level deny lints: `cargo clippy -p pallet-game-solver --all-targets -- -D warnings`
- The `full-runtime` feature gate is the sole mechanism for enabling additional extrinsics — no other feature flag unlocks dispatch calls
- Chain spec `devnet` produces a runnable single-authority (Alice) node
- Runtime metadata is deterministic: re-building the WASM blob from the same source and toolchain produces identical metadata hash
- The `pallet-subtensor` parallel directory at `crates/myosu-chain/pallets/subtensor/` is not referenced by any default-build dependency (it is dead code in stage-0)
- No stage-0 extrinsic or storage item references swap pool state, LP positions, or AMM curves — all swap surface routes through `NoOpSwap`
- Storage item count for `pallet-game-solver` does not increase beyond 193 without explicit review

## Verification

```bash
# Build default (stage-0) runtime
cargo build -p myosu-chain-runtime --release

# Build full-runtime variant
cargo build -p myosu-chain-runtime --release --features full-runtime

# Run swap stub tests
cargo test -p pallet-game-solver swap_stub

# Run all pallet tests
cargo test -p pallet-game-solver

# Clippy with workspace deny lints
cargo clippy -p pallet-game-solver --all-targets -- -D warnings

# Count stage-0 extrinsics (should be 25)
grep -B5 'pub fn ' crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs \
  | grep -v 'full-runtime' -A5 \
  | grep 'pub fn ' | wc -l

# Count storage items (should be <=193)
grep -rc '#\[pallet::storage\]' crates/myosu-chain/pallets/game-solver/src/ | tail -1

# Verify pallet-subtensor is not in default dependency tree
cargo tree -p myosu-chain-runtime --no-default-features 2>&1 | grep pallet-subtensor

# Start devnet node (smoke test — ctrl-c after block production)
cargo run -p myosu-chain-node -- --dev --tmp
```

## Open Questions

- Can Yuma Consensus be simplified for single-subnet stage-0? The current implementation inherits multi-subnet root-network logic that may be unnecessary overhead.
- Is the opentensor polkadot-sdk fork (`rev 71629fd`) essential, or can myosu migrate to upstream? The fork's divergence scope is not yet audited.
- Which of the 25 stage-0 extrinsics are actually exercised by validators and game clients today? Coldkey swap (announce/dispute/reset), EVM key association, root claims, and symbol updates may be candidates for gating behind `full-runtime`.
- What is the target storage item count for stage-0? Plan 005 suggests ~80, but no formal audit has identified which 113 items can be removed.
- Should `NoOpSwap::max_price` return `B::one()` instead of `B::max_value()` for stage-0 safety, given that identity swaps have no slippage? The current `max_value` is safe but semantically misleading.
