# Specification: Chain Fork Scaffold — Minimal Substrate Chain from Subtensor

Source: Master spec AC-CH-01, subtensor runtime analysis at coding/subtensor
Status: Draft
Date: 2026-03-30
Depends-on: none

## Purpose

Fork Bittensor's subtensor Substrate runtime into a minimal, compilable chain
that produces blocks on a local devnet. This spec strips all AI/ML-specific
pallets (weights, epochs, EVM, AMM, randomness beacon) while preserving the
core Substrate infrastructure (consensus, accounts, balances, transaction
payment). The result is a blank-slate chain ready to receive the game-solving
pallet (spec: `031626-03-game-solving-pallet.md`).

The primary consumer is every subsequent spec — nothing works without a
running chain. The secondary consumer is the developer, who needs a
`myosu-node --dev` command that starts a local devnet in under 10 seconds.

**Key design constraint**: the fork must compile and produce blocks before any
game-specific code is added. This separates "does Substrate work?" from "does
our game logic work?" and prevents debugging two things at once.

## Whole-System Goal

Current state:
- `crates/myosu-chain/runtime/` and `crates/myosu-chain/node/` already compile
  as a stripped stage-0 chain
- the owned local proof `myosu-chain --stage0-local-loop-smoke` already boots a
  node, authors blocks, and drives the current poker plus Liar's Dice loop
- the remaining drift is no longer "fork subtensor from scratch", but keeping
  the inherited chain surface, ownership map, and launch doctrine honest about
  the code that now exists

This spec adds:
- the canonical description of the stripped chain boundary that stage 0 now
  actually uses
- the remaining reduction and packaging work needed on top of the live chain
- the ownership map for the node, runtime, and local proof surfaces under
  `crates/myosu-chain/`

If all ACs land:
- `cargo build -p myosu-node --release` compiles the chain binary
- `myosu-node --dev` starts producing blocks within 10 seconds
- RPC endpoint responds at `ws://localhost:9944`
- Test accounts (Alice, Bob, etc.) have funded balances
- The runtime is ready to accept a new game-solving pallet at index 7

Still not solved here:
- Game-solving pallet (CH-02 spec)
- Custom token economics
- Production chain spec or mainnet configuration
- Validator set management beyond dev mode

12-month direction:
- Production-grade chain with custom consensus parameters
- Multi-validator devnet for testing P2P dynamics
- Benchmarked runtime weights for all extrinsics
- Upgradeable runtime via Sudo

## Why This Spec Exists As One Unit

- Stripping pallets, updating the runtime composition, fixing the node binary,
  and creating a chain spec are all required for a single outcome: "chain
  compiles and produces blocks"
- Doing any of these in isolation is untestable — you can't verify a runtime
  that doesn't compile, and you can't compile a runtime with half-removed pallets
- The ~5 ACs form a single atomic build target

## Scope

In scope:
- Fork subtensor runtime into `crates/myosu-chain/`
- Strip AI/EVM/AMM pallets from `construct_runtime!`
- Remove corresponding Config impls and dependencies
- Create minimal node binary
- Create local devnet chain spec
- Verify block production and RPC

Out of scope:
- Game-solving pallet — that's a separate spec (031626-03-game-solving-pallet.md)
- Custom transaction extensions — keep standard Substrate extensions
- EVM or smart contract support — stripped and not coming back
- Production chain spec — devnet only
- Benchmarks — will be added when game pallet lands

## Current State

- `/home/r/coding/myosu/crates/myosu-chain/runtime/src/lib.rs` — live stripped
  runtime carrying the current stage-0 pallet set and runtime wrappers
- `/home/r/coding/myosu/crates/myosu-chain/node/src/service.rs` — live node
  service with startup timing and RPC readiness logging
- `/home/r/coding/myosu/crates/myosu-chain/node/src/chain_spec/` — devnet,
  localnet, testnet, and finney chain-spec surfaces owned in-repo
- `/home/r/coding/myosu/crates/myosu-chain/node/tests/stage0_local_loop.rs` —
  owned proof that the local chain boots and carries the two-subnet stage-0
  loop
- `/home/r/coding/subtensor/` remains the inherited source context, but it is
  no longer the primary execution surface for this repo

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Runtime shell | `crates/myosu-chain/runtime/src/lib.rs` | extend | This is the live stripped runtime, not a future fork target |
| Node binary | `crates/myosu-chain/node/src/main.rs` | extend | Current executable entrypoint for stage-0 proofs |
| Node service | `crates/myosu-chain/node/src/service.rs` | extend | Owns block production, networking, and readiness timing |
| Local chain specs | `crates/myosu-chain/node/src/chain_spec/*.rs` | extend | Current in-repo devnet/localnet truth surfaces |
| Consensus wiring | `crates/myosu-chain/node/src/consensus/` | reuse with care | The live node still carries owned consensus composition here |
| RPC assembly | `crates/myosu-chain/node/src/rpc.rs` | extend | Current custom RPC merge point and operator seam |
| Owned launch proof | `crates/myosu-chain/node/tests/stage0_local_loop.rs` | reuse | The chain already proves the current local loop here |

## Non-goals

