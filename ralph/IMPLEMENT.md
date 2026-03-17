# Implementation Plan

Status: Active
Date: 2026-03-16

## Spec Index

| Spec File | AC Prefix | Count |
|-----------|-----------|-------|
| `031626-00-master-index.md` | — | index |
| (robopoker fork prerequisites) | RF-01..04 | 4 |
| `031626-01-chain-fork-scaffold.md` | CF-01..11 | 11 |
| `031626-02a-game-engine-traits.md` | GT-01..05 | 5 |
| `031626-02b-poker-engine.md` | PE-01..04 | 4 |
| `031626-03-game-solving-pallet.md` | GS-01..10 | 10 |
| (shared chain client) | CC-01 | 1 |
| `031626-04a-miner-binary.md` | MN-01..05 | 5 |
| `031626-04b-validator-oracle.md` | VO-01..07 | 7 |
| `031626-05-gameplay-cli.md` | GP-01..04 | 4 |
| `031626-06-multi-game-architecture.md` | MG-01..04 | 4 |
| `031626-07-tui-implementation.md` | TU-01..12 | 12 |
| `031626-08-abstraction-pipeline.md` | AP-01..03 | 3 |
| `031626-09-launch-integration.md` | LI-01..06 | 6 |
| `031626-10-agent-experience.md` | AX-01..06 | 6 |
| `031626-99-malinka-enhancements.md` | — | external |

---

## Stage 0: Robopoker Fork Prerequisites
Source: specs/031626-02a-game-engine-traits.md Blocking Prerequisites

- [ ] **RF-01** — Fork robopoker v1.0.0 and add serde feature
  - Where: `happybigmtn/robopoker (new fork)`
  - Tests: `cargo test --features serde` in forked repo
  - Blocking: GT-02, PE-01, PE-03 all require serializable NLHE types
  - Verify: NlheInfo, NlheEdge, NlheProfile, NlheEncoder, Path, Encounter all derive Serialize/Deserialize under `serde` feature; existing tests still pass
  - Integration: `Trigger=myosu-games depends on fork; Callsite=Cargo.toml git dep; State=types serializable; Persistence=N/A; Signal=cargo test --features serde passes`
  - Rollback: serde derives conflict with existing trait bounds

- [ ] **RF-02** — Add non-database NlheEncoder constructor
  - Where: `happybigmtn/robopoker crates/nlhe/src/encoder.rs (extend)`
  - Depends on: `RF-01`
  - Tests: `cargo test -p rbp-nlhe encoder::tests::from_map_constructor`
  - Blocking: PE-01 cannot create a functional solver without populated encoder
  - Verify: `NlheEncoder::from_map(BTreeMap<Isomorphism, Abstraction>)` works; `NlheEncoder::from_file(path)` loads binary abstraction table; existing DB path unaffected
  - Integration: `Trigger=miner creates encoder on startup; Callsite=PokerSolver::new(); State=encoder populated; Persistence=abstraction file on disk; Signal=abstraction() returns valid values`
  - Rollback: private field prevents non-DB construction without refactoring

- [ ] **RF-03** — Expose clustering APIs for standalone use
  - Where: `happybigmtn/robopoker crates/clustering/src/layer.rs (extend)`, `crates/clustering/src/lookup.rs (extend)`
  - Depends on: `RF-01`
  - Tests: `cargo test -p rbp-clustering clustering::tests::preflop_clusters_without_db`
  - Blocking: AP-01 needs clustering without PostgreSQL; Layer::cluster() currently takes &Client
  - Verify: File-based I/O alternatives for Lookup, Metric types; preflop clustering produces 169 entries from in-memory data; existing DB path unaffected
  - Integration: `Trigger=myosu-cluster binary; Callsite=clustering::layer::Layer::cluster_to_file(); State=clustering state; Persistence=binary abstraction files; Signal=output files with correct entry counts`
  - Rollback: clustering internals too coupled to PostgreSQL COPY protocol

- [ ] **RF-04** — File-based checkpoint save/load for NlheProfile
  - Where: `happybigmtn/robopoker crates/nlhe/src/profile.rs (extend)`
  - Depends on: `RF-01`
  - Tests: `cargo test -p rbp-nlhe profile::tests::checkpoint_roundtrip`
  - Blocking: MN-05 needs restart resilience; only PostgreSQL persistence exists
  - Verify: save_checkpoint(path) writes bincode; load_checkpoint(path) restores; round-trip preserves iterations + encounters; corrupted file → error not panic; Metrics field skipped (reconstructed on load)
  - Integration: `Trigger=miner shutdown or periodic save; Callsite=training.rs; State=NlheProfile serialized; Persistence=checkpoint file on disk; Signal=file size > 0, load succeeds`
  - Rollback: NlheProfile too large for bincode (>10GB serialized)

---

## Stage 1a: Chain Fork Prerequisites (must compile before CF-01..05)
Source spec: specs/031626-01-chain-fork-scaffold.md (extended)

Note: CF-07 is the FIRST commit — nothing compiles without it. CF-06, CF-08..11
are parallel prerequisites that must all land before CF-01 can strip pallets.

- [ ] **CF-07** — Strip drand/crowdloan Config Supertraits
  - Where: `crates/myosu-chain/pallets/game-solver/src/macros/config.rs (from subtensor)`, `src/coinbase/block_step.rs (from subtensor)`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: pallet_subtensor::Config requires pallet_drand::Config + pallet_crowdloan::Config — NOTHING compiles without this change. This is the FIRST COMMIT in the fork.
  - Verify: Config trait compiles without drand/crowdloan supertraits (config.rs:17); `impl<T: Config + pallet_drand::Config>` in block_step.rs:6 changed to just `impl<T: Config>`; all `pallet_drand::*` and `pallet_crowdloan::*` imports removed; leasing.rs stripped entirely; block_step no longer calls reveal_crv3_commits; RoundNumber type usage removed
  - Integration: `Trigger=cargo check; Callsite=config.rs:17 + block_step.rs:6; State=Config compiles; Persistence=N/A; Signal=cargo check exits 0`
  - Rollback: Config requires types from drand/crowdloan that are deeply woven into core logic

- [x] **CF-06** — SwapInterface No-Op Stub
  - Where: `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs (new)`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: SwapInterface is called in 37 production callsites across registration, staking, and emission. Config requires: `SwapHandler + SwapEngine<GetAlphaForTao<Self>> + SwapEngine<GetTaoForAlpha<Self>>`. All three trait bounds must be satisfied.
  - Verify: Identity stub implements ALL SwapHandler methods: swap() returns input=output, sim_swap() same, approx_fee_amount() returns zero, current_alpha_price() returns 1:1 (U96F32::from_num(1)), get_protocol_tao() returns ZERO, max_price()/min_price() return C::MAX/C::ZERO, adjust_protocol_liquidity() no-op, is_user_liquidity_enabled() returns false, dissolve_all_liquidity_providers()/clear_protocol_liquidity() return Ok(()), toggle_user_liquidity() no-op. DefaultPriceLimit::default_price_limit returns C::MAX. SwapEngine::swap() returns SwapResult with amount_in=amount_out, zero fees. ~80-100 lines total.
  - Integration: `Trigger=register_neuron calls burn; Callsite=registration.rs swap call; State=tokens burned directly (no AMM); Persistence=balance deducted; Signal=registration succeeds`
  - Rollback: SwapInterface trait has methods beyond those enumerated here

