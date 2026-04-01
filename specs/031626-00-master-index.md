# Specification: Myosu Game-Solving Chain — Master Index

Source: Research analysis of opentensor/subtensor architecture + codexpoker solver infrastructure
Status: Active (index document — individual specs are Draft)
Date: 2026-03-30
Depends-on: none

> **This document is a master index.** Each original AC has been expanded into
> its own spec with multiple detailed ACs. The sections below preserve the
> original high-level design. See the Spec Index for detailed build contracts.

## Spec Index

| Spec File | Expands | AC Prefix | Status |
|-----------|---------|-----------|--------|
| `031626-01-chain-fork-scaffold.md` | CH-01 | CF-01..11 | Draft |
| `031626-03-game-solving-pallet.md` | CH-02 | GS-01..10 | Draft |
| `031626-02a-game-engine-traits.md` | GE-01 | GT-01..05 | Draft |
| `031626-02b-poker-engine.md` | GE-02 | PE-01..04 | Draft |
| `031626-04a-miner-binary.md` | MN-01 | MN-01..05 | Draft |
| `031626-04b-validator-oracle.md` | VL-01 | VO-01..07 | Draft |
| `031626-05-gameplay-cli.md` | GP-01 | GP-01..04 | Draft |
| `031626-06-multi-game-architecture.md` | FG-01 | MG-01..04 | Draft |
| `031626-07-tui-implementation.md` | (new) | TU-01..12 | Draft |
| `031626-08-abstraction-pipeline.md` | (new) | AP-01..03 | Draft |
| `031626-09-launch-integration.md` | (new) | LI-01..06 | Draft |
| `031626-10-agent-experience.md` | (new) | AX-01..06 | Draft |
| `031626-11-agent-coordination-mechanism.md` | (new) | AC (design) | Superseded by 12 |
| `031626-12-nlhe-incentive-mechanism.md` | (new) | IL (design) | Draft |
| `031626-13-n-player-trait-design.md` | (new) | — (decision) | Draft |
| `031626-14-poker-variant-family.md` | (new) | PV-01..06 | Draft |
| `031626-15-key-management.md` | (new) | KM-01..04 | Draft |
| `031626-16-cross-game-scoring.md` | (new) | CS-01..03 | Draft |
| `031626-17-spectator-protocol.md` | (new) | SP-01..03 | Draft |
| `031626-18-operational-rpcs.md` | (new) | OR-01..04 | Draft |
| `031626-19-game-engine-sdk.md` | (new) | SDK-01..05 | Draft |

## Purpose

Myosu is a Substrate-based blockchain forked from Bittensor's subtensor that
creates **decentralized game-solving markets**. Each subnet represents a game
variant where miners compete to produce the highest-quality strategies (via
MCCFR and other regret minimization algorithms) and validators measure strategy
quality through exploitability scoring. Yuma Consensus aggregates validator
assessments and distributes emissions to the best solvers.

The platform also provides a **gameplay layer** where humans play against the
trained strategies — turning solver quality into a directly consumable product.
The first subnets target No-Limit Hold'em poker variants, with expansion into
backgammon, mahjong, bridge, and other imperfect-information games.

**Key design constraint**: the solver layer and gameplay layer share game engine
code but never share runtime state or trust boundaries (INV-004).

