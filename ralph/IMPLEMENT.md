# Implementation Plan

Status: Active
Date: 2026-03-16

## Spec Index

| Spec File | AC Prefix | Count |
|-----------|-----------|-------|
| 031626-myosu-game-solving-chain.md | (master index) | — |
| 031626-chain-fork-scaffold.md | CF-01..05 | 5 |
| 031626-game-solving-pallet.md | GS-01..09 | 9 |
| 031626-game-engine-traits.md | GT-01..nn | Planned |
| 031626-poker-engine.md | PE-01..nn | Planned |
| 031626-miner-binary.md | MN-01..nn | Planned |
| 031626-validator-oracle.md | VO-01..nn | Planned |
| 031626-gameplay-cli.md | GP-01..nn | Planned |
| 031626-multi-game-architecture.md | MG-01..nn | Planned |

---

## Stage 1: Chain Fork Scaffold
Source spec: specs/031626-chain-fork-scaffold.md

- [ ] **CF-02** — Prune Workspace Dependencies
  - Where: `crates/myosu-chain/Cargo.toml (new)`, `crates/myosu-chain/runtime/Cargo.toml (new)`
  - Tests: `cargo tree -p myosu-runtime 2>&1 | grep -c "pallet.subtensor"` → 0
  - Blocking: Dependency contamination from stripped pallets will cause build failures
  - Verify: No references to subtensor/frontier/drand in dependency tree; build succeeds; WASM blob produced
  - Integration: `Trigger=cargo build; Callsite=Cargo resolver; State=dependency tree resolved; Persistence=Cargo.lock; Signal=cargo tree shows no stripped crates`
  - Rollback: Transitive dependency on stripped pallet that can't be resolved

- [ ] **CF-01** — Strip AI/EVM Pallets from Runtime
  - Where: `crates/myosu-chain/runtime/src/lib.rs (new, from subtensor)`
  - Tests: `cargo test -p myosu-runtime runtime::tests::runtime_compiles`
  - Blocking: Everything downstream depends on a compilable runtime — single most critical unblock
  - Verify: Runtime compiles; spec_name="myosu"; 13 pallets in construct_runtime!; no subtensor/frontier/EVM references; WASM < 5MB
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

## Stage 2: Game Engine
Source spec: specs/031626-game-engine-traits.md (planned)

- [ ] **GE-01** — Game Engine Trait Abstraction
  - Where: `crates/myosu-games/src/lib.rs (new)`
  - Tests: `cargo test -p myosu-games games::tests::trait_object_safety`
  - Blocking: Abstraction all downstream crates depend on — defines the multi-game interface
  - Verify: Traits are object-safe; mock coin-flip game implements all traits; StrategyProfile expresses mixed strategies; Exploitability returns 0.0 for Nash
  - Integration: `Trigger=compile-time; Callsite=myosu-miner, myosu-validator, myosu-play depend on this; State=N/A (trait defs); Persistence=N/A; Signal=cargo test -p myosu-games passes`
  - Rollback: Traits not object-safe or cannot express poker's information structure

- [ ] **GE-02** — Poker Engine Wrapping Robopoker
  - Where: `crates/myosu-games/poker/ (new)`
  - Tests: `cargo test -p myosu-games poker::tests::nlhe_game_lifecycle`
  - Blocking: Bridge between robopoker solver output and the chain's incentive mechanism
  - Verify: NLHE HU game runs deal→showdown; legal actions match robopoker Edge variants; info sets distinct per player; random strategy exploitability > 0; serialized strategy round-trips
  - Integration: `Trigger=miner/validator create NlheEngine; Callsite=myosu-miner/src/main.rs, myosu-validator/src/oracle/; State=game state transitions; Persistence=strategy profiles serialized; Signal=exploitability score computed`
  - Rollback: robopoker v1.0.0 API changes or types unmappable to trait interface

---