- [ ] **CF-08** — Replace fp_self_contained Extrinsic Types and Custom Fee Handler
  - Where: `crates/myosu-chain/runtime/src/lib.rs (from subtensor)`
  - Tests: `cargo check -p myosu-runtime`
  - Blocking: UncheckedExtrinsic and CheckedExtrinsic use Frontier's fp_self_contained types — runtime won't compile after removing Frontier deps. Also: subtensor's custom transaction fee handler (pallets/transaction-fee/) hard-depends on pallet_subtensor_swap::Config even though Alpha fees are disabled.
  - Verify: UncheckedExtrinsic uses standard generic::UncheckedExtrinsic; CheckedExtrinsic uses standard generic::CheckedExtrinsic; SignedPayload uses standard sp_runtime::generic::SignedPayload; DrandPriority removed from TransactionExtensions tuple; SubtensorTransactionExtension removed; custom fee handler replaced with standard pallet_transaction_payment::FungibleAdapter (removes pallet_subtensor_swap compile dependency); delete fp_self_contained::SelfContainedCall impl (~60 lines)
  - Integration: `Trigger=cargo build -p myosu-runtime; Callsite=runtime/src/lib.rs type aliases; State=extrinsic types correct; Persistence=N/A; Signal=runtime compiles`
  - Rollback: other runtime code depends on fp_self_contained methods or custom fee logic

- [ ] **CF-09** — Strip CRV3 Timelock Commit-Reveal Path
  - Where: `crates/myosu-chain/pallets/game-solver/src/coinbase/ (from subtensor)`, `src/subnets/weights.rs (from subtensor)`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: CRV3 depends on pallet_drand::Pulses for timelock encryption — cannot function without drand
  - Verify: TimelockedWeightCommits storage removed; CRV3WeightCommits* storage removed; reveal_crv3_commits function removed; hash-based commit-reveal v2 (WeightCommits, commit_weights, reveal_weights) fully functional; DrandPriority transaction extension removed
  - Integration: `Trigger=validator calls commit_weights; Callsite=weights.rs; State=hash stored in WeightCommits; Persistence=on-chain; Signal=commit + reveal flow succeeds`
  - Rollback: v2 commit-reveal has a bug that CRV3 was fixing

- [ ] **CF-10** — Port Primitives and Runtime Common Types
  - Where: `crates/myosu-chain/primitives/safe-math/ (new, from subtensor)`, `crates/myosu-chain/primitives/share-pool/ (new, from subtensor)`, `crates/myosu-chain/common/ (new, from subtensor/common/)`
  - Tests: `cargo check -p myosu-safe-math && cargo check -p myosu-share-pool && cargo check -p myosu-runtime-common`
  - Blocking: Yuma epoch uses safe-math; ALL stake operations (20+ functions) use share-pool; NetUid/MechId/TaoCurrency/AlphaCurrency from runtime_common used in nearly every pallet file
  - Verify: safe-math + share-pool pass existing tests; runtime_common compiles with TaoCurrency=AlphaCurrency (single-token alias); NetUid, MechId, NetUidStorageIndex importable; no deps on stripped pallets
  - Integration: `Trigger=epoch.rs + staking.rs + lib.rs import these; Callsite=run_epoch, stake_utils, storage declarations; State=N/A (pure types + math); Persistence=N/A; Signal=all three crates' tests pass`
  - Rollback: runtime_common has deep coupling to stripped pallet types that can't be aliased

- [ ] **CF-11** — Stub ProxyInterface, CommitmentsInterface, AuthorshipProvider, and CheckColdkeySwap
  - Where: `crates/myosu-chain/pallets/game-solver/src/stubs.rs (new)`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: pallet Config requires ProxyInterface, CommitmentsInterface, GetCommitments, AuthorshipProvider, and frame_system DispatchGuard (CheckColdkeySwap depends on pallet_shield::Config)
  - Verify: No-op ProxyInterface (already has () impl in subtensor); no-op CommitmentsInterface (5 lines); no-op GetCommitments (return empty vec); AuthorshipProvider reads Aura block author or returns fixed account; replace CheckColdkeySwap DispatchGuard with no-op (always allow dispatch); Config compiles with all stubs
  - Integration: `Trigger=Config type resolution; Callsite=runtime Config impl; State=N/A; Persistence=N/A; Signal=cargo check passes`
  - Rollback: AuthorshipProvider needed for real emission distribution to block authors

---

## Stage 1b: Chain Fork Scaffold
Source spec: specs/031626-01-chain-fork-scaffold.md

- [ ] **CF-02** — Prune Workspace Dependencies
  - Where: `crates/myosu-chain/Cargo.toml (new)`, `crates/myosu-chain/runtime/Cargo.toml (new)`
  - Depends on: `CF-07`, `CF-06`, `CF-08`, `CF-09`, `CF-10`, `CF-11`
  - Tests: `! cargo tree -p myosu-runtime 2>&1 | grep -q 'pallet.subtensor'`
  - Blocking: Dependency contamination from stripped pallets will cause build failures
  - Verify: No references to subtensor/frontier/drand in dependency tree; build succeeds; WASM blob produced
  - Integration: `Trigger=cargo build; Callsite=Cargo resolver; State=dependency tree resolved; Persistence=Cargo.lock; Signal=cargo tree shows no stripped crates`
  - Rollback: Transitive dependency on stripped pallet that can't be resolved

- [ ] **CF-01** — Strip AI/EVM Pallets from Runtime
  - Where: `crates/myosu-chain/runtime/src/lib.rs (new, from subtensor)`
  - Depends on: `CF-02`
  - Tests: `cargo check -p myosu-runtime`
  - Blocking: Everything downstream depends on a compilable runtime — single most critical unblock
  - Verify: Runtime compiles; spec_name="myosu"; 13 pallets in construct_runtime! (index 7 reserved for game-solver, SafeMode at index 20); no subtensor/frontier/EVM references; WASM < 5MB
  - Integration: `Trigger=cargo build -p myosu-runtime; Callsite=runtime/build.rs WASM builder; State=WASM blob compiled; Persistence=target/ artifact; Signal=build exits 0`
  - Rollback: Runtime fails to compile after stripping — hidden inter-pallet dependencies

- [x] **CF-04** — Local Devnet Chain Spec
  - Where: `crates/myosu-chain/node/src/chain_spec.rs (new, from subtensor)`
  - Depends on: `CF-01`
  - Tests: `cargo check -p myosu-node`
  - Blocking: Chain spec defines genesis state — node can't start without it
  - Verify: Dev and local specs produce valid genesis; Alice/Bob/Charlie/Dave/Eve/Ferdie funded with 1M MYOSU each; Alice is sudo; token symbol MYOSU, 9 decimals; no subtensor/EVM genesis. Pallet GenesisConfig must initialize: NetworksAdded(false initially — GS-09 adds genesis subnet), TotalIssuance(sum of balances), BlockEmission(1_000_000_000 RAO = 1 MYOSU), EmissionSplit {miner: 61, validator: 21, owner: 18}. Genesis subnet (added by GS-09 later): netuid=1, game_type=b"nlhe_hu", tempo=180, max_uids=256, max_validators=64, owner=Alice. CRITICAL: genesis must also set FirstEmissionBlockNumber(netuid, 1) and SubtokenEnabled(netuid, true) for the game subnet — without these, emission never flows (silent failure). Set SubnetMechanism(netuid, 0) for Stable (1:1 identity swap, single-token model). Override on_runtime_upgrade to no-op (skip all 42 subtensor migrations on fresh chain).
  - Integration: `Trigger=node startup reads chain spec; Callsite=node/src/command.rs; State=genesis block initialized; Persistence=genesis block in DB; Signal=build-spec outputs valid JSON`
  - Rollback: Genesis requires types from stripped pallets

- [ ] **CF-03** — Minimal Node Service
  - Where: `crates/myosu-chain/node/src/service.rs (new, from subtensor)`
  - Depends on: `CF-01`
  - Tests: `cargo check -p myosu-node`
  - Blocking: Node binary is the entry point — every test requires running it
  - Verify: Node binary compiles; `--dev` starts and produces blocks; `--help` shows subcommands; `build-spec` outputs JSON; clean shutdown on SIGINT; no panics
  - Integration: `Trigger=myosu-node --dev CLI; Callsite=node/src/main.rs; State=node service running, blocks produced; Persistence=block database; Signal=block numbers incrementing in logs`
  - Rollback: service.rs has deep coupling to Frontier/drand that can't be cleanly removed

