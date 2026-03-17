# Implementation Plan

Status: Active
Date: 2026-03-16

## Spec Index

| Spec File | AC Prefix | Count |
|-----------|-----------|-------|
| `031626-00-master-index.md` | — | index |
| (robopoker fork prerequisites) | RF-01..02 | 2 |
| `031626-01-chain-fork-scaffold.md` | CF-01..05 | 5 |
| `031626-02a-game-engine-traits.md` | GT-01..05 | 5 |
| `031626-02b-poker-engine.md` | PE-01..04 | 4 |
| `031626-03-game-solving-pallet.md` | GS-01..10 | 10 |
| (shared chain client) | CC-01 | 1 |
| `031626-04a-miner-binary.md` | MN-01..05 | 5 |
| `031626-04b-validator-oracle.md` | VO-01..06 | 6 |
| `031626-05-gameplay-cli.md` | GP-01..04 | 4 |
| `031626-06-multi-game-architecture.md` | MG-01..04 | 4 |
| `031626-07-tui-implementation.md` | TU-01..07 | 7 |
| `031626-08-abstraction-pipeline.md` | AP-01..03 | 3 |
| `031626-09-launch-integration.md` | LI-01..05 | 5 |
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
  - Tests: `cargo test -p rbp-nlhe encoder::tests::from_map_constructor`
  - Blocking: PE-01 cannot create a functional solver without populated encoder
  - Verify: `NlheEncoder::from_map(BTreeMap<Isomorphism, Abstraction>)` works; `NlheEncoder::from_file(path)` loads binary abstraction table; existing DB path unaffected
  - Integration: `Trigger=miner creates encoder on startup; Callsite=PokerSolver::new(); State=encoder populated; Persistence=abstraction file on disk; Signal=abstraction() returns valid values`
  - Rollback: private field prevents non-DB construction without refactoring

---

## Stage 1: Chain Fork Scaffold
Source spec: specs/031626-01-chain-fork-scaffold.md

- [ ] **CF-02** — Prune Workspace Dependencies
  - Where: `crates/myosu-chain/Cargo.toml (new)`, `crates/myosu-chain/runtime/Cargo.toml (new)`
  - Tests: `! cargo tree -p myosu-runtime 2>&1 | grep -q 'pallet.subtensor'`
  - Blocking: Dependency contamination from stripped pallets will cause build failures
  - Verify: No references to subtensor/frontier/drand in dependency tree; build succeeds; WASM blob produced
  - Integration: `Trigger=cargo build; Callsite=Cargo resolver; State=dependency tree resolved; Persistence=Cargo.lock; Signal=cargo tree shows no stripped crates`
  - Rollback: Transitive dependency on stripped pallet that can't be resolved

- [ ] **CF-01** — Strip AI/EVM Pallets from Runtime
  - Where: `crates/myosu-chain/runtime/src/lib.rs (new, from subtensor)`
  - Tests: `cargo test -p myosu-runtime runtime::tests::runtime_compiles`
  - Blocking: Everything downstream depends on a compilable runtime — single most critical unblock
  - Verify: Runtime compiles; spec_name="myosu"; 14 pallets in construct_runtime! (incl SafeMode at index 20); no subtensor/frontier/EVM references; WASM < 5MB
  - Integration: `Trigger=cargo build -p myosu-runtime; Callsite=runtime/build.rs WASM builder; State=WASM blob compiled; Persistence=target/ artifact; Signal=build exits 0`
  - Rollback: Runtime fails to compile after stripping — hidden inter-pallet dependencies

- [ ] **CF-04** — Local Devnet Chain Spec
  - Where: `crates/myosu-chain/node/src/chain_spec.rs (new, from subtensor)`
  - Tests: `cargo test -p myosu-node chain_spec::tests::dev_spec_is_valid`
  - Blocking: Chain spec defines genesis state — node can't start without it
  - Verify: Dev and local specs produce valid genesis; Alice/Bob/Charlie/Dave/Eve/Ferdie funded with 1M MYOSU each; Alice is sudo; token symbol MYOSU, 9 decimals; no subtensor/EVM genesis
  - Integration: `Trigger=node startup reads chain spec; Callsite=node/src/command.rs; State=genesis block initialized; Persistence=genesis block in DB; Signal=build-spec outputs valid JSON`
  - Rollback: Genesis requires types from stripped pallets

- [ ] **CF-03** — Minimal Node Service
  - Where: `crates/myosu-chain/node/src/service.rs (new, from subtensor)`
  - Tests: `cargo test -p myosu-node node::tests::service_starts`
  - Blocking: Node binary is the entry point — every test requires running it
  - Verify: Node binary compiles; `--dev` starts and produces blocks; `--help` shows subcommands; `build-spec` outputs JSON; clean shutdown on SIGINT; no panics
  - Integration: `Trigger=myosu-node --dev CLI; Callsite=node/src/main.rs; State=node service running, blocks produced; Persistence=block database; Signal=block numbers incrementing in logs`
  - Rollback: service.rs has deep coupling to Frontier/drand that can't be cleanly removed