- Adding any game-specific logic — this spec produces a blank chain
- Custom consensus parameters — standard Substrate defaults are fine for devnet
- Preserving subtensor's migration system — fresh chain, no migration needed
- Supporting the subtensor pallet's 150+ storage items — all stripped
- EVM compatibility — permanently removed, not deferred

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Runtime composition | Implemented | crates/myosu-chain/runtime/src/lib.rs |
| Node binary | Implemented | crates/myosu-chain/node/src/main.rs |
| Node service | Implemented | crates/myosu-chain/node/src/service.rs |
| Chain spec family | Implemented | crates/myosu-chain/node/src/chain_spec/ |
| Runtime build | Implemented | crates/myosu-chain/runtime/build.rs |
| Local loop proof | Implemented | crates/myosu-chain/node/tests/stage0_local_loop.rs |

## Architecture / Runtime Contract

```
construct_runtime! {
    System:             frame_system                    = 0,
    RandomnessFlip:     pallet_insecure_randomness      = 1,
    Timestamp:          pallet_timestamp                = 2,
    Aura:               pallet_aura                     = 3,
    Grandpa:            pallet_grandpa                  = 4,
    Balances:           pallet_balances                 = 5,
    TransactionPayment: pallet_transaction_payment      = 6,
    // index 7 reserved for pallet_game_solver (CH-02)
    Utility:            pallet_utility                  = 11,
    Sudo:               pallet_sudo                     = 12,
    Multisig:           pallet_multisig                 = 13,
    Preimage:           pallet_preimage                 = 14,
    Scheduler:          pallet_scheduler                = 15,
    Proxy:              pallet_proxy                    = 16,
    SafeMode:           pallet_safe_mode                = 20,
}
```

Primary loop:
- Trigger: `myosu-node --dev` CLI command
- Source of truth: genesis chain spec
- Processing: Aura authorship + GRANDPA finality
- Persisted truth: block database (RocksDB)
- Consumer: subsequent specs (game pallet, miner, validator)

Failure loop:
- Missing dependency → compilation error → fix Cargo.toml
- Runtime panic → node exits → check logs, fix runtime code
- Genesis mismatch → chain won't start → regenerate chain spec

## Adoption / Consumption Path

- Producer: this spec produces a compilable chain binary
- First consumer: AC-CH-02 (game-solving pallet) builds on this runtime
- Operator-visible surface: `myosu-node --dev` CLI + RPC endpoint
- Why it changes behavior now: every subsequent spec depends on block production
- If not consumed yet: N/A — this is the foundation, consumed immediately

---

## A. Runtime Stripping

### AC-CF-01: Strip AI/EVM Pallets from Runtime

- Where: `crates/myosu-chain/runtime/src/lib.rs (new, from subtensor)`
- How: Copy `subtensor/runtime/src/lib.rs` to `crates/myosu-chain/runtime/src/lib.rs`.
  Remove these pallets from `construct_runtime!` and their corresponding
  `impl pallet_*::Config for Runtime` blocks:

  **Remove (indices 7, 17-19, 21-30):**
  | Index | Pallet | Why strip |
  |-------|--------|-----------|
  | 7 | pallet_subtensor (SubtensorModule) | AI weights/epochs/subnets — replaced by game-solver |
  | 17 | pallet_registry | Bittensor identity — not needed |
  | 18 | pallet_commitments | Bittensor proof commitments — not needed |
  | 19 | pallet_admin_utils | Bittensor authority management — Sudo suffices |
  | 21 | pallet_ethereum | Frontier EVM transactions |
  | 22 | pallet_evm | Frontier EVM execution |
  | 23 | pallet_evm_chain_id | Frontier chain ID |
  | 25 | pallet_base_fee | Frontier dynamic fees |
  | 26 | pallet_drand | Randomness beacon |
  | 27 | pallet_crowdloan | Crowdfunding |
  | 28 | pallet_subtensor_swap | TAO/Alpha AMM |
  | 29 | pallet_contracts | WASM smart contracts |
  | 30 | pallet_shield | MEV protection |

  **Keep (indices 0-6, 11-16, 20) — 13 pallets after CF-01 (14th added by GS-09):**
  System(0), RandomnessFlip(1), Timestamp(2), Aura(3), Grandpa(4),
  Balances(5), TransactionPayment(6), Utility(11), Sudo(12), Multisig(13),
  Preimage(14), Scheduler(15), Proxy(16), SafeMode(20).
  Index 7 reserved for pallet_game_solver (added in GS-09).

  Remove all `use` imports for stripped pallets. Remove any conditional
  compilation blocks (`#[cfg(feature = "...")]`) that reference stripped
  pallets. Update `type SignedExtra` to remove `SubtensorTransactionExtension`
  and `DrandPriority`. Remove benchmark configs for stripped pallets.

  Change `spec_name` from `"node-subtensor"` to `"myosu"`.
  Change `impl_name` from `"node-subtensor"` to `"myosu-node"`.
  Reset `spec_version` to `1`.

- Whole-system effect: produces a compilable runtime WASM blob. Without this,
  the node binary has nothing to execute.
- State: runtime version metadata, pallet registry.
- Wiring contract:
  - Trigger: `cargo build -p myosu-runtime`
  - Callsite: `crates/myosu-chain/runtime/build.rs` (WASM builder)
  - State effect: WASM blob compiled into `target/`
  - Persistence effect: runtime binary artifact
  - Observable signal: `cargo build -p myosu-runtime` exits 0