- [ ] **CF-05** — End-to-End Devnet Smoke Test
  - Where: `crates/myosu-chain/tests/integration.rs (new)`
  - Depends on: `CF-03`, `CF-04`
  - Tests: `cargo check -p myosu-chain`
  - Blocking: Gate proving CH-01 is complete — unit tests alone are not sufficient
  - Verify: Node starts and produces block 1 within 15s; system_health RPC responds; Alice has 1M MYOSU; balance transfer Alice→Bob succeeds; node shuts down cleanly; no panics in logs
  - Integration: `Trigger=cargo test; Callsite=tests/integration.rs; State=devnet lifecycle; Persistence=temp block DB (cleaned up); Signal=test passes, block > 0, transfer confirmed`
  - Rollback: Node fails to start, RPC unresponsive, or transactions fail

---

## Stage 2a: Game Engine Traits
Source spec: specs/031626-02a-game-engine-traits.md

- [ ] **GT-01** — Re-export and Extend Robopoker CFR Traits
  - Where: `crates/myosu-games/src/traits.rs (new)`
  - Tests: `cargo test -p myosu-games`
  - Blocking: Every other AC and downstream spec depends on these types
  - Verify: CfrGame, Profile, Encoder importable; GameConfig serializes; StrategyQuery/Response round-trips
  - Integration: `Trigger=compile-time; Callsite=all downstream crates; State=N/A; Persistence=N/A; Signal=cargo test passes`
  - Rollback: rbp-mccfr has incompatible dependency requirements

- [ ] **GT-02** — Wire Serialization for Strategy Transport
  - Where: `crates/myosu-games/src/wire.rs (new)`
  - Depends on: `GT-01`, `RF-01`
  - Tests: `cargo test -p myosu-games`
  - Blocking: Miners and validators must agree on serialization format
  - Verify: WireStrategy serializes to JSON and round-trips; action probabilities preserved; invalid bytes → error not panic
  - Integration: `Trigger=miner serializes, validator deserializes; Callsite=axon handler; State=N/A; Persistence=N/A; Signal=round-trip test passes`
  - Rollback: robopoker Edge/Info types don't implement Serialize

- [ ] **GT-03** — Runtime Game Selection
  - Where: `crates/myosu-games/src/registry.rs (new)`
  - Depends on: `GT-01`
  - Tests: `cargo test -p myosu-games`
  - Blocking: Miners need to select correct solver for their subnet
  - Verify: from_bytes maps known types; unknown → Custom; roundtrip bytes; num_players correct
  - Integration: `Trigger=miner/validator reads subnet game_type; Callsite=main.rs; State=correct engine selected; Persistence=N/A; Signal=from_bytes returns correct variant`
  - Rollback: game_type encoding on-chain doesn't match registry

- [ ] **GT-04** — Remote Strategy Profile Adapter
  - Where: `crates/myosu-games/src/remote_profile.rs (new)`
  - Depends on: `GT-01`
  - Tests: `cargo test -p myosu-games`
  - Blocking: Validators need to compute exploitability from miner query responses without the full Profile object
  - Verify: RemoteProfile from RPS Nash distributions → exploit ≈ 0; from always-rock → exploit > 0; matches local Profile within 1%; missing info set → uniform fallback
  - Integration: `Trigger=validator builds RemoteProfile from responses; Callsite=myosu-validator scoring.rs; State=HashMap of info→distributions; Persistence=N/A; Signal=profile.exploitability(tree) returns valid f64`
  - Rollback: Profile trait requires state beyond cum_weight for exploitability path

- [ ] **GT-05** — RPS Reference Implementation Test Suite
  - Where: `crates/myosu-games/tests/rps_integration.rs (new)`
  - Depends on: `GT-03`, `GT-04`
  - Tests: `cargo test -p myosu-games`
  - Blocking: If RPS doesn't work, nothing will — validates entire trait system
  - Verify: 1000 iterations → each action ~1/3; exploitability < 0.01; wire roundtrip preserves data; always-rock is exploitable
  - Integration: `Trigger=cargo test; Callsite=test suite; State=N/A; Persistence=N/A; Signal=all RPS tests pass`
  - Rollback: robopoker RPS module incompatible with wire layer

---

## Stage 2b: Poker Engine
Source spec: specs/031626-02b-poker-engine.md

- [ ] **PE-01** — Poker Solver Wrapper
  - Where: `crates/myosu-games-poker/src/solver.rs (new)`
  - Depends on: `GT-01`, `RF-01`, `RF-02`
  - Tests: `cargo check -p myosu-games-poker`
  - Blocking: Core of what miners do — no solver means no strategies
  - Verify: Empty solver has 0 epochs; 100 iterations → epochs=100; strategy sums to ~1.0; checkpoint roundtrips; exploitability decreases over training; snapshot_profile() returns cheaply cloneable Arc<NlheProfile> for ArcSwap publishing (required by MN-02)
  - Integration: `Trigger=miner creates solver; Callsite=myosu-miner/main.rs; State=training state in memory; Persistence=checkpoint files; Signal=epochs() increases`
  - Rollback: robopoker Flagship type aliases not public or NlheProfile not serializable

- [ ] **PE-02** — Strategy Query Handler
  - Where: `crates/myosu-games-poker/src/query.rs (new)`
  - Depends on: `PE-01`
  - Tests: `cargo check -p myosu-games-poker`
  - Blocking: Miner axon needs a request handler
  - Verify: Valid query → response with distribution; invalid bytes → error; probabilities sum to 1.0
  - Integration: `Trigger=HTTP request to axon; Callsite=axon handler; State=N/A (read-only); Persistence=N/A; Signal=valid WireStrategy response`
  - Rollback: NlheInfo serialization incompatible with validator queries

- [ ] **PE-03** — Poker Wire Serialization
  - Where: `crates/myosu-games-poker/src/wire.rs (new)`
  - Depends on: `PE-01`, `RF-01`
  - Tests: `cargo check -p myosu-games-poker`
  - Blocking: Without serialization, miners and validators can't communicate
  - Verify: NlheInfo roundtrips; all NlheEdge variants roundtrip; size < 1KB; corrupted bytes → error
  - Integration: `Trigger=miner serializes, validator deserializes; Callsite=query handler + oracle; State=N/A; Persistence=N/A; Signal=roundtrip tests pass`
  - Rollback: robopoker types don't support serde with feature flags

- [ ] **PE-04** — Poker Exploitability Integration
  - Where: `crates/myosu-games-poker/src/exploit.rs (new)`
  - Depends on: `PE-01`
  - Tests: `cargo check -p myosu-games-poker`
  - Blocking: Validator scoring function for poker miners
  - Verify: Trained strategy < 500 mbb/h; random > 200 mbb/h; remote matches local within 5%; always non-negative
  - Integration: `Trigger=validator evaluation loop; Callsite=myosu-validator oracle; State=N/A; Persistence=N/A; Signal=returns f64 exploitability`
  - Rollback: exploitability > 60s for HU NLHE

---

## Stage 3: On-Chain Incentives
Source spec: specs/031626-03-game-solving-pallet.md