- [ ] **CF-05** — End-to-End Devnet Smoke Test
  - Where: `crates/myosu-chain/tests/integration.rs (new)`
  - Tests: `cargo test -p myosu-chain integration::tests::devnet_smoke_test`
  - Blocking: Gate proving CH-01 is complete — unit tests alone are not sufficient
  - Verify: Node starts and produces block 1 within 15s; system_health RPC responds; Alice has 1M MYOSU; balance transfer Alice→Bob succeeds; node shuts down cleanly; no panics in logs
  - Integration: `Trigger=cargo test; Callsite=tests/integration.rs; State=devnet lifecycle; Persistence=temp block DB (cleaned up); Signal=test passes, block > 0, transfer confirmed`
  - Rollback: Node fails to start, RPC unresponsive, or transactions fail

---

## Stage 2a: Game Engine Traits
Source spec: specs/031626-02a-game-engine-traits.md

- [ ] **GT-01** — Re-export and Extend Robopoker CFR Traits
  - Where: `crates/myosu-games/src/traits.rs (new)`
  - Tests: `cargo test -p myosu-games traits::tests::reexports_compile`
  - Blocking: Every other AC and downstream spec depends on these types
  - Verify: CfrGame, Profile, Encoder importable; GameConfig serializes; StrategyQuery/Response round-trips
  - Integration: `Trigger=compile-time; Callsite=all downstream crates; State=N/A; Persistence=N/A; Signal=cargo test passes`
  - Rollback: rbp-mccfr has incompatible dependency requirements

- [ ] **GT-02** — Wire Serialization for Strategy Transport
  - Where: `crates/myosu-games/src/wire.rs (new)`
  - Tests: `cargo test -p myosu-games wire::tests::wire_strategy_roundtrip`
  - Blocking: Miners and validators must agree on serialization format
  - Verify: WireStrategy serializes to JSON and round-trips; action probabilities preserved; invalid bytes → error not panic
  - Integration: `Trigger=miner serializes, validator deserializes; Callsite=axon handler; State=N/A; Persistence=N/A; Signal=round-trip test passes`
  - Rollback: robopoker Edge/Info types don't implement Serialize

- [ ] **GT-03** — Runtime Game Selection
  - Where: `crates/myosu-games/src/registry.rs (new)`
  - Tests: `cargo test -p myosu-games registry::tests::known_game_types`
  - Blocking: Miners need to select correct solver for their subnet
  - Verify: from_bytes maps known types; unknown → Custom; roundtrip bytes; num_players correct
  - Integration: `Trigger=miner/validator reads subnet game_type; Callsite=main.rs; State=correct engine selected; Persistence=N/A; Signal=from_bytes returns correct variant`
  - Rollback: game_type encoding on-chain doesn't match registry

- [ ] **GT-04** — Remote Strategy Profile Adapter
  - Where: `crates/myosu-games/src/remote_profile.rs (new)`
  - Tests: `cargo test -p myosu-games remote_profile::tests::rps_nash_exploitability_zero`
  - Blocking: Validators need to compute exploitability from miner query responses without the full Profile object
  - Verify: RemoteProfile from RPS Nash distributions → exploit ≈ 0; from always-rock → exploit > 0; matches local Profile within 1%; missing info set → uniform fallback
  - Integration: `Trigger=validator builds RemoteProfile from responses; Callsite=myosu-validator scoring.rs; State=HashMap of info→distributions; Persistence=N/A; Signal=profile.exploitability(tree) returns valid f64`
  - Rollback: Profile trait requires state beyond cum_weight for exploitability path

- [ ] **GT-05** — RPS Reference Implementation Test Suite
  - Where: `crates/myosu-games/tests/rps_integration.rs (new)`
  - Tests: `cargo test -p myosu-games rps_integration::train_rps_to_nash`
  - Blocking: If RPS doesn't work, nothing will — validates entire trait system
  - Verify: 1000 iterations → each action ~1/3; exploitability < 0.01; wire roundtrip preserves data; always-rock is exploitable
  - Integration: `Trigger=cargo test; Callsite=test suite; State=N/A; Persistence=N/A; Signal=all RPS tests pass`
  - Rollback: robopoker RPS module incompatible with wire layer

---

## Stage 2b: Poker Engine
Source spec: specs/031626-02b-poker-engine.md

- [ ] **PE-01** — Poker Solver Wrapper
  - Where: `crates/myosu-games-poker/src/solver.rs (new)`
  - Tests: `cargo test -p myosu-games-poker solver::tests::train_100_iterations`
  - Blocking: Core of what miners do — no solver means no strategies
  - Verify: Empty solver has 0 epochs; 100 iterations → epochs=100; strategy sums to ~1.0; checkpoint roundtrips; exploitability decreases over training
  - Integration: `Trigger=miner creates solver; Callsite=myosu-miner/main.rs; State=training state in memory; Persistence=checkpoint files; Signal=epochs() increases`
  - Rollback: robopoker Flagship type aliases not public or NlheProfile not serializable

- [ ] **PE-02** — Strategy Query Handler
  - Where: `crates/myosu-games-poker/src/query.rs (new)`
  - Tests: `cargo test -p myosu-games-poker query::tests::handle_valid_query`
  - Blocking: Miner axon needs a request handler
  - Verify: Valid query → response with distribution; invalid bytes → error; probabilities sum to 1.0
  - Integration: `Trigger=HTTP request to axon; Callsite=axon handler; State=N/A (read-only); Persistence=N/A; Signal=valid WireStrategy response`
  - Rollback: NlheInfo serialization incompatible with validator queries

