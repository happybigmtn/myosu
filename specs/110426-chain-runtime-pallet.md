# Specification: Chain Runtime and Game Solver Pallet

## Objective

Define the current state and intended direction of the Substrate chain runtime, the game-solver pallet (SubtensorModule at runtime index 7), and the consensus/emission/staking surfaces. This spec covers the on-chain coordination layer that miners and validators interact with.

## Evidence Status

### Verified facts (code-grounded)

- Chain is a Substrate node using Polkadot SDK from the opentensor fork at rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61` — `Cargo.toml` workspace dependencies
- Runtime binary: `myosu-chain` (node), `myosu-chain-runtime` (WASM runtime) — `crates/myosu-chain/`
- `SubtensorModule` (pallet-game-solver) is at runtime index 7 in the runtime configuration — `crates/myosu-chain/runtime/src/lib.rs:1250`, `crates/myosu-chain/runtime/src/lib.rs:1275`
- Stage-0 consensus: Aura (block authoring) + GRANDPA (finality) — `crates/myosu-chain/Cargo.toml`
- ~31 storage items in pallet-game-solver covering subnets, neurons, weights, incentives, emission, staking — `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- Yuma consensus implementation spans epoch/run_epoch.rs + math.rs (~3200 lines) — `crates/myosu-chain/pallets/game-solver/src/epoch/`
- Emission: stage-0 floor dust with no-carry-forward policy — `crates/myosu-chain/pallets/game-solver/src/coinbase/`
- `Stage0NoopSwap`: 1:1 identity swap interface (not for funded products) — `crates/myosu-chain/pallets/game-solver/src/swap/`
- Commit-reveal v2 only (no `pallet_drand` dependency for random beacons) — AGENTS.md engineering decision
- `ALPHA_MAP_BATCH_SIZE = 30` — pallet source
- `MAX_NUM_ROOT_CLAIMS = 50` — pallet source
- `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA = 1000` rao (dust threshold) — pallet source
- Chain features: `std`, `try-runtime`, `runtime-benchmarks`, `fast-runtime` — `crates/myosu-chain-runtime/Cargo.toml`
- CI runs pallet stage-0 tests: `cargo test -p pallet-game-solver --quiet -- stage_0` — `.github/workflows/ci.yml:256`
- CI runs runtime migration smoke test with `try-runtime` and `fast-runtime` features — `.github/workflows/ci.yml:261-265`
- CI runs runtime clippy with `fast-runtime` feature — `.github/workflows/ci.yml:452`
- E2E proofs: `local_loop.sh`, `two_node_sync.sh`, `four_node_finality.sh`, `consensus_resilience.sh`, `cross_node_emission.sh`, `validator_determinism.sh`, `emission_flow.sh` — `.github/workflows/ci.yml:296-315`
- E2E integration job has 15-minute timeout — `.github/workflows/ci.yml:272`
- `SKIP_WASM_BUILD=1` used for non-chain CI jobs; chain jobs build WASM explicitly — `.github/workflows/ci.yml:83,223,273`
- `myosu-chain-client` provides shared WebSocket RPC client with `DEFAULT_WS_SCHEME = "ws://"`, `DEFAULT_POLL_INTERVAL = 500ms` — `crates/myosu-chain-client/src/lib.rs:48-52`
- `DEFAULT_NETWORK_RATE_LIMIT` and `DEFAULT_SUBNET_TEMPO` sourced from `runtime::SubtensorInitial*` constants — `crates/myosu-chain-client/src/lib.rs`
- Four-authority finality proof requires threshold 3/3 (4+ authorities to prove 1-down resilience) — AGENTS.md
- Authority-backed devnet: 48-second blocks per authored block (only authority-1 of 4), minute-scale timeouts — AGENTS.md
- Devnet startup takes >60 seconds for JSON-RPC on 127.0.0.1:9955 — AGENTS.md
- `wasm32v1-none` target required for runtime WASM compilation — `.github/workflows/ci.yml:284`
- Stripped pallets from original subtensor: collective, membership, democracy, treasury, identity, vesting, utility (old), tips, bounties, asset-tx-payment, contracts, EVM — AGENTS.md / chain runtime
- Remaining optional pallets gated behind `full-runtime` feature: multisig, preimage, safe-mode, scheduler, proxy, sudo — runtime configuration