- [ ] **GS-01** — Pallet Scaffold with Config, Storage, Mock Runtime, Errors, and Events
  - Where: `crates/myosu-chain/pallets/game-solver/src/lib.rs (new)`, `src/tests/mock.rs (new)`
  - Depends on: `CF-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Every other GS-* AC reads or writes these storage items; mock runtime required for ALL pallet unit tests
  - Verify: Pallet compiles; ~30 storage items with correct defaults; Config trait has Currency + ~20 constants (InitialTempo, InitialKappa, InitialBondsMovingAverage, InitialBondsPenalty, InitialImmunityPeriod, InitialActivityCutoff, InitialMaxAllowedValidators, InitialMinAllowedWeights, etc.); mock runtime includes System+Balances+Timestamp+Aura+Scheduler+Preimage+GameSolver; Config stubs: SwapInterface=NoOpSwap, ProxyInterface=(), CommitmentsInterface=(), GetCommitments=(), AuthorshipProvider=MockAuthor; AxonInfo encodes/decodes; test helpers: new_test_ext(), add_network(), register_ok_neuron(), step_block(), step_epochs(); errors enumerated (~25: SubnetNotExists, HotKeyAlreadyRegistered, TooManyRegistrations, NotEnoughStake, WeightVecNotEqualSize, DuplicateUids, etc.); events enumerated (~10: NetworkAdded, NeuronRegistered, WeightsSet, StakeAdded, StakeRemoved, WeightsCommitted, WeightsRevealed, EpochCompleted, EmissionDistributed, AxonServed)
  - Integration: `Trigger=construct_runtime!; Callsite=runtime/src/lib.rs; State=storage items available; Persistence=on-chain storage; Signal=cargo build succeeds`
  - Rollback: Storage layout incompatible with Yuma's access patterns

- [ ] **GS-02** — Subnet Registry
  - Where: `crates/myosu-chain/pallets/game-solver/src/subnets.rs (new)`
  - Depends on: `GS-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Neurons can only register on existing subnets — unblocks GS-03
  - Verify: create_subnet burns tokens, assigns id, emits event; dissolve clears state; max subnet limit enforced; set_hyperparams sudo-only
  - Integration: `Trigger=create_subnet extrinsic; Callsite=pallet dispatchable; State=subnet registered with game_type; Persistence=10 storage items per subnet; Signal=SubnetCreated event`
  - Rollback: game_type storage can't represent needed metadata

- [ ] **GS-03** — Neuron Registration and Pruning
  - Where: `crates/myosu-chain/pallets/game-solver/src/registration.rs (new)`
  - Depends on: `GS-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Without neurons, nothing to weight or reward — unblocks GS-04, GS-07
  - Verify: Sequential UIDs assigned; burn cost enforced; full subnet prunes weakest; immunity period respected; duplicate rejected; storage vectors extended
  - Integration: `Trigger=register_neuron extrinsic; Callsite=pallet dispatchable; State=UID assigned, vectors extended; Persistence=Keys, Uids, IsNetworkMember; Signal=NeuronRegistered event`
  - Rollback: Pruning logic incorrectly removes high-scoring neurons

- [ ] **GS-04** — Weight Submission
  - Where: `crates/myosu-chain/pallets/game-solver/src/weights.rs (new)`
  - Depends on: `GS-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Weights feed Yuma Consensus — bridge between off-chain evaluation and on-chain incentives
  - Verify: set_weights stores and validates; commit_weights stores hash; reveal_weights verifies and stores; rate limiting enforced; validator permit required
  - Integration: `Trigger=set_weights/commit_weights extrinsic; Callsite=pallet dispatchable; State=weight matrix updated; Persistence=Weights storage; Signal=WeightsSet event`
  - Rollback: Weight validation rejects valid game-solving patterns

- [ ] **GS-05** — Yuma Consensus Port
  - Where: `crates/myosu-chain/pallets/game-solver/src/epoch.rs (new)`, `src/math.rs (new)`
  - Depends on: `GS-04`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Core algorithm — getting it wrong breaks the entire incentive mechanism. Port ~3200 lines from subtensor epoch/run_epoch.rs + math.rs. Use plain NetUid as storage keys (not NetUidStorageIndex) since single-mechanism. Collapse epoch_with_mechanisms() to directly call epoch_mechanism() for MechId::MAIN only. Must simplify get_stake_weights_for_network() to read from share-pool without the TAO/Alpha dual-token weighting (single token = alpha_stake only, no tao_weight multiplication).
  - Verify: Consensus clips above median (kappa=0.5); bonds accumulate via EMA (alpha from BondsMovingAverage); bond penalty (beta=0.1) penalizes out-of-consensus validators; INV-003: bit-identical output to subtensor for same inputs on matching test vectors; zero weights → zero emission; Yuma v3 sigmoid per-bond EMA works
  - Integration: `Trigger=on_initialize at tempo boundary; Callsite=lib.rs::Hooks::on_initialize(); State=scores computed, bonds updated; Persistence=Incentive, Dividends, Bonds, Consensus, ValidatorTrust, ValidatorPermit, Emission, StakeWeight updated; Signal=EpochCompleted event`
  - Rollback: Fixed-point math diverges from subtensor due to dependency mismatch

- [ ] **GS-06** — Emission Distribution (rewrite, not port)
  - Where: `crates/myosu-chain/pallets/game-solver/src/emission.rs (new)`, `src/block_step.rs (new)`
  - Depends on: `GS-05`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Emission is the revenue model — miners won't run solvers without economic reward. NOTE: subtensor's run_coinbase (957 lines) assumes root network + AMM + multi-subnet; 80% is unnecessary. Write clean ~100 line emission from scratch. The on_initialize call chain must be: `on_initialize(n) → block_step() → { adjust_registration_terms(), get_block_emission(), accumulate_pending(), drain_pending() → { should_run_epoch()=true → run_epoch() → distribute_emission(61/21/18) } }`. Strip: CRV3 reveals, moving prices, root proportion, childkey scheduling, auto-claim root divs, hotkey swap cleanup. Keep: adjust_registration_terms, should_run_epoch (formula: `(block + netuid + 1) % (tempo + 1) == 0`), blocks_until_next_epoch.
  - Verify: 61% to miners proportional to Yuma incentive scores; 21% to validators proportional to bond×rank; 18% to subnet owner; no emission without weights; TotalIssuance tracks correctly; no tokens created from thin air; sum(all emissions) == block_emission per epoch; on_initialize weight hardcoded (acceptable for testnet, benchmark before mainnet)
  - Integration: `Trigger=on_initialize at tempo boundary; Callsite=lib.rs::Hooks::on_initialize(); State=pending emission accumulated per block, distributed on epoch; Persistence=balance updates + TotalIssuance; Signal=EmissionDistributed event`
  - Rollback: Emission accounting error creates or destroys tokens