- [ ] **PE-03** — Poker Wire Serialization
  - Where: `crates/myosu-games-poker/src/wire.rs (new)`
  - Tests: `cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip`
  - Blocking: Without serialization, miners and validators can't communicate
  - Verify: NlheInfo roundtrips; all NlheEdge variants roundtrip; size < 1KB; corrupted bytes → error
  - Integration: `Trigger=miner serializes, validator deserializes; Callsite=query handler + oracle; State=N/A; Persistence=N/A; Signal=roundtrip tests pass`
  - Rollback: robopoker types don't support serde with feature flags

- [ ] **PE-04** — Poker Exploitability Integration
  - Where: `crates/myosu-games-poker/src/exploit.rs (new)`
  - Tests: `cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit`
  - Blocking: Validator scoring function for poker miners
  - Verify: Trained strategy < 500 mbb/h; random > 200 mbb/h; remote matches local within 5%; always non-negative
  - Integration: `Trigger=validator evaluation loop; Callsite=myosu-validator oracle; State=N/A; Persistence=N/A; Signal=returns f64 exploitability`
  - Rollback: exploitability > 60s for HU NLHE

---

## Stage 3: On-Chain Incentives
Source spec: specs/031626-03-game-solving-pallet.md

- [ ] **GS-01** — Pallet Scaffold with Config and Storage
  - Where: `crates/myosu-chain/pallets/game-solver/src/lib.rs (new)`
  - Tests: `cargo test -p pallet-game-solver scaffold::tests::pallet_compiles`
  - Blocking: Every other GS-* AC reads or writes these storage items
  - Verify: Pallet compiles; ~25 storage items with correct defaults; Config trait has Currency + constants; AxonInfo encodes/decodes; mock runtime works
  - Integration: `Trigger=construct_runtime!; Callsite=runtime/src/lib.rs; State=storage items available; Persistence=on-chain storage; Signal=cargo build succeeds`
  - Rollback: Storage layout incompatible with Yuma's access patterns

- [ ] **GS-02** — Subnet Registry
  - Where: `crates/myosu-chain/pallets/game-solver/src/subnets.rs (new)`
  - Tests: `cargo test -p pallet-game-solver subnets::tests::create_subnet_basic`
  - Blocking: Neurons can only register on existing subnets — unblocks GS-03
  - Verify: create_subnet burns tokens, assigns id, emits event; dissolve clears state; max subnet limit enforced; set_hyperparams sudo-only
  - Integration: `Trigger=create_subnet extrinsic; Callsite=pallet dispatchable; State=subnet registered with game_type; Persistence=10 storage items per subnet; Signal=SubnetCreated event`
  - Rollback: game_type storage can't represent needed metadata

- [ ] **GS-03** — Neuron Registration and Pruning
  - Where: `crates/myosu-chain/pallets/game-solver/src/registration.rs (new)`
  - Tests: `cargo test -p pallet-game-solver registration::tests::register_basic`
  - Blocking: Without neurons, nothing to weight or reward — unblocks GS-04, GS-07
  - Verify: Sequential UIDs assigned; burn cost enforced; full subnet prunes weakest; immunity period respected; duplicate rejected; storage vectors extended
  - Integration: `Trigger=register_neuron extrinsic; Callsite=pallet dispatchable; State=UID assigned, vectors extended; Persistence=Keys, Uids, IsNetworkMember; Signal=NeuronRegistered event`
  - Rollback: Pruning logic incorrectly removes high-scoring neurons

- [ ] **GS-04** — Weight Submission
  - Where: `crates/myosu-chain/pallets/game-solver/src/weights.rs (new)`
  - Tests: `cargo test -p pallet-game-solver weights::tests::set_weights_basic`
  - Blocking: Weights feed Yuma Consensus — bridge between off-chain evaluation and on-chain incentives
  - Verify: set_weights stores and validates; commit_weights stores hash; reveal_weights verifies and stores; rate limiting enforced; validator permit required
  - Integration: `Trigger=set_weights/commit_weights extrinsic; Callsite=pallet dispatchable; State=weight matrix updated; Persistence=Weights storage; Signal=WeightsSet event`
  - Rollback: Weight validation rejects valid game-solving patterns

- [ ] **GS-05** — Yuma Consensus Port
  - Where: `crates/myosu-chain/pallets/game-solver/src/epoch.rs (new)`, `src/math.rs (new)`
  - Tests: `cargo test -p pallet-game-solver epoch::tests::yuma_matches_subtensor_output`
  - Blocking: Core algorithm — getting it wrong breaks the entire incentive mechanism
  - Verify: Consensus clips above median; bonds accumulate via EMA; INV-003: bit-identical output to subtensor for same inputs; zero weights → zero emission
  - Integration: `Trigger=on_initialize at tempo boundary; Callsite=lib.rs::Hooks::on_initialize(); State=scores computed, bonds updated; Persistence=7 storage maps updated; Signal=EpochCompleted event`
  - Rollback: Fixed-point math diverges from subtensor due to dependency mismatch

- [ ] **GS-06** — Emission Distribution
  - Where: `crates/myosu-chain/pallets/game-solver/src/emission.rs (new)`
  - Tests: `cargo test -p pallet-game-solver emission::tests::equal_subnet_split`
  - Blocking: Emission is the revenue model — miners won't run solvers without economic reward
  - Verify: Equal subnet split; 50/50 miner/validator; proportional to scores; no emission without weights; TotalIssuance tracks correctly; no tokens created from thin air
  - Integration: `Trigger=on_initialize after run_epoch; Callsite=lib.rs::Hooks; State=balances increased; Persistence=balance updates + TotalIssuance; Signal=EmissionDistributed event`
  - Rollback: Emission accounting error creates or destroys tokens

