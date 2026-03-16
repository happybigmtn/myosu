# Implementation Plan

Status: Active
Date: 2026-03-16

## Spec Index

| Spec File | AC Prefix | Count |
|-----------|-----------|-------|
| 031626-myosu-game-solving-chain.md | CH-01..02, GE-01..02, MN-01, VL-01, GP-01, FG-01 | 8 |

---

## Stage 1: Chain Foundation
Source spec: specs/031626-myosu-game-solving-chain.md

- [ ] **CH-01** — Substrate Chain Fork Scaffold
  - Where: `crates/myosu-chain/ (new)`
  - Tests: `cargo test -p myosu-chain chain::tests::devnet_produces_blocks`
  - Blocking: Every other AC depends on a running chain
  - Verify: `myosu-node --dev` produces blocks; RPC responds to `system_health`; genesis accounts funded; clean shutdown on SIGTERM
  - Integration: `Trigger=myosu-node --dev CLI; Callsite=crates/myosu-chain/node/src/main.rs; State=blocks produced every 6s; Persistence=block database in data dir; Signal=RPC system_health responds`
  - Rollback: Chain fails to produce blocks or panics on startup

---

## Stage 2: Game Engine
Source spec: specs/031626-myosu-game-solving-chain.md

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
Source spec: specs/031626-myosu-game-solving-chain.md

- [ ] **CH-02** — Game-Solving Pallet Core
  - Where: `crates/myosu-chain/pallets/game-solver/ (new)`
  - Tests: `cargo test -p pallet-game-solver pallet::tests::create_subnet`
  - Blocking: The on-chain incentive engine — miners/validators are useless without it
  - Verify: create_subnet returns subnet_id; register_neuron assigns uid; set_weights + tempo → emissions distributed; Yuma clips at consensus median; bond EMA accumulates; commit-reveal flow works; full subnet prunes lowest scorer
  - Integration: `Trigger=on_initialize every block; Callsite=pallets/game-solver/src/lib.rs::on_initialize(); State=Yuma updates bonds/emissions at tempo; Persistence=on-chain storage maps; Signal=Emission event per tempo`
  - Rollback: Yuma math diverges from subtensor for identical inputs, or emission accounting creates/loses tokens

---

## Stage 4: Off-Chain Participants
Source spec: specs/031626-myosu-game-solving-chain.md

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
Source spec: specs/031626-myosu-game-solving-chain.md

- [ ] **GP-01** — Human vs Bot Gameplay CLI
  - Where: `crates/myosu-play/ (new)`
  - Tests: `cargo test -p myosu-play play::tests::game_loop_completes`
  - Blocking: Gameplay is the consumer-facing product — without it, myosu is academic
  - Verify: Human can fold/check/call/raise/shove via CLI; bot actions are legal; hand completes with correct pot award; hand history recorded; miner disconnect → fallback to random
  - Integration: `Trigger=myosu-play CLI launch; Callsite=crates/myosu-play/src/main.rs; State=local game, miner queries; Persistence=hand history saved locally; Signal=complete hand in terminal`
  - Rollback: Miner query latency > 2s makes gameplay unplayable

---

## Stage 6: Multi-Game Validation
Source spec: specs/031626-myosu-game-solving-chain.md

- [ ] **FG-01** — Multi-Game Subnet Architecture
  - Where: `crates/myosu-games/ (extend)`, `docs/multi-game-expansion.md (new)`
  - Tests: `cargo test -p myosu-games games::tests::liar_dice_game_lifecycle`
  - Blocking: Proves architecture is multi-game, not poker-only — the differentiator
  - Verify: Liar's Dice implements GameEngine fully; game runs start→finish; exploitability computable; architecture doc covers backgammon, mahjong, bridge, PLO, short deck
  - Integration: `Trigger=N/A (architecture proof); Callsite=N/A; State=N/A; Persistence=docs/multi-game-expansion.md; Signal=Liar's Dice tests pass`
  - Rollback: GameEngine trait cannot express non-poker information structure