- [ ] **GS-07** — Axon Serving
  - Where: `crates/myosu-chain/pallets/game-solver/src/serving.rs (new)`
  - Depends on: `GS-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Without axon discovery, validators can't find miners
  - Verify: Registered neuron serves axon; unregistered rejected; IP/port validated; rate limited; queryable via RPC
  - Integration: `Trigger=serve_axon extrinsic; Callsite=pallet dispatchable; State=AxonInfo stored; Persistence=Axons storage; Signal=AxonServed event`
  - Rollback: AxonInfo encoding incompatible with miner endpoint format

- [ ] **GS-08** — Basic Staking
  - Where: `crates/myosu-chain/pallets/game-solver/src/staking.rs (new)`
  - Depends on: `GS-01`
  - Tests: `cargo check -p pallet-game-solver`
  - Blocking: Without staking, all validators have equal power — no skin-in-the-game
  - Verify: add_stake transfers tokens; remove_stake returns tokens; stake determines validator power; insufficient balance/stake rejected
  - Integration: `Trigger=add_stake/remove_stake extrinsic; Callsite=pallet dispatchable; State=Stake updated; Persistence=Stake storage + balance reserve; Signal=StakeAdded/StakeRemoved event`
  - Rollback: Stake accounting error creates or loses tokens

- [ ] **GS-09** — Add Pallet to Runtime at Index 7
  - Where: `crates/myosu-chain/runtime/src/lib.rs (extend)`, `node/src/chain_spec.rs (extend)`
  - Depends on: `GS-02`, `GS-03`, `GS-04`, `GS-05`, `GS-06`, `GS-07`, `GS-08`
  - Tests: `cargo check -p myosu-chain`
  - Blocking: Integration gate — until pallet is in runtime, it's just library code
  - Verify: Runtime compiles with pallet; index 7 occupied; create_subnet callable; full loop works (create→register→stake→weights→epoch→emission); no CF-05 regression; dev chain spec includes genesis subnet 1 (nlhe_hu, owned by Alice)
  - Integration: `Trigger=runtime compilation; Callsite=runtime/src/lib.rs; State=pallet in block execution; Persistence=pallet storage in chain state; Signal=create_subnet callable via RPC`
  - Rollback: Config requirements conflict with existing runtime

- [ ] **GS-10** — Runtime API for State Queries
  - Where: `crates/myosu-chain/pallets/game-solver/src/rpc.rs (new)`, `runtime/src/lib.rs (extend)`
  - Depends on: `GS-09`
  - Tests: `cargo check -p myosu-runtime`
  - Blocking: Without efficient queries, off-chain participants have no practical way to discover chain state
  - Verify: subnet_info(1) returns SubnetInfo; all_axons(1) returns miner endpoints; all_incentives(1) returns scores after epoch; nonexistent subnet → None
  - Integration: `Trigger=RPC call; Callsite=runtime API impl; State=N/A (read-only); Persistence=N/A; Signal=all_axons returns data`
  - Rollback: runtime API types incompatible with subxt codegen

---

## Stage 4a: Shared Chain Client + Miner Binary
Source spec: specs/031626-04a-miner-binary.md

- [ ] **CC-01** — Shared Chain Client Crate
  - Where: `crates/myosu-chain-client/src/lib.rs (new)`
  - Depends on: `CF-03`
  - Tests: `cargo check -p myosu-chain-client`
  - Blocking: MN-01, VO-01, GP-01 all need chain RPC — DRY violation without shared crate
  - Verify: Connects to devnet via WebSocket; submits extrinsics (register_neuron, serve_axon, set_weights, add_stake); queries storage (Axons, Incentive, SubnetInfo via runtime API); account/keypair management from seed string
  - Integration: `Trigger=any binary startup; Callsite=myosu-miner, myosu-validator, myosu-play; State=RPC connection; Persistence=N/A; Signal=chain queries return data`
  - Rollback: subxt codegen incompatible with myosu runtime metadata

- [ ] **MN-01** — CLI and Chain Registration
  - Where: `crates/myosu-miner/src/main.rs (new)`
  - Depends on: `CC-01`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: Registration is the first thing a miner does
  - Verify: Connects to devnet; registers and receives UID; already-registered skips; invalid key → error
  - Integration: `Trigger=myosu-miner CLI; Callsite=main.rs; State=neuron registered; Persistence=UID in local state; Signal=log "Registered as UID {n}"`
  - Rollback: subxt/RPC client incompatible with chain runtime

- [ ] **MN-02** — Background Training Loop
  - Where: `crates/myosu-miner/src/training.rs (new)`
  - Depends on: `MN-01`, `PE-01`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: Without training, miner serves random strategies
  - Verify: Runs 100 iterations without panic; checkpoint written; solver accessible during training (no deadlock); exploitability logged
  - Integration: `Trigger=after registration; Callsite=main.rs spawns task; State=solver improves; Persistence=checkpoints; Signal=log "Epoch {n}, exploit: {x}"`
  - Rollback: ArcSwap publish contention or snapshot clone too slow for query latency < 2s

- [ ] **MN-03** — HTTP Axon Server
  - Where: `crates/myosu-miner/src/axon.rs (new)`
  - Depends on: `MN-01`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: Validators query this endpoint — no axon = invisible miner
  - Verify: GET /health returns 200; POST /strategy with valid query → response; invalid bytes → 400; handles 100 concurrent requests
  - Integration: `Trigger=after registration; Callsite=main.rs spawns server; State=HTTP listening; Persistence=N/A; Signal=curl /health responds`
  - Rollback: HTTP framework conflicts with tokio training runtime

- [ ] **MN-04** — On-Chain Axon Advertisement
  - Where: `crates/myosu-miner/src/chain.rs (extend)`
  - Depends on: `MN-01`, `MN-03`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: Without advertisement, validators don't know where to query
  - Verify: serve_axon extrinsic succeeds; axon discoverable via RPC query
  - Integration: `Trigger=axon starts listening; Callsite=main.rs; State=AxonInfo on chain; Persistence=on-chain storage; Signal=AxonServed event`
  - Rollback: IP detection fails in container environments

- [ ] **MN-05** — Graceful Shutdown and Resume
  - Where: `crates/myosu-miner/src/main.rs (extend)`
  - Depends on: `MN-02`, `RF-04`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: Production miners need restart resilience
  - Verify: SIGINT → clean exit within 5s; checkpoint exists; restart loads checkpoint; no corruption
  - Integration: `Trigger=SIGINT/SIGTERM; Callsite=tokio signal handler; State=training stops; Persistence=final checkpoint; Signal=log "Shutdown complete"`
  - Rollback: checkpoint format changes between versions

---

## Stage 4b: Validator Oracle
Source spec: specs/031626-04b-validator-oracle.md

- [ ] **VO-01** — CLI and Chain Registration
  - Where: `crates/myosu-validator/src/main.rs (new)`, `src/chain.rs (new)`
  - Depends on: `CC-01`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Validators must be registered and staked to submit weights
  - Verify: Registers on subnet; stakes tokens; has ValidatorPermit after epoch
  - Integration: `Trigger=myosu-validator CLI; Callsite=main.rs; State=registered and staked; Persistence=on-chain; Signal=ValidatorPermit active`
  - Rollback: ValidatorPermit requires more stake than available

- [ ] **VO-02** — Miner Discovery and Query
  - Where: `crates/myosu-validator/src/evaluator.rs (new)`
  - Depends on: `VO-01`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Without miner queries, validator has nothing to score
  - Verify: Discovers all miners with axons; queries strategy; timeout → score 0; invalid response → score 0
  - Integration: `Trigger=evaluation timer; Callsite=evaluator.rs; State=responses collected; Persistence=N/A; Signal=log "Queried {n} miners"`
  - Rollback: miner axon protocol incompatible

- [ ] **VO-03** — Deterministic Test Position Generation
  - Where: `crates/myosu-validator/src/positions.rs (new)`
  - Depends on: `PE-04`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Deterministic positions required for INV-003
  - Verify: Same seed → identical positions; different seeds → different; covers all streets; all positions valid
  - Integration: `Trigger=evaluation start; Callsite=evaluator.rs; State=position set generated; Persistence=N/A; Signal=positions logged`
  - Rollback: encoder requires pre-loaded abstractions not available

- [ ] **VO-04** — Exploitability Scoring
  - Where: `crates/myosu-validator/src/scoring.rs (new)`
  - Depends on: `VO-03`, `PE-04`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Scoring function is Yuma's input — bad scores → bad incentives
  - Verify: Nash-like → weight ~65535; random → weight ~0; unresponsive → 0; deterministic (INV-003)
  - Integration: `Trigger=after miner queries; Callsite=evaluator.rs; State=scores computed; Persistence=N/A; Signal=scores logged`
  - Rollback: exploitability too slow for per-epoch evaluation

- [ ] **VO-05** — Weight Submission (Direct and Commit-Reveal)
  - Where: `crates/myosu-validator/src/submitter.rs (new)`
  - Depends on: `VO-01`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Without weight submission, Yuma has no input
  - Verify: Direct submission succeeds; commit-reveal flow succeeds; weights are u16; empty subnet → no submission
  - Integration: `Trigger=after scoring; Callsite=submitter.rs; State=weights on chain; Persistence=on-chain; Signal=WeightsSet event`
  - Rollback: commit-reveal timing window misaligned

- [ ] **VO-06** — Evaluation Loop Orchestration
  - Where: `crates/myosu-validator/src/main.rs (extend)`
  - Depends on: `VO-02`, `VO-04`, `VO-05`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Orchestration ties all pieces together
  - Verify: Full cycle completes within tempo; failed queries don't block; graceful shutdown
  - Integration: `Trigger=block polling; Callsite=main.rs; State=weights submitted per tempo; Persistence=on-chain; Signal=log "Submitted weights for {n} miners"`
  - Rollback: evaluation takes longer than tempo period

- [ ] **VO-07** — Two-Validator INV-003 Agreement Test
  - Where: `crates/myosu-validator/tests/determinism.rs (new)`
  - Depends on: `VO-06`
  - Tests: `cargo check -p myosu-validator`
  - Blocking: Most important correctness property — if validators disagree, Yuma consensus is meaningless
  - Verify: Two independently initialized validators score same miner on same epoch; scores identical within epsilon (1e-6); different encoder hashes → test fails; same positions generated from same seed
  - Integration: `Trigger=cargo test; Callsite=tests/determinism.rs; State=two validator instances; Persistence=N/A; Signal=score_a == score_b within epsilon`
  - Rollback: floating-point non-determinism in exploitability computation

---

## Stage 5: Product Layer
Source spec: specs/031626-05-gameplay-cli.md

- [ ] **GP-01** — Best Miner Discovery
  - Where: `crates/myosu-play/src/discovery.rs (new)`
  - Depends on: `CC-01`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Players should face the strongest bot available
  - Verify: Returns miner with highest incentive; unreachable → fallback; no miners → random bot
  - Integration: `Trigger=game session start; Callsite=main.rs; State=best miner selected; Persistence=N/A; Signal=log "Connected to miner UID {n}"`
  - Rollback: incentive scores not populated before first epoch

- [ ] **GP-02** — Interactive Game Loop
  - Where: `crates/myosu-play/src/game_loop.rs (new)`
  - Depends on: `GP-01`, `PE-02`
  - Tests: `cargo check -p myosu-play`
  - Blocking: This is the user-facing product
  - Verify: Fold → hand ends; showdown → best hand wins; invalid input reprompts; all-in resolves; stats track correctly
  - Integration: `Trigger=myosu-play CLI; Callsite=main.rs; State=game state via robopoker; Persistence=N/A; Signal=hand result displayed`
  - Rollback: robopoker Game can't represent states needed for display

- [ ] **GP-03** — Bot Strategy Integration
  - Where: `crates/myosu-play/src/bot.rs (new)`
  - Depends on: `GP-02`, `PE-01`
  - Tests: `cargo check -p myosu-play`
  - Blocking: The bot is what makes the game challenging
  - Verify: Bot queries miner and returns legal action; timeout → random; sampled action always legal; zero-prob actions never sampled
  - Integration: `Trigger=bot's turn; Callsite=game_loop.rs; State=action sampled; Persistence=N/A; Signal=bot action displayed`
  - Rollback: miner query latency > 500ms