- [ ] **GS-07** — Axon Serving
  - Where: `crates/myosu-chain/pallets/game-solver/src/serving.rs (new)`
  - Tests: `cargo test -p pallet-game-solver serving::tests::serve_axon_basic`
  - Blocking: Without axon discovery, validators can't find miners
  - Verify: Registered neuron serves axon; unregistered rejected; IP/port validated; rate limited; queryable via RPC
  - Integration: `Trigger=serve_axon extrinsic; Callsite=pallet dispatchable; State=AxonInfo stored; Persistence=Axons storage; Signal=AxonServed event`
  - Rollback: AxonInfo encoding incompatible with miner endpoint format

- [ ] **GS-08** — Basic Staking
  - Where: `crates/myosu-chain/pallets/game-solver/src/staking.rs (new)`
  - Tests: `cargo test -p pallet-game-solver staking::tests::add_stake_basic`
  - Blocking: Without staking, all validators have equal power — no skin-in-the-game
  - Verify: add_stake transfers tokens; remove_stake returns tokens; stake determines validator power; insufficient balance/stake rejected
  - Integration: `Trigger=add_stake/remove_stake extrinsic; Callsite=pallet dispatchable; State=Stake updated; Persistence=Stake storage + balance reserve; Signal=StakeAdded/StakeRemoved event`
  - Rollback: Stake accounting error creates or loses tokens

- [ ] **GS-09** — Add Pallet to Runtime at Index 7
  - Where: `crates/myosu-chain/runtime/src/lib.rs (extend)`, `node/src/chain_spec.rs (extend)`
  - Tests: `cargo test -p myosu-chain integration::tests::full_incentive_loop`
  - Blocking: Integration gate — until pallet is in runtime, it's just library code
  - Verify: Runtime compiles with pallet; index 7 occupied; create_subnet callable; full loop works (create→register→stake→weights→epoch→emission); no CF-05 regression; dev chain spec includes genesis subnet 1 (nlhe_hu, owned by Alice)
  - Integration: `Trigger=runtime compilation; Callsite=runtime/src/lib.rs; State=pallet in block execution; Persistence=pallet storage in chain state; Signal=create_subnet callable via RPC`
  - Rollback: Config requirements conflict with existing runtime

- [ ] **GS-10** — Runtime API for State Queries
  - Where: `crates/myosu-chain/pallets/game-solver/src/rpc.rs (new)`, `runtime/src/lib.rs (extend)`
  - Tests: `cargo test -p myosu-runtime runtime::tests::runtime_api_all_axons`
  - Blocking: Without efficient queries, off-chain participants have no practical way to discover chain state
  - Verify: subnet_info(1) returns SubnetInfo; all_axons(1) returns miner endpoints; all_incentives(1) returns scores after epoch; nonexistent subnet → None
  - Integration: `Trigger=RPC call; Callsite=runtime API impl; State=N/A (read-only); Persistence=N/A; Signal=all_axons returns data`
  - Rollback: runtime API types incompatible with subxt codegen

---

## Stage 4a: Shared Chain Client + Miner Binary
Source spec: specs/031626-04a-miner-binary.md

- [ ] **CC-01** — Shared Chain Client Crate
  - Where: `crates/myosu-chain-client/src/lib.rs (new)`
  - Tests: `cargo test -p myosu-chain-client client::tests::connect_to_devnet`
  - Blocking: MN-01, VO-01, GP-01 all need chain RPC — DRY violation without shared crate
  - Verify: Connects to devnet via WebSocket; submits extrinsics (register_neuron, serve_axon, set_weights, add_stake); queries storage (Axons, Incentive, SubnetInfo via runtime API); account/keypair management from seed string
  - Integration: `Trigger=any binary startup; Callsite=myosu-miner, myosu-validator, myosu-play; State=RPC connection; Persistence=N/A; Signal=chain queries return data`
  - Rollback: subxt codegen incompatible with myosu runtime metadata

- [ ] **MN-01** — CLI and Chain Registration
  - Where: `crates/myosu-miner/src/main.rs (new)`
  - Tests: `cargo test -p myosu-miner chain::tests::register_neuron_success`
  - Blocking: Registration is the first thing a miner does
  - Verify: Connects to devnet; registers and receives UID; already-registered skips; invalid key → error
  - Integration: `Trigger=myosu-miner CLI; Callsite=main.rs; State=neuron registered; Persistence=UID in local state; Signal=log "Registered as UID {n}"`
  - Rollback: subxt/RPC client incompatible with chain runtime

- [ ] **MN-02** — Background Training Loop
  - Where: `crates/myosu-miner/src/training.rs (new)`
  - Tests: `cargo test -p myosu-miner training::tests::training_loop_runs`
  - Blocking: Without training, miner serves random strategies
  - Verify: Runs 100 iterations without panic; checkpoint written; solver accessible during training (no deadlock); exploitability logged
  - Integration: `Trigger=after registration; Callsite=main.rs spawns task; State=solver improves; Persistence=checkpoints; Signal=log "Epoch {n}, exploit: {x}"`
  - Rollback: RwLock contention makes query latency > 2s