- Required tests:
  - `cargo test -p myosu-runtime runtime::tests::runtime_compiles`
  - `cargo test -p myosu-runtime runtime::tests::version_is_myosu`
  - `cargo test -p myosu-runtime runtime::tests::pallet_count_is_correct`
- Pass/fail:
  - `cargo build -p myosu-runtime` compiles without errors
  - Runtime version reports `spec_name: "myosu"`, `spec_version: 1`
  - `construct_runtime!` contains exactly 13 pallets (System through SafeMode, index 7 reserved)
  - No references to `pallet_subtensor`, `pallet_ethereum`, `pallet_evm`,
    `pallet_drand`, `pallet_shield`, `pallet_crowdloan`, `pallet_subtensor_swap`
    remain in the compiled runtime
  - WASM binary size < 5MB (subtensor's is ~15MB with EVM)
- Blocking note: everything downstream depends on a compilable runtime. This
  AC is the single-most-critical unblock in the entire project.
- Rollback condition: runtime fails to compile after stripping, indicating
  hidden dependencies between pallets that require deeper surgery.

### AC-CF-02: Prune Workspace Dependencies

- Where: `crates/myosu-chain/Cargo.toml (new)`, `crates/myosu-chain/runtime/Cargo.toml (new)`
- How: Create the workspace Cargo.toml for `myosu-chain` with only the
  dependencies needed by kept pallets. Remove from `[workspace.dependencies]`:

  ```
  # Bittensor-specific
  pallet_subtensor, pallet_subtensor_swap, pallet_subtensor_swap_runtime_api,
  pallet_subtensor_swap_rpc, pallet_drand, pallet_shield, pallet_crowdloan,
  pallet_registry, pallet_commitments, pallet_admin_utils

  # Frontier EVM
  fp-evm, fp-rpc, fp-self-contained, fp-storage, fc-consensus, fc-db,
  fc-mapping-sync, fc-rpc, fc-rpc-core, fc-storage, pallet_ethereum,
  pallet_evm, pallet_evm_chain_id, pallet_base_fee

  # Other
  subtensor-macros, subtensor-custom-rpc, subtensor-custom-rpc-runtime-api
  ```

  Keep the Polkadot SDK fork rev (`71629fd93b6c12a362a5cfb6331accef9b2b2b61`)
  and all standard `frame-*`, `sp-*`, `pallet-*` Substrate dependencies.

  The `runtime/Cargo.toml` should list only:
  - `frame-support`, `frame-system`, `frame-executive`
  - `sp-api`, `sp-runtime`, `sp-core`, `sp-io`, `sp-std`, `sp-version`
  - Kept pallets: `pallet-aura`, `pallet-grandpa`, `pallet-balances`,
    `pallet-timestamp`, `pallet-transaction-payment`, `pallet-sudo`,
    `pallet-utility`, `pallet-multisig`, `pallet-preimage`,
    `pallet-scheduler`, `pallet-proxy`, `pallet-safe-mode`,
    `pallet-insecure-randomness-collective-flip`
  - `substrate-wasm-builder` (build dependency)

- Whole-system effect: without clean dependencies, the build pulls in
  megabytes of unused code and may fail on missing transitive deps.
- State: no runtime state — build configuration.
- Wiring contract:
  - Trigger: `cargo build -p myosu-runtime`
  - Callsite: Cargo resolver
  - State effect: dependency tree resolved
  - Persistence effect: Cargo.lock generated
  - Observable signal: `cargo tree -p myosu-runtime` shows no subtensor/frontier crates
- Required tests:
  - `cargo tree -p myosu-runtime 2>&1 | grep -c "pallet.subtensor"` → 0
  - `cargo tree -p myosu-runtime 2>&1 | grep -c "pallet.ethereum"` → 0
  - `cargo tree -p myosu-runtime 2>&1 | grep -c "frontier"` → 0
- Pass/fail:
  - `cargo tree -p myosu-runtime` contains zero references to stripped crates
  - `cargo build -p myosu-runtime` succeeds
  - Build time < 10 minutes on a 16-core machine (subtensor takes ~20 min)
  - WASM blob produced in `target/release/wbuild/myosu-runtime/`
- Blocking note: dependency contamination from stripped pallets will cause
  build failures or bloat. Clean deps enable fast iteration.
- Rollback condition: transitive dependency from a kept pallet to a stripped
  pallet that can't be resolved without deeper changes.

---

## B. Node Binary

### AC-CF-03: Minimal Node Service

- Where: `crates/myosu-chain/node/src/service.rs (new, from subtensor)`
- How: Copy `subtensor/node/src/service.rs` and strip:
  - All Frontier EVM block import, mapping, and sync code
  - Drand integration (HTTP beacon fetching)
  - MEV shield encrypted transaction handling
  - Subtensor custom RPC extensions
  - Conditional EVM block import logic

  Keep:
  - Standard Substrate service builder pattern
  - Aura block authoring
  - GRANDPA finality gadget
  - Standard Substrate RPC (system, chain, author, state)
  - Offchain worker infrastructure (may be needed later)
  - Telemetry (optional but useful)

  Create `crates/myosu-chain/node/src/main.rs` with standard Substrate CLI
  (`sc_cli::SubstrateCli`). Support subcommands: `--dev`, `build-spec`,
  `export-blocks`, `import-blocks`, `purge-chain`.

  Create `crates/myosu-chain/node/src/cli.rs` with `Cli` struct deriving
  `clap::Parser` and `sc_cli::SubstrateCli`.

  Create `crates/myosu-chain/node/src/command.rs` routing subcommands.

- Whole-system effect: the node binary is how anyone runs a myosu chain.
  Without it, the runtime WASM is just a blob on disk.
- State: node service state (block database, network peers, RPC server).
- Wiring contract:
  - Trigger: `myosu-node --dev` CLI command
  - Callsite: `crates/myosu-chain/node/src/main.rs`
  - State effect: node service starts, block production begins
  - Persistence effect: block database written to `--base-path` directory
  - Observable signal: log output shows "Idle" or block numbers incrementing
- Required tests:
  - `cargo test -p myosu-node node::tests::cli_parses`
  - `cargo test -p myosu-node node::tests::service_starts`
- Pass/fail:
  - `cargo build -p myosu-node` succeeds
  - `myosu-node --dev` starts and prints block production logs
  - `myosu-node --help` shows available subcommands
  - `myosu-node build-spec --chain local` outputs valid JSON chain spec
  - Node shuts down cleanly on Ctrl+C (SIGINT)
  - No panics on startup with default configuration
- Blocking note: the node binary is the entry point for developers. Every
  test of the chain requires running the node.
- Rollback condition: service.rs has deep coupling to Frontier or drand that
  can't be cleanly removed without major refactoring.

### AC-CF-04: Local Devnet Chain Spec

- Where: `crates/myosu-chain/node/src/chain_spec.rs (new, from subtensor)`
- How: Create a single `chain_spec.rs` (no multi-file split needed for devnet).
  Base on `subtensor/node/src/chain_spec/localnet.rs`.

  Genesis configuration:
  ```rust
  // Consensus authorities
  Aura: { authorities: [alice_aura_key] }  // single validator for --dev
  Grandpa: { authorities: [(alice_grandpa_key, 1)] }

  // Funded accounts (1,000,000 MYOSU each, 9 decimal places)
  Balances: {
      balances: [
          (alice, 1_000_000_000_000_000),
          (bob,   1_000_000_000_000_000),
          (charlie, 1_000_000_000_000_000),
          (dave,  1_000_000_000_000_000),
          (eve,   1_000_000_000_000_000),
          (ferdie, 1_000_000_000_000_000),
      ]
  }

  // Governance
  Sudo: { key: alice }
  ```

  No subtensor genesis, no EVM genesis, no drand genesis.
  Properties: `token_symbol: "MYOSU"`, `token_decimals: 9`, `ss58_format: 42`.

  **Dev mode genesis subnet**: When GS-09 integrates the game-solver pallet,
  the dev chain spec should include a genesis subnet (subnet 1, game_type
  `b"nlhe_hu"`, owned by Alice) so integration tests don't need explicit
  subnet creation as a setup step. Document this cross-AC dependency.

  Support `--dev` (single validator, instant seal) and `--chain local`
  (two validators, Alice + Bob, 6-second blocks).

- Whole-system effect: defines the initial state of the chain. Without a
  valid chain spec, the node cannot start.
- State: genesis block state (accounts, authorities, sudo key).
- Wiring contract:
  - Trigger: node startup reads chain spec
  - Callsite: `crates/myosu-chain/node/src/command.rs`
  - State effect: genesis block initialized with configured state
  - Persistence effect: genesis block written to block database
  - Observable signal: `myosu-node build-spec --chain local --raw` outputs JSON
- Required tests:
  - `cargo test -p myosu-node chain_spec::tests::dev_spec_is_valid`
  - `cargo test -p myosu-node chain_spec::tests::local_spec_is_valid`
  - `cargo test -p myosu-node chain_spec::tests::genesis_accounts_funded`
- Pass/fail:
  - Dev chain spec creates a valid genesis block
  - Local chain spec creates a valid genesis block with 2 authorities
  - Alice, Bob, Charlie, Dave, Eve, Ferdie all have 1,000,000 MYOSU
  - Alice is sudo key
  - Token symbol is "MYOSU" with 9 decimals
  - `build-spec --chain local --raw` produces valid JSON
  - Chain spec contains no subtensor, EVM, or drand genesis configuration
- Blocking note: the chain spec defines what the chain looks like at block 0.
  An invalid spec means the chain can't start.
- Rollback condition: genesis configuration requires types from stripped pallets.

---

## C. Integration Verification

### AC-CF-05: End-to-End Devnet Smoke Test

- Where: `crates/myosu-chain/tests/ (new)`
- How: Integration test that:
  1. Starts `myosu-node --dev` as a child process
  2. Waits for RPC to become available (poll `system_health`)
  3. Verifies block production (block number > 0 within 15 seconds)
  4. Queries account balances via RPC (`system.account`)
  5. Submits a balance transfer (Alice → Bob, 100 MYOSU) via RPC
  6. Verifies the transfer succeeded (Bob's balance increased)
  7. Shuts down the node cleanly

  This test proves the entire vertical slice works: compilation → node
  startup → genesis → block production → transaction execution → RPC query.

  Use `subxt` or raw JSON-RPC HTTP calls for chain interaction. The test
  should timeout after 60 seconds if the chain doesn't start.

- Whole-system effect: this is the gate that proves CH-01 is complete. If
  this test passes, the chain is ready for game pallet development.
- State: test harness manages node lifecycle.
- Wiring contract:
  - Trigger: `cargo test -p myosu-chain integration::tests::devnet_smoke_test`
  - Callsite: `crates/myosu-chain/tests/integration.rs`
  - State effect: devnet starts, transaction processed, node stops
  - Persistence effect: temporary block database (cleaned up after test)
  - Observable signal: test passes, block number > 0, transfer confirmed
- Required tests:
  - `cargo test -p myosu-chain integration::tests::devnet_smoke_test`
  - `cargo test -p myosu-chain integration::tests::rpc_system_health`
  - `cargo test -p myosu-chain integration::tests::balance_transfer`
- Pass/fail:
  - Node starts and produces block 1 within 15 seconds
  - `system_health` RPC returns `{ peers: 0, isSyncing: false, shouldHavePeers: false }`
  - Alice's balance is 1,000,000 MYOSU at genesis
  - Balance transfer of 100 MYOSU from Alice to Bob succeeds
  - Bob's balance increases by 100 MYOSU (minus fees)
  - Node shuts down cleanly (exit code 0)
  - No panics in node logs during the entire test
- Blocking note: without this integration test, we can't be sure the chain
  actually works end-to-end. Unit tests on individual pallets are necessary
  but not sufficient.
- Rollback condition: node fails to start, RPC doesn't respond, or
  transactions fail — indicating deeper issues in the runtime stripping.

---

## D. Subtensor Entanglement Surgery (audit-discovered prerequisites)

These ACs were discovered during the 2026-03-17 pre-implementation audit.
Subtensor's pallet_subtensor has deep coupling to drand, crowdloan, Frontier,
and the AMM swap pallet that was not visible from the construct_runtime!
macro alone. These must all be resolved before CF-01 can strip pallets.

### Critical genesis requirements (discovered in final 10/10 audit)

The following must be set in genesis or emission will SILENTLY never flow:
- `FirstEmissionBlockNumber(netuid, 1)` — without this, subnet is filtered
  from `get_subnets_to_emit_to()` and receives zero emission forever
- `SubtokenEnabled(netuid, true)` — same filter
- `SubnetMechanism(netuid, 0)` — set to 0 (Stable) for single-token 1:1 swap
  (NOT 1/Dynamic which invokes real AMM price curves)

Additional genesis notes:
- Override `on_runtime_upgrade` to no-op (skip all 42 subtensor migrations)
- Replace `SubtensorTxFeeHandler` with standard `FungibleAdapter` (CF-08)
- Strip leasing extrinsics (call_index 110/111) that reference stripped types
- Standard `pallet_utility` can replace subtensor's fork (strips `if_else`
  and `dispatch_as_fallible` extrinsics, which we don't need)

```
CF-07 (strip drand/crowdloan supertraits)  ← FIRST COMMIT
  │
  ├──► CF-06 (SwapInterface no-op stub)
  ├──► CF-08 (replace fp_self_contained)
  ├──► CF-09 (strip CRV3 timelock)
  ├──► CF-10 (port safe-math + share-pool)
  └──► CF-11 (stub ProxyInterface + CommitmentsInterface)
        │
        ▼
      CF-02 (prune deps) → CF-01 (strip pallets) → CF-04/CF-03/CF-05
```

### AC-CF-06: SwapInterface No-Op Stub

- Where: `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs (new)`
- How: Create a no-op implementation of the `SwapInterface` trait (which
  combines `SwapHandler` + `SwapEngine`). The stub performs identity swaps:
  `swap_tao_for_alpha(amount)` returns `amount` unchanged (no AMM, no pool).
  `swap_alpha_for_tao(amount)` returns `amount` unchanged. In myosu's
  single-token model, there is no Alpha/TAO distinction — MYOSU tokens
  are burned directly during registration and staked directly during
  add_stake.

  Callsites in subtensor that use SwapInterface (37 total):
  - `staking/stake_utils.rs` — 12 calls (swap, sim_swap, current_alpha_price, etc.)
  - `staking/add_stake.rs` — 2 calls (max_price, swap)
  - `staking/remove_stake.rs` — 5 calls (min_price, max_price, swap, current_alpha_price)
  - `staking/move_stake.rs` — 4 calls (min_price, max_price, current_alpha_price)
  - `staking/helpers.rs` — 6 calls (current_alpha_price, sim_swap, min_price, is_user_liquidity_enabled)
  - `staking/claim_root.rs` — 1 call (min_price)
  - `subnets/registration.rs` — 1 call (max_price)
  - `coinbase/run_coinbase.rs` — 3 calls (adjust_protocol_liquidity, max_price, current_alpha_price)
  - `coinbase/root.rs` — 2 calls (dissolve_all_liquidity_providers, clear_protocol_liquidity)
  - `rpc_info/stake_info.rs` — 1 call (approx_fee_amount)

  Identity stub (~80-100 LOC): swap returns input unchanged, sim_swap same,
  current_alpha_price returns 1:1, all fee methods return zero, liquidity
  methods are no-ops.

- Whole-system effect: unblocks registration, staking, and emission without
  porting the entire AMM swap pallet (800+ lines).
- State: no new state — the stub is stateless.
- Wiring contract:
  - Trigger: any extrinsic that previously used AMM swaps
  - Callsite: registration.rs, staking/*.rs, coinbase/*.rs
  - State effect: tokens transferred directly (no pool intermediary)
  - Persistence effect: balance changes only
  - Observable signal: registration/staking/emission work without swap pallet
- Required tests:
  - `cargo test -p pallet-game-solver swap_stub::tests::identity_swap`
  - `cargo test -p pallet-game-solver swap_stub::tests::registration_uses_stub`
- Pass/fail:
  - `swap_tao_for_alpha(100)` returns 100
  - `swap_alpha_for_tao(100)` returns 100
  - Registration burns tokens directly (no pool interaction)
  - add_stake credits stake directly (no pool interaction)
  - Emission distributes tokens directly (no pool interaction)
- Blocking note: SwapInterface is called in 3 of 4 core extrinsic paths.
  Without this stub, nothing compiles after stripping the swap pallet.
- Rollback condition: SwapInterface trait has methods beyond swap that are
  called in critical paths we haven't identified.

### AC-CF-07: Strip drand and crowdloan Config Supertraits

- Where: `crates/myosu-chain/pallets/game-solver/src/macros/config.rs (from subtensor)`
- How: Remove `+ pallet_drand::Config + pallet_crowdloan::Config` from the
  Config trait definition at `config.rs:17`. Remove ALL `T: Config +
  pallet_drand::Config` bounds (found in `block_step.rs:6` and elsewhere).
  Remove `subnets/leasing.rs` entirely (reads pallet_crowdloan::Contributions).
  Remove `reveal_crv3_commits()` call from `block_step()`. Remove all
  imports of `pallet_drand::*` and `pallet_crowdloan::*`.

  This is the FIRST COMMIT in the fork. Without it, `cargo check` fails
  because the Config trait requires the stripped pallets to exist.

- Whole-system effect: makes the pallet compilable without drand and crowdloan
  dependencies. This is the single hardest blocker in the entire fork.
- State: no state change — removes trait requirements.
- Wiring contract:
  - Trigger: `cargo check -p pallet-game-solver`
  - Callsite: config.rs Config trait definition
  - State effect: Config trait compiles standalone
  - Persistence effect: N/A
  - Observable signal: `cargo check` exits 0
- Required tests:
  - `cargo check -p pallet-game-solver`
- Pass/fail:
  - Config trait compiles without pallet_drand or pallet_crowdloan
  - No references to `pallet_drand::` in the crate
  - No references to `pallet_crowdloan::` in the crate
  - `leasing.rs` removed
  - `reveal_crv3_commits()` removed from block_step
- Blocking note: pallet_subtensor::Config REQUIRES pallet_drand::Config as a
  supertrait. This is a compilation firewall — you cannot even run `cargo check`
  without addressing this first.
- Rollback condition: Config has deeper coupling to drand types beyond the
  supertrait (e.g., associated types that reference drand storage).

### AC-CF-08: Replace fp_self_contained Extrinsic Types

- Where: `crates/myosu-chain/runtime/src/lib.rs (from subtensor)`
- How: Replace Frontier's extrinsic types with standard Substrate types:
  ```rust
  // Before (subtensor):
  pub type UncheckedExtrinsic = fp_self_contained::UncheckedExtrinsic<...>;
  pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<...>;

  // After (myosu):
  pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
  pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
  ```

  Remove `DrandPriority<Runtime>` from the `SignedExtra` /
  `TransactionExtensions` tuple. Remove the `Ethereum::on_finalize()` call
  from the runtime API implementation.

- Whole-system effect: removes Frontier dependency from the runtime type system.
- State: no state change — type aliases.
- Wiring contract:
  - Trigger: `cargo build -p myosu-runtime`
  - Callsite: runtime/src/lib.rs type aliases
  - State effect: extrinsic types use standard Substrate encoding
  - Persistence effect: N/A
  - Observable signal: runtime compiles without fp_self_contained
- Required tests:
  - `cargo check -p myosu-runtime`
- Pass/fail:
  - No references to `fp_self_contained` in runtime
  - No references to `DrandPriority` in SignedExtra
  - No references to `Ethereum::on_finalize` in runtime API
  - Standard Substrate extrinsic encoding works
- Blocking note: without this, removing Frontier dependencies causes type errors
  in the core extrinsic pipeline.
- Rollback condition: other runtime code depends on fp_self_contained methods
  (e.g., self-contained transaction validation).

### AC-CF-09: Strip CRV3 Timelock Commit-Reveal Path

- Where: `crates/myosu-chain/pallets/game-solver/src/coinbase/reveal_commits.rs (delete)`,
  `src/subnets/weights.rs (modify)`, `src/coinbase/block_step.rs (modify)`
- How: Remove the CRV3 (Commit-Reveal Version 3, timelock encryption via drand)
  path entirely. Keep only the hash-based commit-reveal v2:
  - Delete `reveal_crv3_commits()` function and `TimelockedWeightCommits` storage
  - Delete `CRV3WeightCommits`, `CRV3WeightCommitsV2` storage items
  - Keep `WeightCommits` storage, `commit_weights()`, `reveal_weights()` extrinsics
  - Remove the `DrandPriority` transaction extension

  The v2 flow is: validator calls `commit_weights(hash)` → waits N blocks →
  calls `reveal_weights(uids, values, salt)` → pallet verifies
  `hash(uids, values, salt) == committed_hash` → stores weights.

- Whole-system effect: removes the last drand dependency from the pallet code.
- State: removes 3 storage items, keeps WeightCommits.
- Wiring contract:
  - Trigger: validator calls commit_weights/reveal_weights
  - Callsite: weights.rs
  - State effect: weight hash stored, then verified and applied
  - Persistence effect: WeightCommits → Weights on reveal
  - Observable signal: commit + reveal flow succeeds in test
- Required tests:
  - `cargo test -p pallet-game-solver weights::tests::commit_reveal_v2_works`
  - `cargo test -p pallet-game-solver weights::tests::reveal_wrong_hash_fails`
- Pass/fail:
  - `commit_weights(hash)` stores hash in WeightCommits
  - `reveal_weights(uids, values, salt)` succeeds if hash matches
  - `reveal_weights` with wrong salt fails
  - No references to `TimelockedWeightCommits` or `CRV3` in crate
  - `block_step()` does not call `reveal_crv3_commits()`
- Blocking note: CRV3 reads `pallet_drand::Pulses` which doesn't exist in our
  fork. Without stripping it, weight submission panics.
- Rollback condition: v2 commit-reveal has a timing vulnerability that CRV3
  was specifically designed to prevent.

### AC-CF-10: Port Primitives and Runtime Common Types

- Where: `crates/myosu-chain/primitives/safe-math/ (new, from subtensor/primitives/safe-math/)`,
  `crates/myosu-chain/primitives/share-pool/ (new, from subtensor/primitives/share-pool/)`,
  `crates/myosu-chain/common/ (new, from subtensor/common/)`
- How: Copy three crates from subtensor into the myosu workspace:
  - `safe-math` (384 lines): saturating arithmetic operations on `substrate_fixed`
    types (I32F32, I64F64). Used extensively in epoch code.
  - `share-pool` (452 lines): proportional share accounting for staking. Handles
    mint/burn/value calculations on U64F64 shares. Cannot be replaced with a
    simple StorageDoubleMap — 20+ stake functions depend on it.
  - `subtensor_runtime_common` (common/): provides `NetUid`, `MechId`,
    `NetUidStorageIndex`, `TaoCurrency`, `AlphaCurrency`, `Currency` trait,
    `AuthorshipInfo` trait. Used in nearly every pallet file. Rename to
    `myosu_runtime_common`. For the single-token model, `AlphaCurrency` and
    `TaoCurrency` can be aliased to the same underlying type.

  All depend on `substrate_fixed` (encointer fork, v0.6.0).
  Pin the same git dependency.

- Whole-system effect: provides the math foundation for Yuma Consensus and
  staking. Without these, GS-05 and GS-08 cannot be ported.
- State: no runtime state — pure functions.
- Wiring contract:
  - Trigger: epoch.rs and staking.rs import these
  - Callsite: math operations in consensus and staking
  - State effect: N/A (pure functions)
  - Persistence effect: N/A
  - Observable signal: existing tests pass in myosu workspace
- Required tests:
  - `cargo test -p myosu-safe-math`
  - `cargo test -p myosu-share-pool`
  - `cargo test -p myosu-runtime-common`
- Pass/fail:
  - All existing safe-math and share-pool tests pass without modification
  - `substrate_fixed` v0.6.0 from encointer fork resolves
  - `myosu_runtime_common::NetUid`, `MechId`, `NetUidStorageIndex` importable
  - `TaoCurrency` and `AlphaCurrency` both alias to same underlying type
  - No dependencies on stripped pallets (drand, crowdloan, swap)
- Blocking note: epoch code uses `safe_math::*` throughout. share-pool is used
  by ALL stake operations (20+ functions in stake_utils.rs). `runtime_common`
  provides types used in nearly every pallet file. Without these three crates,
  nothing compiles.
- Rollback condition: runtime_common has deep coupling to stripped pallet types
  that can't be aliased away.

### AC-CF-11: Stub ProxyInterface, CommitmentsInterface, and AuthorshipProvider

- Where: `crates/myosu-chain/pallets/game-solver/src/stubs.rs (new)`
- How: Create no-op implementations for the two interface traits that the
  pallet Config requires from stripped pallets:

  `ProxyInterface<AccountId>`: 2 methods (`add_lease_beneficiary_proxy`,
  `remove_lease_beneficiary_proxy`). **Already has a `()` no-op impl in
  subtensor.** Only called from `subnets/leasing.rs` which is stripped in CF-07.

  `CommitmentsInterface`: 1 method (`purge_netuid`). Needs a 5-line `()` impl.
  Only called from `coinbase/root.rs:219` during subnet dissolution.

  `GetCommitments<AccountId>`: 1 method from `pallet_commitments`. Used in
  `rpc_info/metagraph.rs:1536`. The existing mock already uses `()` for this.

  `AuthorshipProvider`: Config requires `type AuthorshipProvider: AuthorshipInfo<AccountId>`.
  Provides `fn author() -> Option<AccountId>` for block builder fee distribution.
  Implement by reading from Aura's inherent data or from `frame_system::Pallet::block_author()`.
  Used in coinbase for distributing fees to block authors.

  ProxyInterface, CommitmentsInterface, and GetCommitments are used in peripheral
  paths only. AuthorshipProvider is used in emission but can return a fixed
  account (Alice) in devnet mode. Game-solving critical paths (registration,
  weights, epoch) do not depend on any of these being real implementations.

- Whole-system effect: allows Config to compile without the proxy and
  commitments pallets.
- State: no state — stubs are stateless.
- Wiring contract:
  - Trigger: Config type resolution
  - Callsite: runtime Config impl for pallet_game_solver
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: `cargo check` passes
- Required tests:
  - `cargo check -p pallet-game-solver`
- Pass/fail:
  - Config compiles with stub ProxyInterface
  - Config compiles with stub CommitmentsInterface
  - No runtime behavior change for game-solving paths
- Blocking note: without these stubs, the Config trait has unresolvable
  associated types.
- Rollback condition: ProxyInterface or CommitmentsInterface methods are called
  in critical registration or weight submission paths.

---

## Operational Controls

Phase order:
1. AC-CF-07 (strip supertraits) — FIRST COMMIT, enables cargo check
2. AC-CF-06 (swap stub) + CF-08 (extrinsic types) + CF-09 (CRV3 strip) + CF-10 (primitives) + CF-11 (interface stubs) — parallel prerequisites
3. AC-CF-02 (dependencies) — clean workspace compiles
4. AC-CF-01 (runtime stripping) — runtime WASM builds
5. AC-CF-04 (chain spec) — genesis is valid
6. AC-CF-03 (node binary) — node starts and produces blocks
7. AC-CF-05 (smoke test) — end-to-end verification

Gate rules:
- CF-07 must land before ANY other CF-* AC can compile
- CF-06, CF-08, CF-09, CF-10, CF-11 can proceed in parallel after CF-07
- CF-02 must resolve before CF-01 can compile
- CF-01 must produce WASM before CF-03 can link it
- CF-04 must be valid before CF-03 can start the node
- CF-05 requires CF-01 through CF-04 to all pass

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| Dependency resolution | Transitive dep on stripped crate | Yes | Yes | Build error |
| Runtime compilation | Missing type from stripped pallet | Yes | Yes | Build error |
| WASM building | substrate-wasm-builder misconfiguration | Yes | Yes | Build error |
| Node startup | Genesis state incompatible with runtime | Yes | Yes | Node panic |
| Block production | Aura authority key mismatch | Yes | Yes | No blocks produced |
| RPC server | Port conflict | No (dev mode) | No | Different port |
| Balance transfer | Insufficient balance or bad nonce | Yes | N/A | Transaction reverted |

Diagram maintenance:
- `README.md` — update architecture diagram when runtime composition changes

## Decision Log

- 2026-03-16: Keep SafeMode pallet (index 20) — useful emergency pause for
  devnet debugging, near-zero maintenance cost.
- 2026-03-16: Single chain_spec.rs file instead of subtensor's multi-file
  split — we only need devnet and local, not finney/testnet/devnet.
- 2026-03-16: Reserve pallet index 7 for game-solver — matches subtensor's
  index for its main pallet, keeping mental model consistent.
- 2026-03-16: Keep Polkadot SDK at subtensor's fork rev rather than upgrading
  — reduces variables during initial fork. Upgrade is a separate spec.
- 2026-03-16: Use Aura (not BABE) for devnet — simpler, instant seal in
  --dev mode. Production can switch to BABE later.
- 2026-03-17: CF-07 must be FIRST COMMIT — pallet_subtensor::Config requires
  pallet_drand::Config + pallet_crowdloan::Config as supertraits. Nothing
  compiles until these are removed. Discovered in pre-implementation audit.
- 2026-03-17: SwapInterface no-op stub (CF-06) instead of porting AMM —
  swap pallet is 800+ lines and deeply coupled to dual-token model. Identity
  swap is sufficient for single-token myosu.
- 2026-03-17: Strip CRV3 timelock, keep commit-reveal v2 only — CRV3 depends
  on pallet_drand::Pulses which is stripped. Hash-based v2 is sufficient.
- 2026-03-17: Single-token model (MYOSU only, no Alpha/TAO) — AMM pools add
  ~30 storage items and 800+ lines for zero Stage 0 value.
- 2026-03-17: 13 pallets after CF-01, not 14 — index 7 is reserved but empty
  until GS-09 adds the game-solver pallet.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `cargo check -p pallet-game-solver` after supertrait strip | Config compiles standalone | CF-07 |
| 2 | SwapInterface stub identity swap test | Swap stub works | CF-06 |
| 3 | `cargo build -p myosu-runtime` succeeds | Runtime stripping | CF-01, CF-02, CF-08 |
| 4 | `cargo tree -p myosu-runtime` has no subtensor/frontier crates | Dependency pruning | CF-02 |
| 5 | safe-math and share-pool tests pass | Primitives ported | CF-10 |
| 6 | commit-reveal v2 flow works without drand | CRV3 stripped | CF-09 |
| 7 | `myosu-node --dev` starts and produces blocks | Node binary + chain spec | CF-03, CF-04 |
| 8 | `system_health` RPC responds on port 9944 | RPC integration | CF-03 |
| 9 | Alice transfers 100 MYOSU to Bob via RPC | Transaction execution | CF-05 |
| 10 | Full smoke test passes autonomously | End-to-end | All |