- [ ] **GP-04** — Hand History Recording
  - Where: `crates/myosu-play/src/recorder.rs (new)`
  - Depends on: `GP-02`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Essential for player review and debugging
  - Verify: JSON file created; all actions recorded in order; session stats correct; disk full → warning, continue
  - Integration: `Trigger=hand completes; Callsite=game_loop.rs; State=N/A; Persistence=hands/*.json; Signal=file created`
  - Rollback: N/A — failures don't block gameplay

---

## Stage 6: Multi-Game Validation
Source spec: specs/031626-06-multi-game-architecture.md

- [ ] **MG-01** — Liar's Dice Game Engine
  - Where: `crates/myosu-games-liars-dice/ (new)`
  - Depends on: `GT-01`
  - Tests: `cargo check -p myosu-games-liars-dice`
  - Blocking: Architectural proof — if Liar's Dice can't implement CfrGame, multi-game claim is false
  - Verify: Root is chance node; legal bids increase; challenge resolves; payoff is zero-sum; all trait bounds satisfied
  - Integration: `Trigger=solver creates game; Callsite=training loop; State=game transitions; Persistence=N/A; Signal=tests pass`
  - Rollback: CfrGame: Copy impossible for variable-length bid history

- [ ] **MG-02** — Liar's Dice Solver and Nash Verification
  - Where: `crates/myosu-games-liars-dice/ (extend)`
  - Depends on: `MG-01`
  - Tests: `cargo check -p myosu-games-liars-dice`
  - Blocking: Exact Nash verification is strongest possible proof of trait system correctness
  - Verify: 10K iterations → exploit < 0.001; strategy is mixed; probabilities valid; wire serialization works
  - Integration: `Trigger=test harness; Callsite=test suite; State=solver converges; Persistence=N/A; Signal=exploit assertion passes`
  - Rollback: convergence requires > 100K iterations (impl error)

- [ ] **MG-03** — Zero-Change Verification
  - Where: `crates/myosu-games-liars-dice/tests/ (new)`
  - Depends on: `MG-02`, `PE-01`
  - Tests: `cargo check -p myosu-games && cargo check -p myosu-games-poker`
  - Blocking: The zero-change property IS the architectural claim
  - Verify: All existing tests pass without modification; no diff in existing crate sources
  - Integration: `Trigger=cargo test; Callsite=CI; State=N/A; Persistence=N/A; Signal=all tests green`
  - Rollback: existing traits need modification for Liar's Dice

- [ ] **MG-04** — Multi-Game Expansion Guide
  - Where: `docs/multi-game-expansion.md (new)`
  - Depends on: `MG-03`
  - Tests: N/A (documentation)
  - Blocking: Guides future development and investor conversations
  - Verify: Covers 6 candidate games; concrete type signatures; estimated info set counts; reviewed
  - Integration: `N/A (documentation)`
  - Rollback: N/A

---

## Stage 7: TUI Implementation
Source spec: specs/031626-07-tui-implementation.md

- [ ] **TU-01** — GameRenderer Trait
  - Where: `crates/myosu-tui/src/renderer.rs (new)`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Every game renderer depends on this trait — must be stable first
  - Verify: Object-safe (Box<dyn GameRenderer> compiles); mock renderer works; pipe_output returns structured text; completions non-empty
  - Integration: `Trigger=compile-time; Callsite=shell.rs calls render_state(); State=N/A; Persistence=N/A; Signal=trait compiles`
  - Rollback: trait requires game-specific types

- [ ] **TU-07** — Color Theme Implementation
  - Where: `crates/myosu-tui/src/theme.rs (new)`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Shell layout needs theme for declaration styling
  - Verify: All 8 color tokens from design.md defined; readable without color
  - Integration: `Trigger=compile-time; Callsite=shell.rs applies theme; State=N/A; Persistence=N/A; Signal=tests pass`
  - Rollback: N/A

- [ ] **TU-02** — Five-Panel Shell Layout
  - Where: `crates/myosu-tui/src/shell.rs (new)`
  - Depends on: `TU-01`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Universal visual frame for all 20 games
  - Verify: 5 panels render at 60-120 columns; state panel min 4 lines; log scrolls; header shows game path
  - Integration: `Trigger=resize or state change; Callsite=event loop; State=frame buffer; Persistence=N/A; Signal=5 panels visible`
  - Rollback: layout constraints conflict at small terminal sizes

- [x] **TU-04** — Readline Input with History
  - Where: `crates/myosu-tui/src/input.rs (new)`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Input quality determines gameplay feel
  - Verify: Type + submit works; history navigation; tab completion; Ctrl-W deletes word; /commands detected
  - Integration: `Trigger=key events; Callsite=events.rs; State=buffer, cursor, history; Persistence=N/A; Signal=characters appear`
  - Rollback: readline keybindings conflict with game keys

- [ ] **TU-03** — Event Loop and Async Updates
  - Where: `crates/myosu-tui/src/events.rs (new)`
  - Depends on: `TU-01`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Ties shell + input + async miner queries together
  - Verify: Key press triggers re-render within 16ms; miner response updates state without blocking; Ctrl-C clean shutdown
  - Integration: `Trigger=key event or miner response; Callsite=main.rs; State=game + terminal; Persistence=N/A; Signal=responsive UI`
  - Rollback: crossterm and tokio event loops conflict