- [ ] **MN-03** — HTTP Axon Server
  - Where: `crates/myosu-miner/src/axon.rs (new)`
  - Tests: `cargo test -p myosu-miner axon::tests::health_endpoint`
  - Blocking: Validators query this endpoint — no axon = invisible miner
  - Verify: GET /health returns 200; POST /strategy with valid query → response; invalid bytes → 400; handles 100 concurrent requests
  - Integration: `Trigger=after registration; Callsite=main.rs spawns server; State=HTTP listening; Persistence=N/A; Signal=curl /health responds`
  - Rollback: HTTP framework conflicts with tokio training runtime

- [ ] **MN-04** — On-Chain Axon Advertisement
  - Where: `crates/myosu-miner/src/chain.rs (extend)`
  - Tests: `cargo test -p myosu-miner chain::tests::serve_axon_success`
  - Blocking: Without advertisement, validators don't know where to query
  - Verify: serve_axon extrinsic succeeds; axon discoverable via RPC query
  - Integration: `Trigger=axon starts listening; Callsite=main.rs; State=AxonInfo on chain; Persistence=on-chain storage; Signal=AxonServed event`
  - Rollback: IP detection fails in container environments

- [ ] **MN-05** — Graceful Shutdown and Resume
  - Where: `crates/myosu-miner/src/main.rs (extend)`
  - Tests: `cargo test -p myosu-miner main::tests::graceful_shutdown`
  - Blocking: Production miners need restart resilience
  - Verify: SIGINT → clean exit within 5s; checkpoint exists; restart loads checkpoint; no corruption
  - Integration: `Trigger=SIGINT/SIGTERM; Callsite=tokio signal handler; State=training stops; Persistence=final checkpoint; Signal=log "Shutdown complete"`
  - Rollback: checkpoint format changes between versions

---

## Stage 4b: Validator Oracle
Source spec: specs/031626-04b-validator-oracle.md

- [ ] **VO-01** — CLI and Chain Registration
  - Where: `crates/myosu-validator/src/main.rs (new)`, `src/chain.rs (new)`
  - Tests: `cargo test -p myosu-validator chain::tests::register_and_stake`
  - Blocking: Validators must be registered and staked to submit weights
  - Verify: Registers on subnet; stakes tokens; has ValidatorPermit after epoch
  - Integration: `Trigger=myosu-validator CLI; Callsite=main.rs; State=registered and staked; Persistence=on-chain; Signal=ValidatorPermit active`
  - Rollback: ValidatorPermit requires more stake than available

- [ ] **VO-02** — Miner Discovery and Query
  - Where: `crates/myosu-validator/src/evaluator.rs (new)`
  - Tests: `cargo test -p myosu-validator evaluator::tests::discover_miners`
  - Blocking: Without miner queries, validator has nothing to score
  - Verify: Discovers all miners with axons; queries strategy; timeout → score 0; invalid response → score 0
  - Integration: `Trigger=evaluation timer; Callsite=evaluator.rs; State=responses collected; Persistence=N/A; Signal=log "Queried {n} miners"`
  - Rollback: miner axon protocol incompatible

- [ ] **VO-03** — Deterministic Test Position Generation
  - Where: `crates/myosu-validator/src/positions.rs (new)`
  - Tests: `cargo test -p myosu-validator positions::tests::deterministic_same_seed`
  - Blocking: Deterministic positions required for INV-003
  - Verify: Same seed → identical positions; different seeds → different; covers all streets; all positions valid
  - Integration: `Trigger=evaluation start; Callsite=evaluator.rs; State=position set generated; Persistence=N/A; Signal=positions logged`
  - Rollback: encoder requires pre-loaded abstractions not available

- [ ] **VO-04** — Exploitability Scoring
  - Where: `crates/myosu-validator/src/scoring.rs (new)`
  - Tests: `cargo test -p myosu-validator scoring::tests::nash_strategy_max_weight`
  - Blocking: Scoring function is Yuma's input — bad scores → bad incentives
  - Verify: Nash-like → weight ~65535; random → weight ~0; unresponsive → 0; deterministic (INV-003)
  - Integration: `Trigger=after miner queries; Callsite=evaluator.rs; State=scores computed; Persistence=N/A; Signal=scores logged`
  - Rollback: exploitability too slow for per-epoch evaluation

- [ ] **VO-05** — Weight Submission (Direct and Commit-Reveal)
  - Where: `crates/myosu-validator/src/submitter.rs (new)`
  - Tests: `cargo test -p myosu-validator submitter::tests::submit_direct_weights`
  - Blocking: Without weight submission, Yuma has no input
  - Verify: Direct submission succeeds; commit-reveal flow succeeds; weights are u16; empty subnet → no submission
  - Integration: `Trigger=after scoring; Callsite=submitter.rs; State=weights on chain; Persistence=on-chain; Signal=WeightsSet event`
  - Rollback: commit-reveal timing window misaligned

- [ ] **VO-06** — Evaluation Loop Orchestration
  - Where: `crates/myosu-validator/src/main.rs (extend)`
  - Tests: `cargo test -p myosu-validator main::tests::evaluation_loop_completes`
  - Blocking: Orchestration ties all pieces together
  - Verify: Full cycle completes within tempo; failed queries don't block; graceful shutdown
  - Integration: `Trigger=block polling; Callsite=main.rs; State=weights submitted per tempo; Persistence=on-chain; Signal=log "Submitted weights for {n} miners"`
  - Rollback: evaluation takes longer than tempo period

---

## Stage 5: Product Layer
Source spec: specs/031626-05-gameplay-cli.md