### Recommendations (intended system)

- Emission rewrite should be completed as new implementation, not ported from original subtensor (80% unused) — AGENTS.md engineering decision
- Single-token model (not dual Alpha/TAO) — AGENTS.md engineering decision
- `SubtensorModule` naming is inherited from fork; renaming to `GameSolverModule` or similar deferred to post-stage-0

### Hypotheses / unresolved questions

- Whether the opentensor polkadot-sdk fork will be rebased to a newer upstream version is unresolved (CHAIN-SDK-001 risk)
- Runtime benchmark weights may not be calibrated for stage-0 hardware profile
- Whether `pallet_drand` will be needed for future randomness requirements beyond commit-reveal v2

## Acceptance Criteria

- Chain produces blocks locally with single-authority Aura consensus
- `SubtensorModule` resolves at runtime index 7
- Pallet stage-0 tests pass: `cargo test -p pallet-game-solver --quiet -- stage_0`
- Runtime migration smoke test passes on fresh genesis
- Two-node sync proof demonstrates block propagation
- Four-authority finality proof demonstrates GRANDPA consensus under 1-down resilience
- Consensus resilience proof demonstrates recovery after authority restart
- Cross-node emission agreement proves identical emission accounting across nodes
- Validator determinism proof demonstrates identical scoring (INV-003: epsilon < 1e-6)
- Emission flow proof verifies `sum(distributions) == block_emission * epochs`
- Dust stays within `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA` (1000 rao)
- `Stage0NoopSwap` performs 1:1 identity transformation (no economic logic)
- `cargo clippy -p myosu-chain-runtime --features fast-runtime -- -D warnings` passes
- `cargo clippy -p pallet-game-solver -- -D warnings` passes

## Verification

```bash
# Pallet stage-0 tests
SKIP_WASM_BUILD=1 cargo test -p pallet-game-solver --quiet -- stage_0

# Runtime tests
SKIP_WASM_BUILD=1 cargo test -p myosu-chain-runtime --quiet

# Runtime migration smoke test (requires WASM build)
cargo build -p myosu-chain-runtime --features fast-runtime --quiet
cargo test -p myosu-chain --features fast-runtime,try-runtime --quiet \
  devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis

# E2E integration proofs (each script is self-contained)
bash tests/e2e/local_loop.sh
bash tests/e2e/two_node_sync.sh
bash tests/e2e/four_node_finality.sh
bash tests/e2e/consensus_resilience.sh
bash tests/e2e/cross_node_emission.sh
bash tests/e2e/validator_determinism.sh
bash tests/e2e/emission_flow.sh

# Chain clippy
SKIP_WASM_BUILD=1 cargo clippy -p myosu-chain-runtime --features fast-runtime -- -D warnings
SKIP_WASM_BUILD=1 cargo clippy -p pallet-game-solver -- -D warnings
SKIP_WASM_BUILD=1 cargo clippy -p myosu-chain --features fast-runtime -- -D warnings
```

## Open Questions

1. **Emission rewrite status:** The coinbase module implements stage-0 floor dust. How much of the Yuma consensus epoch runner is fully operational vs. placeholder? The ~3200-line epoch implementation may have ported-but-untested paths.
2. **Rate limiting calibration:** Hyperparameter-based transaction rate limiting per subnet uses `SubtensorInitialNetworkRateLimit = 0` in stage-0. Is zero the intentional local-proof default, or does it disable rate limiting entirely?
3. **`SubtensorInitialTempo = 2`:** Stage-0 subnet tempo of 2 blocks per epoch is very fast. Is this calibrated for local proof only, and what's the intended production value?
4. **Fork maintenance:** The opentensor polkadot-sdk fork (CHAIN-SDK-001) pins at a specific rev. Who monitors upstream security patches? What's the rebase trigger?
5. **Pallet naming:** `SubtensorModule` / `pallet-game-solver` naming divergence between code and documentation could confuse operators. Is renaming planned?
6. **`full-runtime` feature scope:** Optional pallets (multisig, preimage, safe-mode, scheduler, proxy, sudo) are gated. Which are needed for stage-1 and which are candidates for removal?