- [ ] **TU-05** — Screen State Machine
  - Where: `crates/myosu-tui/src/screens.rs (new)`
  - Depends on: `TU-03`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Navigation between game states
  - Verify: Lobby→Game on subnet select; Game→Stats on /stats; /analyze→Coaching; any key returns from overlay
  - Integration: `Trigger=/commands or game completion; Callsite=event loop; State=Screen enum; Persistence=N/A; Signal=display switches`
  - Rollback: screen transitions lose game state

- [ ] **TU-06** — Pipe Mode for Agent Protocol
  - Where: `crates/myosu-tui/src/pipe.rs (new)`
  - Depends on: `TU-01`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Agent-native design depends on pipe mode
  - Verify: --pipe output has zero ANSI codes; matches design.md pipe format; stdin accepted; agent plays complete hand
  - Integration: `Trigger=--pipe flag; Callsite=main.rs; State=stdin/stdout; Persistence=hand history; Signal=structured text output`
  - Rollback: pipe vs TUI rendering diverges

- [ ] **TU-10** — Blueprint Strategy Loading
  - Where: `crates/myosu-play/src/blueprint.rs (new)`
  - Depends on: `RF-04`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Enables trained bot + solver advisor. Without this, training mode uses heuristic-only bot.
  - Verify: Artifact discovery from env var / default path / home dir; manifest schema validation; mmap strategy lookup < 1μs; hash mismatch → error with actionable message; distribution sums to 1.0; all returned actions are legal
  - Integration: `Trigger=training mode startup; Callsite=training.rs; State=mmap files opened; Persistence=read-only; Signal=~ bot strategy: blueprint · exploit X mbb/h`
  - Test fixture: `~/.codexpoker/blueprint/` — 113M infosets, 335M edges, schema v1. Validated 2026-03-17: +2.35 chips/hand vs random (1000 hands), mirror ~0 (balanced). Set `MYOSU_BLUEPRINT_DIR=~/.codexpoker/blueprint` to use. Edge regression test: assert > 1.0 chips/hand vs random. Balance test: assert |edge| < 1.0 mirror. Port reference: `codexpoker/src/bin/bot_duel.rs`, `practice_probe.rs`, `test_blueprint_load.rs`
  - Rollback: myosu abstraction format incompatible with codexpoker format

- [ ] **TU-08** — NLHE Poker Renderer and Truth Stream
  - Where: `crates/myosu-games-poker/src/renderer.rs (new)`, `crates/myosu-games-poker/src/truth_stream.rs (new)`
  - Depends on: `TU-02`, `PE-01`
  - Tests: `cargo check -p myosu-games-poker`
  - Blocking: Reference GameRenderer implementation. Validates TU-01 trait design and establishes pattern for all game renderers.
  - Verify: State panel shows cards with suit symbols, board, pot, stacks, hero cards; truth stream processes Events into log lines with visual grammar (icons, separators, colors); parse_input handles poker actions (f/c/r/s); pipe_output is machine-parseable with zero ANSI codes; pot odds and MDF calculated correctly
  - Integration: `Trigger=game state change; Callsite=shell.rs calls render_state(); State=gameboard + truth stream; Persistence=N/A; Signal=cards render, actions appear in log`
  - Rollback: GameRenderer trait needs methods not anticipated by TU-01

- [ ] **TU-09** — Training Mode (Local Bot Play)
  - Where: `crates/myosu-play/src/training.rs (new)`
  - Depends on: `TU-08`, `TU-10`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Phase 0 poker experience. Standalone play without chain infrastructure.
  - Verify: Hand completes via fold and showdown; /deal sets hero cards; /board sets board; /stack sets stacks; /showdown forces runout; practice chips start at 10,000 and update correctly; bot uses blueprint (TU-10) or heuristic fallback; alternating button; bot acts within 500ms
  - Integration: `Trigger=myosu-play --train or /practice; Callsite=main.rs creates TrainingTable; State=game + chips + pending commands; Persistence=hand history JSON; Signal=hands play to completion`
  - Rollback: robopoker Game API insufficient for bot dispatch

- [ ] **TU-11** — Solver Advisor
  - Where: `crates/myosu-play/src/advisor.rs (new)`
  - Depends on: `TU-08`, `TU-10`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Key differentiating feature — transforms training mode from "play against bot" to "learn GTO from trained solver"
  - Verify: Shows action distribution when hero has pending decision; filters actions < 1% probability; round probabilities to integers; toggle with /advisor; ON by default in training mode; distribution from same backend as bot; format matches "SOLVER: fold X% · call Y% · raise Z%"
  - Integration: `Trigger=hero decision pending + advisor enabled; Callsite=NlheRenderer::render_state(); State=cached distribution; Persistence=N/A; Signal=advisor line visible in state panel`
  - Rollback: BotBackend::action_distribution() too slow (should be < 1ms)

- [ ] **TU-12** — Session Stats and HUD
  - Where: `crates/myosu-play/src/stats.rs (new)`, `crates/myosu-games-poker/src/hud.rs (new)`
  - Depends on: `TU-08`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Not blocking for core gameplay — provides session feedback loop
  - Verify: Tracks hands played, win rate (BB/h), total profit, showdown %; HUD shows bot VPIP/PFR/AF; stats < 30 hands marked unreliable (*); stats reset on session entry; /stats shows full session summary
  - Integration: `Trigger=hand completion; Callsite=training.rs updates stats; State=PlayerStats counters; Persistence=N/A (session-scoped); Signal=header shows chips, /stats shows summary`
  - Rollback: N/A — pure accumulation

---

## Stage 8: Abstraction Pipeline
Source spec: specs/031626-08-abstraction-pipeline.md

- [x] **AP-01** — Clustering Binary
  - Where: `crates/myosu-cluster/src/main.rs (new)`
  - Depends on: `RF-03`
  - Tests: `cargo check -p myosu-cluster`
  - Blocking: Without abstraction tables, miners produce random strategies
  - Verify: Preflop produces 169 entries; all 4 files written; manifest SHA-256 matches; deterministic re-run
  - Integration: `Trigger=myosu-cluster CLI; Callsite=main.rs; State=clustering state; Persistence=4 bin files + manifest.json; Signal=manifest with hashes`
  - Rollback: robopoker clustering API not accessible outside rbp-autotrain

- [ ] **AP-02** — File-Based Encoder Loading
  - Where: `happybigmtn/robopoker rbp-nlhe/src/encoder.rs (extend)`
  - Depends on: `RF-02`
  - Tests: `cargo test -p rbp-nlhe encoder::tests::from_dir_loads_all_streets`
  - Blocking: Builds on RF-02 (from_map/from_file constructors) — adds from_dir with multi-street loading and hash verification
  - Verify: from_dir loads all 4 streets; abstraction() returns valid values; tampered file rejected; hash deterministic
  - Integration: `Trigger=miner startup; Callsite=PokerSolver::new(); State=encoder populated; Persistence=read-only; Signal=hash logged`
  - Rollback: 138M entries don't fit in memory

- [x] **AP-03** — Pre-Computed Artifact Distribution
  - Where: `artifacts/abstractions/ (new)`, `scripts/download-abstractions.sh (new)`
  - Depends on: `AP-01`, `AP-02`
  - Tests: `scripts/download-abstractions.sh exits 0`
  - Blocking: Without pre-computed artifacts, every miner spends hours clustering
  - Verify: Download completes in <5min on 100Mbps; hash matches; miner starts after download; tampered artifact rejected
  - Integration: `Trigger=miner startup detects missing abstractions; Callsite=main.rs bootstrap; State=files on disk; Persistence=~3GB; Signal=hash logged`
  - Rollback: artifact URL unavailable