- [ ] **GP-01** — Best Miner Discovery
  - Where: `crates/myosu-play/src/discovery.rs (new)`
  - Tests: `cargo test -p myosu-play discovery::tests::finds_best_miner`
  - Blocking: Players should face the strongest bot available
  - Verify: Returns miner with highest incentive; unreachable → fallback; no miners → random bot
  - Integration: `Trigger=game session start; Callsite=main.rs; State=best miner selected; Persistence=N/A; Signal=log "Connected to miner UID {n}"`
  - Rollback: incentive scores not populated before first epoch

- [ ] **GP-02** — Interactive Game Loop
  - Where: `crates/myosu-play/src/game_loop.rs (new)`
  - Tests: `cargo test -p myosu-play game_loop::tests::hand_completes_showdown`
  - Blocking: This is the user-facing product
  - Verify: Fold → hand ends; showdown → best hand wins; invalid input reprompts; all-in resolves; stats track correctly
  - Integration: `Trigger=myosu-play CLI; Callsite=main.rs; State=game state via robopoker; Persistence=N/A; Signal=hand result displayed`
  - Rollback: robopoker Game can't represent states needed for display

- [ ] **GP-03** — Bot Strategy Integration
  - Where: `crates/myosu-play/src/bot.rs (new)`
  - Tests: `cargo test -p myosu-play bot::tests::query_and_sample_action`
  - Blocking: The bot is what makes the game challenging
  - Verify: Bot queries miner and returns legal action; timeout → random; sampled action always legal; zero-prob actions never sampled
  - Integration: `Trigger=bot's turn; Callsite=game_loop.rs; State=action sampled; Persistence=N/A; Signal=bot action displayed`
  - Rollback: miner query latency > 500ms

- [ ] **GP-04** — Hand History Recording
  - Where: `crates/myosu-play/src/recorder.rs (new)`
  - Tests: `cargo test -p myosu-play recorder::tests::record_hand`
  - Blocking: Essential for player review and debugging
  - Verify: JSON file created; all actions recorded in order; session stats correct; disk full → warning, continue
  - Integration: `Trigger=hand completes; Callsite=game_loop.rs; State=N/A; Persistence=hands/*.json; Signal=file created`
  - Rollback: N/A — failures don't block gameplay

---

## Stage 6: Multi-Game Validation
Source spec: specs/031626-06-multi-game-architecture.md

- [ ] **MG-01** — Liar's Dice Game Engine
  - Where: `crates/myosu-games-liars-dice/ (new)`
  - Tests: `cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game`
  - Blocking: Architectural proof — if Liar's Dice can't implement CfrGame, multi-game claim is false
  - Verify: Root is chance node; legal bids increase; challenge resolves; payoff is zero-sum; all trait bounds satisfied
  - Integration: `Trigger=solver creates game; Callsite=training loop; State=game transitions; Persistence=N/A; Signal=tests pass`
  - Rollback: CfrGame: Copy impossible for variable-length bid history

- [ ] **MG-02** — Liar's Dice Solver and Nash Verification
  - Where: `crates/myosu-games-liars-dice/ (extend)`
  - Tests: `cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero`
  - Blocking: Exact Nash verification is strongest possible proof of trait system correctness
  - Verify: 10K iterations → exploit < 0.001; strategy is mixed; probabilities valid; wire serialization works
  - Integration: `Trigger=test harness; Callsite=test suite; State=solver converges; Persistence=N/A; Signal=exploit assertion passes`
  - Rollback: convergence requires > 100K iterations (impl error)

- [ ] **MG-03** — Zero-Change Verification
  - Where: `crates/myosu-games-liars-dice/tests/ (new)`
  - Tests: `cargo test -p myosu-games && cargo test -p myosu-games-poker`
  - Blocking: The zero-change property IS the architectural claim
  - Verify: All existing tests pass without modification; no diff in existing crate sources
  - Integration: `Trigger=cargo test; Callsite=CI; State=N/A; Persistence=N/A; Signal=all tests green`
  - Rollback: existing traits need modification for Liar's Dice

- [ ] **MG-04** — Multi-Game Expansion Guide
  - Where: `docs/multi-game-expansion.md (new)`
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
  - Tests: `cargo test -p myosu-tui renderer::tests::trait_is_object_safe`
  - Blocking: Every game renderer depends on this trait — must be stable first
  - Verify: Object-safe (Box<dyn GameRenderer> compiles); mock renderer works; pipe_output returns structured text; completions non-empty
  - Integration: `Trigger=compile-time; Callsite=shell.rs calls render_state(); State=N/A; Persistence=N/A; Signal=trait compiles`
  - Rollback: trait requires game-specific types

- [ ] **TU-07** — Color Theme Implementation
  - Where: `crates/myosu-tui/src/theme.rs (new)`
  - Tests: `cargo test -p myosu-tui theme::tests::all_colors_defined`
  - Blocking: Shell layout needs theme for declaration styling
  - Verify: All 8 color tokens from design.md defined; readable without color
  - Integration: `Trigger=compile-time; Callsite=shell.rs applies theme; State=N/A; Persistence=N/A; Signal=tests pass`
  - Rollback: N/A

