# Specification: Chain Fork Scaffold — Minimal Substrate Chain from Subtensor

Source: Master spec AC-CH-01, subtensor runtime analysis at coding/subtensor
Status: Draft
Date: 2026-03-16
Depends-on: none

## Purpose

Fork Bittensor's subtensor Substrate runtime into a minimal, compilable chain
that produces blocks on a local devnet. This spec strips all AI/ML-specific
pallets (weights, epochs, EVM, AMM, randomness beacon) while preserving the
core Substrate infrastructure (consensus, accounts, balances, transaction
payment). The result is a blank-slate chain ready to receive the game-solving
pallet (spec: `031626-game-solving-pallet.md`).

The primary consumer is every subsequent spec — nothing works without a
running chain. The secondary consumer is the developer, who needs a
`myosu-node --dev` command that starts a local devnet in under 10 seconds.

**Key design constraint**: the fork must compile and produce blocks before any
game-specific code is added. This separates "does Substrate work?" from "does
our game logic work?" and prevents debugging two things at once.

## Whole-System Goal

Current state:
- `coding/subtensor` contains a full Bittensor node with 30 pallets, EVM,
  AMM, AI incentives, and 3.7MB of pallet_subtensor code
- The runtime compiles to WASM and produces blocks via BABE+GRANDPA
- No myosu-specific code exists

This spec adds:
- A stripped Substrate runtime with ~12 pallets (down from 30)
- A node binary that produces blocks on local devnet
- A local chain spec with test accounts
- Workspace structure under `crates/myosu-chain/`

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
- Game-solving pallet — that's a separate spec (031626-game-solving-pallet.md)
- Custom transaction extensions — keep standard Substrate extensions
- EVM or smart contract support — stripped and not coming back
- Production chain spec — devnet only
- Benchmarks — will be added when game pallet lands

## Current State

- `/home/r/coding/subtensor/runtime/src/lib.rs` — 2,679 lines, 30 pallets in
  `construct_runtime!`, spec_version 385
- `/home/r/coding/subtensor/node/` — full node binary with Frontier EVM,
  MEV shield, drand integration
- `/home/r/coding/subtensor/pallets/subtensor/` — 3.7MB, ~150 storage items,
  the entire AI incentive layer
- `/home/r/coding/subtensor/node/src/chain_spec/localnet.rs` — local devnet
  with Alice/Bob test accounts and Aura/Grandpa authorities

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Substrate runtime shell | `subtensor/runtime/src/lib.rs` | extend (strip) | Remove 18 pallets, keep 12 |
| Node binary | `subtensor/node/src/main.rs` | extend (strip) | Remove Frontier, drand, shield service code |
| Local chain spec | `subtensor/node/src/chain_spec/localnet.rs` | extend | Strip subtensor genesis, keep balances/aura/grandpa |
| BABE+GRANDPA consensus | `subtensor/node/src/consensus/` | reuse | Standard Substrate consensus, no changes needed |
| Common types | `subtensor/common/src/lib.rs` | reuse | AccountId, Balance, BlockNumber definitions |
| RPC configuration | `subtensor/node/src/rpc.rs` | extend | Remove Frontier RPC, keep standard Substrate RPC |
| Build system | `subtensor/runtime/build.rs` | extend | Change metadata token name from "TAO" to "MYOSU" |

## Non-goals

- Adding any game-specific logic — this spec produces a blank chain
- Custom consensus parameters — standard Substrate defaults are fine for devnet
- Preserving subtensor's migration system — fresh chain, no migration needed
- Supporting the subtensor pallet's 150+ storage items — all stripped
- EVM compatibility — permanently removed, not deferred

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Runtime composition | New (from subtensor) | crates/myosu-chain/runtime/src/lib.rs |
| Node binary | New (from subtensor) | crates/myosu-chain/node/src/main.rs |
| Node service | New (from subtensor) | crates/myosu-chain/node/src/service.rs |
| Chain spec (local) | New (from subtensor) | crates/myosu-chain/node/src/chain_spec.rs |
| Common types | New (from subtensor) | crates/myosu-chain/common/src/lib.rs |
| Runtime build | New (from subtensor) | crates/myosu-chain/runtime/build.rs |

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

  **Keep (indices 0-6, 11-16, 20) — 14 pallets total:**
  System(0), RandomnessFlip(1), Timestamp(2), Aura(3), Grandpa(4),
  Balances(5), TransactionPayment(6), Utility(11), Sudo(12), Multisig(13),
  Preimage(14), Scheduler(15), Proxy(16), SafeMode(20).

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
  - `construct_runtime!` contains exactly 14 pallets (System through SafeMode)
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

## Operational Controls

Phase order:
1. AC-CF-02 (dependencies) — clean workspace compiles
2. AC-CF-01 (runtime stripping) — runtime WASM builds
3. AC-CF-04 (chain spec) — genesis is valid
4. AC-CF-03 (node binary) — node starts and produces blocks
5. AC-CF-05 (smoke test) — end-to-end verification

Gate rules:
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

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `cargo build -p myosu-runtime` succeeds | Runtime stripping | CF-01, CF-02 |
| 2 | `cargo tree -p myosu-runtime` has no subtensor/frontier crates | Dependency pruning | CF-02 |
| 3 | `myosu-node --dev` starts and produces blocks | Node binary + chain spec | CF-03, CF-04 |
| 4 | `system_health` RPC responds on port 9944 | RPC integration | CF-03 |
| 5 | Alice transfers 100 MYOSU to Bob via RPC | Transaction execution | CF-05 |
| 6 | Full smoke test passes autonomously | End-to-end | All |