---

## Stage 9: Launch Integration (NLHE HU as Product)
Source spec: specs/031626-09-launch-integration.md

- [ ] **LI-01** — Devnet Orchestration
  - Where: `ops/devnet/docker-compose.yml (new)`, `ops/devnet/README.md (new)`, `crates/myosu-chain/Dockerfile (new)`, `crates/myosu-miner/Dockerfile (new)`, `crates/myosu-validator/Dockerfile (new)`
  - Depends on: `CF-05`, `MN-01`, `VO-01`
  - Tests: `docker compose up -d && sleep 30 && curl -sf http://localhost:8080/health`
  - Blocking: Developers and testers need one command to run the full stack
  - Verify: Chain produces blocks within 10s; miner registers within 30s; validator submits within first tempo; curl /health responds; compose down cleans up
  - Integration: `Trigger=docker compose up; Callsite=ops/devnet/; State=full stack running; Persistence=volumes; Signal=all services healthy`
  - Rollback: Docker build times exceed 30 minutes

- [ ] **LI-02** — Miner Bootstrap Sequence
  - Where: `crates/myosu-miner/src/bootstrap.rs (new)`, `scripts/miner-bootstrap.sh (new)`
  - Depends on: `LI-01`, `MN-02`, `MN-03`
  - Tests: `cargo check -p myosu-miner`
  - Blocking: New miners must go from zero to operational automatically
  - Verify: Fresh miner downloads abstractions, registers, starts training; resume from checkpoint works; chain unreachable → retry with backoff; subnet not found → clear error
  - Integration: `Trigger=miner startup; Callsite=main.rs; State=10-step bootstrap; Persistence=abstractions + checkpoint; Signal=log at each step`
  - Rollback: abstraction download fails in Docker network

- [ ] **LI-03** — Gameplay ↔ TUI Wiring
  - Where: `crates/myosu-play/src/main.rs (extend)`, `crates/myosu-games-poker/src/renderer.rs (new)`
  - Depends on: `GP-02`, `TU-08`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Gameplay must render through the TUI shell, not ad-hoc CLI prompts
  - Verify: NlheRenderer implements GameRenderer; TUI matches design.md 8.1; bot actions in log; /stats works; --pipe plays one hand
  - Integration: `Trigger=myosu-play CLI; Callsite=main.rs creates TUI shell with NlheRenderer; State=game + TUI; Persistence=hand history; Signal=design.md screen rendered`
  - Rollback: GameRenderer trait can't express NLHE state panel

- [ ] **LI-04** — End-to-End Acceptance Test
  - Where: `tests/e2e/nlhe_launch.rs (new)`
  - Depends on: `LI-02`, `LI-03`
  - Tests: `true`
  - Blocking: If this test passes, we can launch. If it fails, we can't.
  - Verify: Chain → subnet → miner → train → validator → Yuma → emissions → gameplay → hand history. All 8 steps pass. Total < 120 seconds. Uses preflop-only abstractions for speed.
  - Integration: `Trigger=cargo test; Callsite=tests/e2e/; State=full stack lifecycle; Persistence=temp; Signal=all assertions pass`
  - Rollback: any component integration failure

- [ ] **LI-05** — Launch Readiness Checklist
  - Where: `docs/launch-checklist.md (new)`
  - Depends on: `LI-04`
  - Tests: N/A (documentation + manual verification)
  - Blocking: Prevents declaring launch before all critical paths work
  - Verify: All checklist items checkable; covers chain, solver, scoring, gameplay, integration; explicitly lists what's NOT required; does NOT list /analyze coaching output (deferred to post-launch)
  - Integration: `N/A (documentation)`
  - Rollback: N/A

- [ ] **LI-06** — Consolidated Invariant Gate Test
  - Where: `tests/e2e/invariant_gate.rs (new)`
  - Depends on: `LI-04`
  - Tests: `true`
  - Blocking: OS.md bootstrap exit requires "all 6 invariants pass"; no single test validates this
  - Verify: INV-003 (two validators agree within epsilon on same miner); INV-004 (cargo tree shows no path myosu-play→myosu-miner or reverse); INV-006 (robopoker fork CHANGELOG.md exists and documents changes from v1.0.0); emission accounting (sum distributions == block_emission * epochs)
  - Integration: `Trigger=cargo test; Callsite=tests/e2e/invariant_gate.rs; State=full stack + invariant checks; Persistence=N/A; Signal=all invariant assertions pass`
  - Rollback: any invariant violation discovered during gate test

---

## Stage 10: Agent Integration
Source spec: specs/031626-10-agent-experience.md

- [ ] **AX-01** — Game State JSON Schema
  - Where: `crates/myosu-tui/src/schema.rs (new)`, `docs/api/game-state.json (new)`
  - Depends on: `TU-01`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Any agent in any language needs machine-readable game state
  - Verify: JSON schema validates; legal_actions exhaustive (every valid action enumerated); parseable by Python/JS/Rust
  - Integration: `Trigger=game state change; Callsite=schema.rs; State=JSON output; Persistence=N/A; Signal=valid JSON`
  - Rollback: schema too rigid for non-poker games

- [ ] **AX-02** — Action JSON Schema
  - Where: `crates/myosu-tui/src/schema.rs (extend)`
  - Depends on: `AX-01`
  - Tests: `cargo check -p myosu-tui`
  - Blocking: Agents need a structured way to submit actions with error recovery
  - Verify: Valid action accepted; invalid returns 400 with legal_actions; all action types roundtrip
  - Integration: `Trigger=agent submits action; Callsite=api.rs; State=game updated; Persistence=N/A; Signal=updated state returned`
  - Rollback: action format too complex for simple bots

- [ ] **AX-03** — HTTP Game API
  - Where: `crates/myosu-play/src/api.rs (new)`
  - Depends on: `AX-02`, `GP-02`
  - Tests: `cargo check -p myosu-play`
  - Blocking: HTTP is universal — Claude Code, Python scripts, curl all use it
  - Verify: Create session; submit action; hand completes; invalid action → 400 with legal_actions; 10 concurrent sessions
  - Integration: `Trigger=HTTP request; Callsite=api.rs; State=session state server-side; Persistence=hand history; Signal=JSON response`
  - Rollback: HTTP latency too high for competitive play

- [ ] **AX-04** — WebSocket Game API
  - Where: `crates/myosu-play/src/ws.rs (new)`
  - Depends on: `AX-03`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Persistent connections with server-push for responsive agent play
  - Verify: Connect and play one hand; spectator receives updates; reconnect preserves session
  - Integration: `Trigger=WS connection; Callsite=ws.rs; State=persistent session; Persistence=hand history; Signal=JSON frames`
  - Rollback: WS complexity not justified for launch

- [ ] **AX-05** — Python SDK
  - Where: `sdk/python/myosu/ (new)`
  - Depends on: `AX-03`
  - Tests: `pytest sdk/python/tests/test_client.py`
  - Blocking: Python is the LLM tool-use lingua franca — 5-line bot must be possible
  - Verify: create_session works; act() returns updated state; strategy callback plays N hands; pip install from git
  - Integration: `Trigger=import myosu; Callsite=Python process; State=HTTP session; Persistence=N/A; Signal=game.result populated`
  - Rollback: SDK maintenance burden too high for small team

- [ ] **AX-06** — Bot Registration (Bring Your Own Strategy)
  - Where: `crates/myosu-play/src/api.rs (extend)`
  - Depends on: `AX-03`
  - Tests: `cargo check -p myosu-play`
  - Blocking: Agents need to compete against each other and against the solver
  - Verify: bot-vs-solver mode works; bot-vs-bot mode works (two API clients, myosu hosts engine); spectate mode works
  - Integration: `Trigger=POST /sessions with mode; Callsite=api.rs; State=game with two API players; Persistence=hand history; Signal=both players receive state`
  - Rollback: multi-player API sessions too complex for launch