- [ ] **TU-02** — Five-Panel Shell Layout
  - Where: `crates/myosu-tui/src/shell.rs (new)`
  - Tests: `cargo test -p myosu-tui shell::tests::layout_at_60_columns`
  - Blocking: Universal visual frame for all 20 games
  - Verify: 5 panels render at 60-120 columns; state panel min 4 lines; log scrolls; header shows game path
  - Integration: `Trigger=resize or state change; Callsite=event loop; State=frame buffer; Persistence=N/A; Signal=5 panels visible`
  - Rollback: layout constraints conflict at small terminal sizes

- [ ] **TU-04** — Readline Input with History
  - Where: `crates/myosu-tui/src/input.rs (new)`
  - Tests: `cargo test -p myosu-tui input::tests::tab_completion`
  - Blocking: Input quality determines gameplay feel
  - Verify: Type + submit works; history navigation; tab completion; Ctrl-W deletes word; /commands detected
  - Integration: `Trigger=key events; Callsite=events.rs; State=buffer, cursor, history; Persistence=N/A; Signal=characters appear`
  - Rollback: readline keybindings conflict with game keys

- [ ] **TU-03** — Event Loop and Async Updates
  - Where: `crates/myosu-tui/src/events.rs (new)`
  - Tests: `cargo test -p myosu-tui events::tests::key_event_handled`
  - Blocking: Ties shell + input + async miner queries together
  - Verify: Key press triggers re-render within 16ms; miner response updates state without blocking; Ctrl-C clean shutdown
  - Integration: `Trigger=key event or miner response; Callsite=main.rs; State=game + terminal; Persistence=N/A; Signal=responsive UI`
  - Rollback: crossterm and tokio event loops conflict

- [ ] **TU-05** — Screen State Machine
  - Where: `crates/myosu-tui/src/screens.rs (new)`
  - Tests: `cargo test -p myosu-tui screens::tests::lobby_to_game`
  - Blocking: Navigation between game states
  - Verify: Lobby→Game on subnet select; Game→Stats on /stats; /analyze→Coaching; any key returns from overlay
  - Integration: `Trigger=/commands or game completion; Callsite=event loop; State=Screen enum; Persistence=N/A; Signal=display switches`
  - Rollback: screen transitions lose game state

- [ ] **TU-06** — Pipe Mode for Agent Protocol
  - Where: `crates/myosu-tui/src/pipe.rs (new)`
  - Tests: `cargo test -p myosu-tui pipe::tests::pipe_output_no_ansi`
  - Blocking: Agent-native design depends on pipe mode
  - Verify: --pipe output has zero ANSI codes; matches design.md pipe format; stdin accepted; agent plays complete hand
  - Integration: `Trigger=--pipe flag; Callsite=main.rs; State=stdin/stdout; Persistence=hand history; Signal=structured text output`
  - Rollback: pipe vs TUI rendering diverges

---

## Stage 8: Abstraction Pipeline
Source spec: specs/031626-08-abstraction-pipeline.md

- [ ] **AP-01** — Clustering Binary
  - Where: `crates/myosu-cluster/src/main.rs (new)`
  - Tests: `cargo test -p myosu-cluster cluster::tests::preflop_produces_169_entries`
  - Blocking: Without abstraction tables, miners produce random strategies
  - Verify: Preflop produces 169 entries; all 4 files written; manifest SHA-256 matches; deterministic re-run
  - Integration: `Trigger=myosu-cluster CLI; Callsite=main.rs; State=clustering state; Persistence=4 bin files + manifest.json; Signal=manifest with hashes`
  - Rollback: robopoker clustering API not accessible outside rbp-autotrain

- [ ] **AP-02** — File-Based Encoder Loading
  - Where: `happybigmtn/robopoker rbp-nlhe/src/encoder.rs (extend)`
  - Tests: `cargo test -p rbp-nlhe encoder::tests::from_dir_loads_all_streets`
  - Blocking: Part of RF-02 — miners load encoder without PostgreSQL
  - Verify: from_dir loads all 4 streets; abstraction() returns valid values; tampered file rejected; hash deterministic
  - Integration: `Trigger=miner startup; Callsite=PokerSolver::new(); State=encoder populated; Persistence=read-only; Signal=hash logged`
  - Rollback: 138M entries don't fit in memory

- [ ] **AP-03** — Pre-Computed Artifact Distribution
  - Where: `artifacts/abstractions/ (new)`, `scripts/download-abstractions.sh (new)`
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
  - Tests: `docker compose up -d && sleep 30 && curl -sf http://localhost:8080/health`
  - Blocking: Developers and testers need one command to run the full stack
  - Verify: Chain produces blocks within 10s; miner registers within 30s; validator submits within first tempo; curl /health responds; compose down cleans up
  - Integration: `Trigger=docker compose up; Callsite=ops/devnet/; State=full stack running; Persistence=volumes; Signal=all services healthy`
  - Rollback: Docker build times exceed 30 minutes

- [ ] **LI-02** — Miner Bootstrap Sequence
  - Where: `crates/myosu-miner/src/bootstrap.rs (new)`, `scripts/miner-bootstrap.sh (new)`
  - Tests: `cargo test -p myosu-miner bootstrap::tests::full_sequence_on_devnet`
  - Blocking: New miners must go from zero to operational automatically
  - Verify: Fresh miner downloads abstractions, registers, starts training; resume from checkpoint works; chain unreachable → retry with backoff; subnet not found → clear error
  - Integration: `Trigger=miner startup; Callsite=main.rs; State=10-step bootstrap; Persistence=abstractions + checkpoint; Signal=log at each step`
  - Rollback: abstraction download fails in Docker network