## Stage 3: On-Chain Incentives
Source spec: specs/031626-game-solving-pallet.md

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
  - Where: `crates/myosu-chain/runtime/src/lib.rs (extend)`
  - Tests: `cargo test -p myosu-chain integration::tests::full_incentive_loop`
  - Blocking: Integration gate — until pallet is in runtime, it's just library code
  - Verify: Runtime compiles with pallet; index 7 occupied; create_subnet callable; full loop works (create→register→stake→weights→epoch→emission); no CF-05 regression
  - Integration: `Trigger=runtime compilation; Callsite=runtime/src/lib.rs; State=pallet in block execution; Persistence=pallet storage in chain state; Signal=create_subnet callable via RPC`
  - Rollback: Config requirements conflict with existing runtime

---

## Stage 4: Off-Chain Participants
Source spec: specs/031626-miner-binary.md, specs/031626-validator-oracle.md (planned)

- [ ] **MN-01** — Miner Binary with MCCFR Training
  - Where: `crates/myosu-miner/ (new)`
  - Tests: `cargo test -p myosu-miner miner::tests::register_on_devnet`
  - Blocking: No miner = no supply — validators have nothing to score
  - Verify: Registers on subnet, receives UID; axon serves strategy queries; 100 MCCFR iterations produce non-uniform strategy; checkpoint saved/loadable; graceful shutdown
  - Integration: `Trigger=myosu-miner CLI launch; Callsite=crates/myosu-miner/src/main.rs; State=neuron registered, training running; Persistence=strategy checkpoints on disk; Signal=axon responds to queries`
  - Rollback: robopoker training API changes or axon protocol mismatches

- [ ] **VL-01** — Validator Binary with Exploitability Oracle
  - Where: `crates/myosu-validator/ (new)`
  - Tests: `cargo test -p myosu-validator validator::tests::compute_exploitability`
  - Blocking: Without validators, Yuma has no input — chain distributes zero emissions
  - Verify: Registers with validator permit; queries miner axon; exploitability > 0 for random strategy; weight vector submitted via RPC; commit-reveal flow; garbage miner gets weight 0
  - Integration: `Trigger=myosu-validator CLI launch + periodic timer; Callsite=crates/myosu-validator/src/main.rs; State=weight vector submitted per tempo; Persistence=weights stored on-chain; Signal=weights visible via chain RPC`
  - Rollback: Exploitability non-deterministic (INV-003) or too expensive for tempo period

---

## Stage 5: Product Layer
Source spec: specs/031626-gameplay-cli.md (planned)

- [ ] **GP-01** — Human vs Bot Gameplay CLI
  - Where: `crates/myosu-play/ (new)`
  - Tests: `cargo test -p myosu-play play::tests::game_loop_completes`
  - Blocking: Gameplay is the consumer-facing product — without it, myosu is academic
  - Verify: Human can fold/check/call/raise/shove via CLI; bot actions are legal; hand completes with correct pot award; hand history recorded; miner disconnect → fallback to random
  - Integration: `Trigger=myosu-play CLI launch; Callsite=crates/myosu-play/src/main.rs; State=local game, miner queries; Persistence=hand history saved locally; Signal=complete hand in terminal`
  - Rollback: Miner query latency > 2s makes gameplay unplayable

---

## Stage 6: Multi-Game Validation
Source spec: specs/031626-multi-game-architecture.md (planned)

- [ ] **FG-01** — Multi-Game Subnet Architecture
  - Where: `crates/myosu-games/ (extend)`, `docs/multi-game-expansion.md (new)`
  - Tests: `cargo test -p myosu-games games::tests::liar_dice_game_lifecycle`
  - Blocking: Proves architecture is multi-game, not poker-only — the differentiator
  - Verify: Liar's Dice implements GameEngine fully; game runs start→finish; exploitability computable; architecture doc covers backgammon, mahjong, bridge, PLO, short deck
  - Integration: `Trigger=N/A (architecture proof); Callsite=N/A; State=N/A; Persistence=docs/multi-game-expansion.md; Signal=Liar's Dice tests pass`
  - Rollback: GameEngine trait cannot express non-poker information structure