Robopoker v1.0.0 (https://github.com/krukah/robopoker) serves as the core
MCCFR engine for all poker subnets.

## Whole-System Goal

Original design-time starting point:
- The repo began before any in-repo chain implementation existed
- Robopoker v1.0.0 provides a production MCCFR poker solver with clustering,
  abstraction, and blueprint training
- Codexpoker has a gameplay layer (TUI, P2P, fairness crypto) that demonstrates
  the pattern but is not integrated with a solver incentive chain
- Bittensor's subtensor provides a proven Substrate runtime for subnet-based
  incentive markets but is coupled to AI/ML workloads

This spec adds:
- A forked Substrate chain with game-solving pallets replacing Bittensor's AI-specific mechanics
- A solver incentive layer where miners run MCCFR and validators score exploitability
- A gameplay layer where humans play against the best available strategies
- Subnet registration for different game variants

If all ACs land:
- A local devnet produces blocks with game-solving pallets
- One poker subnet runs with miners producing strategies and validators scoring them
- Humans can play poker against trained strategies via CLI
- The architecture supports adding new game subnets without chain changes

Still not solved here:
- Production deployment, mainnet launch, token economics tuning
- Mental poker (ZK shuffle, threshold encryption) for P2P play
- Advanced game subnets beyond NLHE (backgammon, mahjong, bridge)
- Cross-subnet strategy transfer or meta-learning (provenance tracking designed in spec 12, transfer credits deferred to spec 16)
- Frontend/web UI — CLI only for now

12-month direction:
- 5+ game subnets running concurrently (NLHE HU, 6-max, PLO, backgammon, mahjong)
- Hundreds of miners competing per subnet
- Web-based gameplay interface
- Strategy marketplace where players purchase coaching subscriptions
- Bridge to existing poker platforms for strategy consumption

## Why This Spec Exists As One Unit

- The chain fork, solver integration, validator oracle, and gameplay layer are
  the minimum coherent vertical slice — each is useless without the others
- A chain without solvers has no miners. Solvers without validators have no
  incentives. Validators without gameplay have no product.
- The four layers share the game engine crate, making them a natural unit
- Splitting would create specs that cannot be independently validated

## Scope

In scope:
- Substrate chain fork from subtensor with game-solving pallets
- Poker game engine crate wrapping robopoker v1.0.0
- Miner binary that runs MCCFR training and serves strategy queries
- Validator binary that computes exploitability scores and submits weights
- Gameplay binary where humans play against trained strategies (CLI)
- Subnet registration for NLHE heads-up as the first variant
- Local devnet for development and testing

Out of scope:
- Production deployment and mainnet launch — requires tokenomics design
- Non-poker game engine implementations — architecture supports them, all 20
  games have TUI mockups in DESIGN.md, but only NLHE ships in Phase 0
- Web UI — TUI addressed by spec 031626-07 (TU-01..12) and integrated via LI-03
- P2P multiplayer between humans — single-player vs bot only
- Mental poker cryptography — not needed for human vs bot
- Bridge/deposit contract to external chains
- Token economics beyond basic emission mechanics

Design-complete but not yet implemented:
- Onboarding flow (DESIGN.md 8.0a-8.0c) — first-run account creation, seed
  backup, network selection. Designed for keyboard-only, agent-compatible input.
- Wallet screen (DESIGN.md 8.0d) — balance, staking, session overview.
- 19 non-NLHE game screens (DESIGN.md 9.2-9.20) — full TUI mockups exist for
  all 20 games from OS.md. Each requires a `GameRenderer` implementation when
  the corresponding game engine is built.
- Spectator mode (DESIGN.md 9.24) — watch agent vs agent play.

## Current State

- The repo is no longer greenfield. A local stage-0 loop exists here: the
  stripped chain produces blocks, `myosu-miner` trains and serves bounded
  strategy, `myosu-validator` scores and submits weights, and `myosu-play`
  consumes both artifact-backed and live miner advice.
- The additive second-game seam is also proven locally. Liar's Dice has its
  own game crate and participates in the owned two-subnet coexistence proof.
- `robopoker` v1.0.0 and `subtensor` remain the upstream baselines, but this
  repo now carries stage-0 reductions, wrappers, and proof harnesses on top of
  them rather than just planning against them.
- Hosted CI proof exists on draft PR `#1`: run `23741634642` closed plan `010`
  with green `Stage-0 Repo Shape`, doctrine, gameplay, and chain jobs under
  the `<15 min` timing target.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Shared game seam | `crates/myosu-games/` | extend | Live trait and registry layer for additive games |
| Poker solver + renderer | `crates/myosu-games-poker/` | extend | Artifact-backed poker engine, wire types, and renderer are already live |
| Second-game proof | `crates/myosu-games-liars-dice/` | extend | Additive non-poker game crate already proven through local play and chain coexistence |
| Chain runtime + node + pallet | `crates/myosu-chain/` | reduce + extend | Local devnet, owned smoke path, and stage-0 pallet loop already exist |
| Shared chain client | `crates/myosu-chain-client/` | use | RPC, storage, and signed extrinsic seam shared by miner, validator, and gameplay |
| Miner service | `crates/myosu-miner/` | extend | Bounded training, checkpoint loading, and HTTP axon are already live |
| Validator service | `crates/myosu-validator/` | extend | Deterministic scoring and weight submission are already live |
| Gameplay surface | `crates/myosu-play/` + `crates/myosu-tui/` | extend | Training shell, pipe mode, smoke path, and live miner discovery/query are already live |
| Upstream baselines | `robopoker`, `subtensor`, `codexpoker` | reference / fork | Provide the provenance and inherited primitives that the current repo has now reduced and integrated |

## Non-goals

- Token price optimization or DeFi composability — solve the game first
- GPU-accelerated training — CPU MCCFR is the baseline, GPU is optimization
- Real-money gameplay — play tokens only during bootstrap
- Consensus innovation — standard BABE+GRANDPA from Substrate is sufficient
- Cross-chain bridges — single chain for now
- Mobile or web clients — CLI is the bootstrap interface

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Game engine (poker) | Live | crates/myosu-games-poker/ |
| Game engine traits | Live | crates/myosu-games/src/lib.rs |
| Chain runtime | Live stage-0 fork | crates/myosu-chain/ |
| Game-solving pallet | Live stage-0 pallet | crates/myosu-chain/pallets/game-solver/ |
| Miner binary | Live | crates/myosu-miner/ |
| Validator binary | Live | crates/myosu-validator/ |
| Gameplay CLI | Live | crates/myosu-play/ |
| Exploitability scoring | Live | crates/myosu-validator/src/validation.rs |

## Architecture / Runtime Contract

```
                    ┌─────────────────────────────────┐
                    │      Myosu Chain (Substrate)     │
                    │                                  │
                    │  ┌──────────────────────────┐   │
                    │  │  pallet_game_solver       │   │
                    │  │  - subnet registry        │   │
                    │  │  - neuron UIDs            │   │
                    │  │  - weight storage         │   │
                    │  │  - Yuma Consensus         │   │
                    │  │  - emission distribution  │   │
                    │  └──────────────────────────┘   │
                    └──────────┬───────────┬──────────┘
                               │           │
                    ┌──────────▼──┐   ┌────▼──────────┐
                    │   Miner     │   │   Validator   │
                    │             │   │               │
                    │ robopoker   │   │ exploitability│
                    │ MCCFR       │◄──┤ oracle        │
                    │ trainer     │   │               │
                    │             │   │ submit_weights│
                    │ serve_axon  │   │ to chain      │
                    └─────────────┘   └───────────────┘
                          │
                          │ strategy queries
                          ▼
                    ┌─────────────┐
                    │  Gameplay   │
                    │  CLI        │
                    │             │
                    │  human vs   │
                    │  best miner │
                    └─────────────┘
```

Primary loop:
- Trigger: tempo epoch fires every N blocks on chain
- Source of truth: weight matrix from validators, miner strategy profiles
- Processing: Yuma Consensus computes incentive/dividend scores
- Persisted truth: emission distribution, bond EMA, neuron scores
- Consumer: miners (rewards), validators (dividends), gameplay (best strategy)

Failure loop:
- Miner produces invalid strategy → validator scores zero → miner pruned
- Validator submits non-deterministic scores → Yuma clips at consensus median
- No miners registered → subnet earns zero emissions
- Chain halts → standard Substrate GRANDPA recovery

## Adoption / Consumption Path

- Producer: miners produce strategy profiles via MCCFR
- First consumer: validators evaluate strategy quality
- Operator-visible surface: CLI showing subnet scores, miner rankings, exploitability
- Why it changes behavior now: creates a competitive market for game-solving quality
- If not consumed yet: gameplay layer consumes best miner's strategy for human play

---

## A. Chain Foundation

### AC-CH-01: Substrate Chain Fork Scaffold

- Where: `crates/myosu-chain/` (new)
- How: Fork subtensor's Substrate runtime. Strip AI-specific pallets
  (pallet_subtensor's weight/epoch logic). Keep: BABE+GRANDPA consensus,
  account model, balance pallet, timestamp, identity. Create a minimal
  `node/` binary that produces blocks on a local devnet. Chain spec for
  local development with instant finality.
- Whole-system effect: provides the blockchain substrate all other components
  build on. Without a running chain, nothing else works.
- State: genesis config, chain spec, block production state.
- Wiring contract:
  - Trigger: `myosu-node --dev` CLI command
  - Callsite: `crates/myosu-chain/node/src/main.rs`
  - State effect: blocks produced every 6 seconds on devnet
  - Persistence effect: block database in local data directory
  - Observable signal: `myosu-node --dev` produces blocks, RPC responds
- Required tests:
  - `cargo test -p myosu-chain chain::tests::devnet_produces_blocks`
  - `cargo test -p myosu-chain chain::tests::rpc_responds`
  - `cargo test -p myosu-chain chain::tests::genesis_accounts_funded`
- Pass/fail:
  - `myosu-node --dev` starts and produces blocks within 10 seconds
  - RPC endpoint `http://localhost:9944` responds to `system_health`
  - Genesis accounts have initial balances
  - Chain halts cleanly on SIGTERM
- Blocking note: every other AC depends on a running chain. This is the
  foundation that unblocks all other work.
- Rollback condition: chain fails to produce blocks or panics on startup.

### AC-CH-02: Game-Solving Pallet Core

- Where: `crates/myosu-chain/pallets/game-solver/` (new)
- How: Implement `pallet_game_solver` as a FRAME pallet with:
  - Subnet registry: `create_subnet(game_type, hyperparams)` extrinsic
  - Neuron registration: `register_neuron(subnet_id)` with burn cost
  - Weight submission: `set_weights(subnet_id, weights: Vec<(u16, u16)>)`
    with commit-reveal option
  - Epoch dispatch: `on_initialize` runs Yuma Consensus every `tempo` blocks
  - Emission: distribute rewards proportional to Yuma output
  Port Yuma math from subtensor's `epoch/run_epoch.rs` + `epoch/math.rs`
  (~500 lines of fixed-point matrix operations). Replace TAO types with
  generic `Currency` trait. Keep kappa, bond EMA, consensus clip logic.
- Whole-system effect: this is the incentive engine. Without it, miners have
  no reason to produce strategies and validators have no reason to score them.
- State: `SubnetRegistry`, `NeuronRegistry[subnet][uid]`,
  `Weights[subnet][validator][miner]`, `Bonds[subnet][validator][miner]`,
  `Emissions[subnet][uid]`, `Tempo`, `Kappa`.
- Wiring contract:
  - Trigger: block production fires `on_initialize` every block
  - Callsite: `pallets/game-solver/src/lib.rs::on_initialize()`
  - State effect: at tempo boundary, compute Yuma → update bonds, emissions
  - Persistence effect: on-chain storage maps updated
  - Observable signal: `Emission` event emitted per tempo, queryable via RPC
- Required tests:
  - `cargo test -p pallet-game-solver pallet::tests::create_subnet`
  - `cargo test -p pallet-game-solver pallet::tests::register_neuron`
  - `cargo test -p pallet-game-solver pallet::tests::set_weights_and_epoch`
  - `cargo test -p pallet-game-solver pallet::tests::yuma_consensus_basic`
  - `cargo test -p pallet-game-solver pallet::tests::emission_distribution`
  - `cargo test -p pallet-game-solver pallet::tests::commit_reveal_weights`
- Pass/fail:
  - `create_subnet("nlhe_hu", default_params)` → subnet_id 1 registered
  - `register_neuron(1)` with 100 tokens burned → uid 0 assigned
  - After 3 validators set weights and tempo fires → emissions distributed
  - Yuma clips weights above consensus median (kappa=0.5)
  - Bond EMA accumulates for consistent validators
  - Commit-reveal: hash submitted → reveal accepted → weights stored
  - Duplicate subnet creation fails with `SubnetAlreadyExists`
  - Registration on full subnet prunes lowest-scoring neuron
- Blocking note: the pallet is the on-chain coordination layer. Miners and
  validators are useless without it. Port from subtensor minimizes risk.
- Rollback condition: Yuma math produces different results than subtensor for
  identical inputs, or emission accounting loses/creates tokens.

---

## B. Game Engine

### AC-GE-01: Game Engine Trait Abstraction

- Where: `crates/myosu-games/src/lib.rs` (new)
- How: Define traits that abstract over different games:
  ```rust
  pub trait GameEngine: Send + Sync {
      type State: Clone + Serialize;
      type Action: Clone + Serialize + Eq + Hash;
      type Info: Clone + Serialize + Eq + Hash;  // information set
      fn initial_state(config: &GameConfig) -> Self::State;
      fn legal_actions(state: &Self::State) -> Vec<Self::Action>;
      fn apply_action(state: &mut Self::State, action: &Self::Action);
      fn is_terminal(state: &Self::State) -> bool;
      fn information_set(state: &Self::State, player: usize) -> Self::Info;
      fn utility(state: &Self::State, player: usize) -> f64;
  }
  pub trait StrategyProfile: Send + Sync {
      type Info;
      type Action;
      fn action_probabilities(&self, info: &Self::Info) -> Vec<(Self::Action, f64)>;
  }
  pub trait Exploitability {
      type Engine: GameEngine;
      fn compute(
          strategy: &dyn StrategyProfile<
              Info = <Self::Engine as GameEngine>::Info,
              Action = <Self::Engine as GameEngine>::Action,
          >,
          config: &GameConfig,
          sample_count: usize,
      ) -> f64;  // milli-big-blinds per hand
  }
  ```
  These traits are game-agnostic. Poker, backgammon, mahjong all implement them.
- Whole-system effect: enables the validator oracle and gameplay layer to work
  with any game, not just poker. Future subnets add new `GameEngine` impls
  without touching the chain or validator code.
- State: no runtime state — pure trait definitions.
- Wiring contract:
  - Trigger: compile-time, consumed by downstream crates
  - Callsite: `myosu-miner`, `myosu-validator`, `myosu-play` depend on this
  - State effect: N/A (trait definitions)
  - Persistence effect: N/A
  - Observable signal: `cargo test -p myosu-games` passes
- Required tests:
  - `cargo test -p myosu-games games::tests::trait_object_safety`
  - `cargo test -p myosu-games games::tests::mock_game_round_trip`
- Pass/fail:
  - Traits are object-safe (can use `dyn GameEngine`)
  - A mock game (coin flip) implements all traits and runs a complete game
  - `StrategyProfile` can express mixed strategies (probability distributions)
  - `Exploitability::compute` returns 0.0 for a Nash equilibrium strategy
- Blocking note: this abstraction is what makes myosu a multi-game platform
  rather than a poker-only chain. Every downstream crate depends on it.
- Rollback condition: traits are not object-safe, or cannot express poker's
  information structure (partial observability, chance nodes).

### AC-GE-02: Poker Engine Wrapping Robopoker

- Where: `crates/myosu-games-poker/` (new)
- How: Implement `GameEngine` for No-Limit Hold'em by wrapping robopoker v1.0.0.
  Map robopoker's `Game`, `Edge`, `Turn`, `Seat` types to the trait interface.
  Wrap robopoker's `Info` → `StrategyProfile` via the `Blueprint` trait.
  Implement `Exploitability` using best-response computation: given a strategy
  profile, compute the opponent's best-response value by traversing the game
  tree. The exploitability is the sum of both players' best-response values
  (should be ≥ 0, approaching 0 at Nash equilibrium).
  Depend on robopoker via git tag: `robopoker = { git = "https://github.com/krukah/robopoker", tag = "v1.0.0" }`.
- Whole-system effect: connects robopoker's solver output to the chain's
  incentive mechanism. Without this, miners can't be scored.
- State: `NlheEngine` struct implementing `GameEngine`, `NlheExploitability`
  implementing `Exploitability`.
- Wiring contract:
  - Trigger: miner and validator create `NlheEngine` instances
  - Callsite: `crates/myosu-miner/src/training.rs`,
    `crates/myosu-validator/src/validation.rs`
  - State effect: game state transitions produce actions and utilities
  - Persistence effect: strategy profiles serialized for network transport
  - Observable signal: exploitability score computed for any strategy profile
- Required tests:
  - `cargo test -p myosu-games poker::tests::nlhe_game_lifecycle`
  - `cargo test -p myosu-games poker::tests::nlhe_legal_actions`
  - `cargo test -p myosu-games poker::tests::nlhe_information_sets`
  - `cargo test -p myosu-games poker::tests::exploitability_random_strategy`
  - `cargo test -p myosu-games poker::tests::exploitability_known_equilibrium`
- Pass/fail:
  - NLHE heads-up game runs from deal to showdown
  - Legal actions match robopoker's `Edge` variants at each decision point
  - Information sets are distinct for different player observations
  - Random strategy has exploitability > 0 (significantly)
  - Known equilibrium (if available for small game) has exploitability ≈ 0
  - Serialized strategy round-trips correctly
- Blocking note: this is the bridge between robopoker's solver and the chain's
  incentive system. The validator oracle depends on `Exploitability` working.
- Rollback condition: robopoker v1.0.0 API changes or types cannot be mapped
  to the trait interface.

---

## C. Miner

### AC-MN-01: Miner Binary with MCCFR Training

- Where: `crates/myosu-miner/` (new)
- How: Binary that:
  1. Registers as a neuron on a game-solving subnet
  2. Runs robopoker's MCCFR trainer to produce a strategy profile
  3. Serves strategy queries via an axon endpoint (HTTP/gRPC)
  4. Advertises its axon IP:port on-chain via `serve_axon` extrinsic
  The miner wraps robopoker's `Blueprint` trait implementation. Training
  runs continuously in the background, improving the strategy. The axon
  endpoint accepts game state queries and returns action distributions
  from the current best strategy.
  CLI: `myosu-miner --chain ws://localhost:9944 --subnet 1 --seed <key>`
- Whole-system effect: miners are the supply side — without them producing
  strategies, the chain has nothing to incentivize or consume.
- State: training state (regrets, policy), current best strategy profile,
  chain registration (uid, subnet_id).
- Wiring contract:
  - Trigger: `myosu-miner` CLI launch
  - Callsite: `crates/myosu-miner/src/main.rs`
  - State effect: neuron registered on chain, axon advertised, training running
  - Persistence effect: strategy profiles checkpointed to disk
  - Observable signal: axon responds to strategy queries, training metrics logged
- Required tests:
  - `cargo test -p myosu-miner miner::tests::register_on_devnet`
  - `cargo test -p myosu-miner miner::tests::serve_strategy_query`
  - `cargo test -p myosu-miner miner::tests::training_produces_strategy`
- Pass/fail:
  - Miner registers on subnet 1 and receives a UID
  - Axon endpoint accepts `{game_state}` and returns `{action_probs}`
  - After 100 MCCFR iterations, strategy is non-uniform (not random)
  - Strategy checkpoint saved to disk, loadable on restart
  - Miner gracefully shuts down and deregisters
- Blocking note: no miner = no supply. Validators have nothing to score.
- Rollback condition: robopoker training API changes or axon protocol mismatches.

---

## D. Validator

### AC-VL-01: Validator Binary with Exploitability Oracle

- Where: `crates/myosu-validator/` (new)
- How: Binary that:
  1. Registers as a validator on a game-solving subnet (must have sufficient stake)
  2. Periodically queries each miner's axon with game state challenges
  3. Computes exploitability of each miner's strategy using best-response
  4. Submits weight vector to chain via `set_weights` (or commit-reveal)
  The **exploitability oracle** works by:
  - Generating a random sample of N game states (position, board, stacks)
  - Querying the miner for action distributions at each state
  - Computing best-response value against the miner's strategy
  - Scoring: `weight = max(0, 1 - exploitability / baseline_exploitability)`
  The commit-reveal mechanism hides test positions until after miners respond,
  preventing memorization.
  CLI: `myosu-validator --chain ws://localhost:9944 --subnet 1 --seed <key>`
- Whole-system effect: validators are the quality control layer. Without them,
  there's no signal to Yuma Consensus about which miners are good.
- State: miner scores, test position sets, weight history, chain registration.
- Wiring contract:
  - Trigger: `myosu-validator` CLI launch + periodic evaluation timer
  - Callsite: `crates/myosu-validator/src/main.rs`
  - State effect: weight vector submitted to chain per tempo
  - Persistence effect: weight vector stored on-chain, scores logged locally
  - Observable signal: weights visible via chain RPC, evaluation metrics logged
- Required tests:
  - `cargo test -p myosu-validator validator::tests::register_on_devnet`
  - `cargo test -p myosu-validator validator::tests::query_miner_axon`
  - `cargo test -p myosu-validator validator::tests::compute_exploitability`
  - `cargo test -p myosu-validator validator::tests::submit_weights`
  - `cargo test -p myosu-validator validator::tests::commit_reveal_flow`
- Pass/fail:
  - Validator registers on subnet 1 with validator permit
  - Queries miner axon and receives valid action distributions
  - Exploitability computation returns positive value for random strategy
  - Weight vector submitted to chain and visible via RPC
  - Commit-reveal: hash submitted first, reveal accepted after deadline
  - Miner that returns garbage gets weight 0
- Blocking note: without validators, Yuma Consensus has no input. The chain
  distributes zero emissions. The exploitability oracle is the core innovation.
- Rollback condition: exploitability computation is non-deterministic (INV-003)
  or too expensive to run within tempo period.

---

## E. Gameplay

### AC-GP-01: Human vs Bot Gameplay CLI

- Where: `crates/myosu-play/` (new)
- How: CLI application that:
  1. Connects to the chain to discover the highest-ranked miner
  2. Queries that miner's axon for action distributions
  3. Runs a local poker game (NLHE heads-up) where the human makes decisions
     and the bot plays according to the miner's strategy
  4. Displays game state, pot, community cards, and bot's played actions
  Game loop: deal → human acts → bot acts (sampled from strategy) → repeat
  until showdown. Uses robopoker's `Game` engine for state management.
  CLI: `myosu-play --chain ws://localhost:9944 --subnet 1 --stake 100`
- Whole-system effect: this is the product. Without gameplay, the solver market
  has no consumer-facing value. This is what makes people care about solver quality.
- State: local game state, connection to chain and miner axon, session stats.
- Wiring contract:
  - Trigger: `myosu-play` CLI launch
  - Callsite: `crates/myosu-play/src/main.rs`
  - State effect: game runs locally, bot actions fetched from miner
  - Persistence effect: hand history saved locally
  - Observable signal: complete hand plays out in terminal
- Required tests:
  - `cargo test -p myosu-play play::tests::game_loop_completes`
  - `cargo test -p myosu-play play::tests::bot_actions_valid`
  - `cargo test -p myosu-play play::tests::hand_history_saved`
- Pass/fail:
  - Human can fold, check, call, raise, or go all-in via CLI prompts
  - Bot's actions are legal (from miner's strategy distribution)
  - Hand completes and pot is correctly awarded
  - Hand history recorded with all actions and outcomes
  - Gracefully handles miner disconnection (falls back to random strategy)
- Blocking note: gameplay is the consumer-facing product. Without it, myosu is
  an academic exercise in decentralized solver coordination.
- Rollback condition: miner query latency > 2s makes gameplay unplayable, or
  robopoker game state cannot be shared between play and miner.

---

## F. Future Game Expansions (Architecture Only)

### AC-FG-01: Multi-Game Subnet Architecture

- Where: `crates/myosu-games/` (extend), `crates/myosu-chain/pallets/game-solver/` (extend)
- How: Ensure the architecture supports adding new games without chain changes:
  - `GameEngine` trait in `myosu-games` is the extension point
  - `pallet_game_solver` stores `game_type: Vec<u8>` per subnet (opaque)
  - Miner/validator binaries select game engine based on subnet's `game_type`
  - New game = new `GameEngine` impl + new `Exploitability` impl + new miner/validator config
  Document the expansion path for:
  - **Backgammon**: dice + decisions, TD-learning baseline, doubling cube
  - **Mahjong (Riichi)**: 4-player, tile draws, imperfect info, massive state space
  - **Bridge**: 4-player partnership, bidding + play, information inference
  - **Liar's Dice**: small state space, good for proof-of-concept subnet
  - **Short Deck Hold'em**: smaller card set, faster solving, variant poker subnet
  - **PLO (Pot-Limit Omaha)**: 4 hole cards, larger game tree than NLHE
- Whole-system effect: proves the architecture is multi-game, not poker-only.
  Each new game is a new subnet with its own solver market.
- State: no new runtime state — architecture documentation.
- Wiring contract:
  - Trigger: N/A (architecture documentation)
  - Callsite: N/A
  - State effect: N/A
  - Persistence effect: architecture doc in `docs/multi-game-expansion.md`
  - Observable signal: document exists and is reviewed
- Required tests:
  - `cargo test -p myosu-games games::tests::liar_dice_game_lifecycle`
    (implement Liar's Dice as proof that the trait system works for non-poker)
- Pass/fail:
  - Liar's Dice implements `GameEngine` trait completely
  - Liar's Dice game runs from start to finish
  - Exploitability can be computed for Liar's Dice strategies
  - Architecture doc describes expansion path for each candidate game
- Blocking note: without proving the architecture is multi-game, myosu is just
  a poker chain. The multi-game story is the differentiator.
- Rollback condition: `GameEngine` trait cannot express a non-poker game's
  information structure.

---

## Operational Controls

Phase order:
1. Chain scaffold (CH-01) — blocks produce, RPC works
2. Game engine (GE-01, GE-02) — poker wraps robopoker, traits compile
3. Pallet (CH-02) — on-chain incentive mechanics work
4. Miner (MN-01) — produces and serves strategies
5. Validator (VL-01) — scores and submits weights
6. Gameplay (GP-01) — humans play against bots
7. Multi-game proof (FG-01) — architecture validated with Liar's Dice

Gate rules:
- CH-01 must produce blocks before any pallet work
- GE-01 trait definitions must compile before GE-02 wraps robopoker
- CH-02 pallet must accept weights before VL-01 submits them
- MN-01 must serve queries before VL-01 can query miners
- VL-01 must submit weights before GP-01 can discover best miner

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| Chain startup | Missing genesis config | Yes | Yes | Fatal: chain won't start |
| Neuron registration | Insufficient burn balance | Yes | Yes | Clear error message |
| MCCFR training | OOM on large game tree | Yes | Yes | Graceful degradation to smaller abstraction |
| Axon serving | Port already in use | Yes | Yes | Retry with different port |
| Exploitability | Non-deterministic result | Yes | Yes | Fatal: violates INV-003 |
| Weight submission | Transaction rejected | Yes | Yes | Retry with backoff |
| Gameplay query | Miner unreachable | Yes | Yes | Fallback to random strategy |

## Decision Log

- 2026-03-16: Fork subtensor rather than deploy as Bittensor subnet — need
  control over chain parameters, game-specific tokenomics, no $1-2M lock cost.
- 2026-03-16: Keep Yuma Consensus math mostly unchanged — proven incentive
  mechanism, currency-agnostic at the computation layer.
- 2026-03-16: robopoker v1.0.0 via git dependency, not vendored — maintain
  upstream fidelity (INV-006), contribute patches back.
- 2026-03-16: CLI-only gameplay for bootstrap — web UI is scope creep, CLI
  proves the product thesis cheaply.
- 2026-03-16: Liar's Dice as multi-game proof — smallest viable non-poker game,
  solvable exactly, validates trait abstraction.
- 2026-03-16: Name "myosu" (묘수) chosen — Korean for "brilliant move," reflects
  game-solving focus and Korean gaming culture heritage.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `myosu-node --dev` produces blocks, RPC responds | Chain foundation | CH-01 |
| 2 | Create NLHE-HU subnet on devnet | Pallet registration | CH-02 |
| 3 | Register miner, run 100 MCCFR iterations, serve strategy query | Miner lifecycle | MN-01, GE-02 |
| 4 | Validator queries miner, computes exploitability, submits weights | Validation loop | VL-01, GE-02 |
| 5 | After tempo, Yuma distributes emissions to highest-scored miner | End-to-end incentive | CH-02, MN-01, VL-01 |
| 6 | Human plays one complete hand of NLHE against best miner's bot | Product thesis | GP-01 |
| 7 | Liar's Dice implements GameEngine trait and runs a complete game | Multi-game arch | FG-01, GE-01 |
| 8 | Full loop: chain → miner → validator → Yuma → gameplay on devnet | System integration | All |