- [ ] **LI-03** — Gameplay ↔ TUI Wiring
  - Where: `crates/myosu-play/src/main.rs (extend)`, `crates/myosu-games-poker/src/renderer.rs (new)`
  - Tests: `cargo test -p myosu-play integration::tests::one_hand_in_tui`
  - Blocking: Gameplay must render through the TUI shell, not ad-hoc CLI prompts
  - Verify: NlheRenderer implements GameRenderer; TUI matches design.md 8.1; bot actions in log; /stats works; --pipe plays one hand
  - Integration: `Trigger=myosu-play CLI; Callsite=main.rs creates TUI shell with NlheRenderer; State=game + TUI; Persistence=hand history; Signal=design.md screen rendered`
  - Rollback: GameRenderer trait can't express NLHE state panel

- [ ] **LI-04** — End-to-End Acceptance Test
  - Where: `tests/e2e/nlhe_launch.rs (new)`
  - Tests: `cargo test --test nlhe_launch -- --ignored` (long-running, opt-in)
  - Blocking: If this test passes, we can launch. If it fails, we can't.
  - Verify: Chain → subnet → miner → train → validator → Yuma → emissions → gameplay → hand history. All 8 steps pass. Total < 120 seconds. Uses preflop-only abstractions for speed.
  - Integration: `Trigger=cargo test; Callsite=tests/e2e/; State=full stack lifecycle; Persistence=temp; Signal=all assertions pass`
  - Rollback: any component integration failure

- [ ] **LI-05** — Launch Readiness Checklist
  - Where: `docs/launch-checklist.md (new)`
  - Tests: N/A (documentation + manual verification)
  - Blocking: Prevents declaring launch before all critical paths work
  - Verify: All checklist items checkable; covers chain, solver, scoring, gameplay, integration; explicitly lists what's NOT required
  - Integration: `N/A (documentation)`
  - Rollback: N/A

---

## Stage 10: Agent Integration
Source spec: specs/031626-10-agent-experience.md

- [ ] **AX-01** — Game State JSON Schema
  - Where: `crates/myosu-tui/src/schema.rs (new)`, `docs/api/game-state.json (new)`
  - Tests: `cargo test -p myosu-tui schema::tests::nlhe_state_serializes`
  - Blocking: Any agent in any language needs machine-readable game state
  - Verify: JSON schema validates; legal_actions exhaustive (every valid action enumerated); parseable by Python/JS/Rust
  - Integration: `Trigger=game state change; Callsite=schema.rs; State=JSON output; Persistence=N/A; Signal=valid JSON`
  - Rollback: schema too rigid for non-poker games

- [ ] **AX-02** — Action JSON Schema
  - Where: `crates/myosu-tui/src/schema.rs (extend)`
  - Tests: `cargo test -p myosu-tui schema::tests::valid_action_accepted`
  - Blocking: Agents need a structured way to submit actions with error recovery
  - Verify: Valid action accepted; invalid returns 400 with legal_actions; all action types roundtrip
  - Integration: `Trigger=agent submits action; Callsite=api.rs; State=game updated; Persistence=N/A; Signal=updated state returned`
  - Rollback: action format too complex for simple bots

- [ ] **AX-03** — HTTP Game API
  - Where: `crates/myosu-play/src/api.rs (new)`
  - Tests: `cargo test -p myosu-play api::tests::play_one_hand`
  - Blocking: HTTP is universal — Claude Code, Python scripts, curl all use it
  - Verify: Create session; submit action; hand completes; invalid action → 400 with legal_actions; 10 concurrent sessions
  - Integration: `Trigger=HTTP request; Callsite=api.rs; State=session state server-side; Persistence=hand history; Signal=JSON response`
  - Rollback: HTTP latency too high for competitive play

- [ ] **AX-04** — WebSocket Game API
  - Where: `crates/myosu-play/src/ws.rs (new)`
  - Tests: `cargo test -p myosu-play ws::tests::connect_and_play`
  - Blocking: Persistent connections with server-push for responsive agent play
  - Verify: Connect and play one hand; spectator receives updates; reconnect preserves session
  - Integration: `Trigger=WS connection; Callsite=ws.rs; State=persistent session; Persistence=hand history; Signal=JSON frames`
  - Rollback: WS complexity not justified for launch

- [ ] **AX-05** — Python SDK
  - Where: `sdk/python/myosu/ (new)`
  - Tests: `pytest sdk/python/tests/test_client.py`
  - Blocking: Python is the LLM tool-use lingua franca — 5-line bot must be possible
  - Verify: create_session works; act() returns updated state; strategy callback plays N hands; pip install from git
  - Integration: `Trigger=import myosu; Callsite=Python process; State=HTTP session; Persistence=N/A; Signal=game.result populated`
  - Rollback: SDK maintenance burden too high for small team

- [ ] **AX-06** — Bot Registration (Bring Your Own Strategy)
  - Where: `crates/myosu-play/src/api.rs (extend)`
  - Tests: `cargo test -p myosu-play api::tests::bot_vs_bot_session`
  - Blocking: Agents need to compete against each other and against the solver
  - Verify: bot-vs-solver mode works; bot-vs-bot mode works (two API clients, myosu hosts engine); spectate mode works
  - Integration: `Trigger=POST /sessions with mode; Callsite=api.rs; State=game with two API players; Persistence=hand history; Signal=both players receive state`
  - Rollback: multi-player API sessions too complex for launch
