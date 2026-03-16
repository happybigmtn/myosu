# Specification: Game Engine Traits — Multi-Game Abstraction Layer

Source: Master spec AC-GE-01, robopoker v1.0.0 trait analysis
Status: Draft
Date: 2026-03-16
Depends-on: none (pure library, no chain dependency)

## Purpose

Define the trait abstraction that makes myosu a multi-game platform. Every game
(poker, backgammon, mahjong, Liar's Dice) implements these traits. The solver
(miner), validator, and gameplay layers depend only on these traits — they never
import game-specific types directly.

**Critical discovery**: robopoker v1.0.0 already provides game-agnostic CFR
traits (`CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Profile`, `Encoder`) in
its `rbp-mccfr` crate. Rather than creating a parallel abstraction, myosu's
traits should **thin-wrap** robopoker's trait system, adding only what's needed
for network transport (serialization) and the validator oracle (exploitability
as a standalone function).

The primary consumers are `myosu-miner`, `myosu-validator`, and `myosu-play`.

**Key design constraint**: traits must be object-safe where possible for dynamic
dispatch. Games are registered at runtime (miners join subnets), not at compile
time.

## Whole-System Goal

Current state:
- robopoker's `rbp-mccfr` crate defines `CfrGame`, `CfrEdge`, `CfrTurn`,
  `CfrInfo`, `Profile`, `Encoder` — all game-agnostic
- robopoker's `Profile` trait has `exploitability()` and
  `averaged_distribution()` methods built in
- robopoker has a Rock-Paper-Scissors reference implementation in `rps/`
- No myosu game abstraction exists

This spec adds:
- `myosu-games` crate with thin wrapper traits over robopoker's trait system
- Serialization layer for network transport of game states and strategies
- Game registry for runtime game selection
- Exploitability as a standalone queryable function (not just on Profile)

If all ACs land:
- Any game implementing robopoker's CFR traits can plug into myosu
- Miners serve strategy queries over the network using serialized types
- Validators compute exploitability for any registered game
- The gameplay layer can run any registered game

Still not solved here:
- Specific game implementations (poker, Liar's Dice — separate specs)
- Network protocol format (miner/validator specs)
- Training pipeline configuration per game

12-month direction:
- 8+ game implementations sharing these traits
- Hot-loadable game engines via dynamic libraries
- Cross-game transfer learning using shared trait interface

## Why This Spec Exists As One Unit

- The trait definitions, serialization layer, game registry, and exploitability
  function form a single coherent API surface
- Splitting would create specs that can't demonstrate a working round trip
  (serialize strategy → deserialize → compute exploitability)
- The RPS reference implementation validates all traits together

## Scope

In scope:
- Wrapper traits adding serialization to robopoker's CFR traits
- `GameRegistry` for runtime game selection by game_type string
- Standalone exploitability computation function
- Serialization/deserialization for strategy profiles (action distributions)
- RPS reference implementation proving the trait system works

Out of scope:
- Poker-specific implementation (spec: 031626-poker-engine.md)
- Liar's Dice implementation (spec: 031626-multi-game-architecture.md)
- Network transport protocol — serialization format only, not wire protocol
- Training configuration — that's the miner spec

## Current State

- `crates/myosu-games/src/lib.rs` — stub, compiles but empty
- `crates/myosu-games/Cargo.toml` — exists with basic deps, no robopoker
- robopoker v1.0.0 at `/home/r/coding/robopoker` with:
  - `rbp-mccfr/src/state/game.rs` — `CfrGame` trait (Copy + Send + Sync)
  - `rbp-mccfr/src/state/edge.rs` — `CfrEdge` trait (Copy + Hash + Ord)
  - `rbp-mccfr/src/state/turn.rs` — `CfrTurn` trait (chance/terminal/player)
  - `rbp-mccfr/src/state/info.rs` — `CfrInfo` trait (public + private)
  - `rbp-mccfr/src/strategy/profile.rs` — `Profile` trait with
    `exploitability()`, `averaged_distribution()`, `iterated_distribution()`
  - `rbp-mccfr/src/strategy/encoder.rs` — `Encoder` trait
  - `rbp-mccfr/src/rps/` — Rock-Paper-Scissors reference implementation

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Game state trait | `rbp-mccfr::CfrGame` | reuse | Exactly what we need (root, turn, apply, payoff) |
| Action trait | `rbp-mccfr::CfrEdge` | reuse | Copy + Hash + Ord is correct for strategy tables |
| Turn/node type | `rbp-mccfr::CfrTurn` | reuse | chance/terminal/player distinction |
| Information set | `rbp-mccfr::CfrInfo` | reuse | public + private decomposition |
| Strategy profile | `rbp-mccfr::Profile` | extend | Has exploitability, needs serialization |
| Encoder | `rbp-mccfr::Encoder` | reuse | Maps game states to info sets |
| RPS example | `rbp-mccfr::rps` | reference | Shows how to implement all traits |

## Non-goals

- Creating a wholly new trait system parallel to robopoker's — unnecessary duplication
- Object safety for all traits — `CfrGame: Copy` prevents `dyn CfrGame`, which is fine;
  use enum dispatch or generic monomorphization instead
- Persistence format — serialization is for network transport, not storage
- Backward compatibility with codexpoker's trait system — clean break

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Re-export traits | New | crates/myosu-games/src/traits.rs |
| Serialization layer | New | crates/myosu-games/src/wire.rs |
| Game registry | New | crates/myosu-games/src/registry.rs |
| Exploitability fn | New | crates/myosu-games/src/exploit.rs |
| Config types | New | crates/myosu-games/src/config.rs |
| Crate root | Extend | crates/myosu-games/src/lib.rs |

---

## A. Core Traits

### AC-GT-01: Re-export and Extend Robopoker CFR Traits

- Where: `crates/myosu-games/src/traits.rs (new)`, `crates/myosu-games/Cargo.toml (extend)`
- How: Add `rbp-mccfr` as a dependency (via git tag v1.0.0). Re-export the
  core CFR traits with myosu-specific documentation:

  ```rust
  // Re-exports from robopoker — these ARE the game-agnostic interface
  pub use rbp_mccfr::{CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder};
  pub use rbp_mccfr::{Tree, Node, Branch, InfoSet};
  pub use rbp_core::{Utility, Probability};
  ```

  Add a `GameConfig` struct for subnet-specific configuration:
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct GameConfig {
      pub game_type: String,       // "nlhe_hu", "nlhe_6max", "liars_dice", etc.
      pub num_players: u8,
      pub params: serde_json::Value,  // game-specific config (stack sizes, blind structure, etc.)
  }
  ```

  Add a `StrategyQuery` and `StrategyResponse` pair for miner communication:
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct StrategyQuery<I: Serialize> {
      pub info: I,                           // information set
  }

  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct StrategyResponse<E: Serialize> {
      pub actions: Vec<(E, Probability)>,    // action distribution
  }
  ```

  These are generic over the game's Info and Edge types but serializable
  for network transport.

- Whole-system effect: establishes the API contract between all myosu crates.
  Every game, miner, validator, and player depends on these types.
- State: no runtime state — type definitions and re-exports.
- Wiring contract:
  - Trigger: compile-time dependency
  - Callsite: myosu-miner, myosu-validator, myosu-play import these
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: `cargo test -p myosu-games` compiles and passes
- Required tests:
  - `cargo test -p myosu-games traits::tests::reexports_compile`
  - `cargo test -p myosu-games traits::tests::game_config_serializes`
  - `cargo test -p myosu-games traits::tests::strategy_query_response_roundtrip`
- Pass/fail:
  - `CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Profile` all importable from `myosu_games`
  - `GameConfig { game_type: "nlhe_hu", num_players: 2, params: json!({}) }` serializes to JSON
  - `StrategyQuery` → serialize → deserialize round-trips correctly
  - `StrategyResponse` with action probabilities summing to 1.0 validates
- Blocking note: every other AC in this spec and downstream specs depends on
  these types being importable. This is the foundation.
- Rollback condition: `rbp-mccfr` crate is not publishable or has incompatible
  dependency requirements.

### AC-GT-02: Wire Serialization for Strategy Transport

- Where: `crates/myosu-games/src/wire.rs (new)`
- How: Define serialization wrappers for types that need to cross the network
  boundary (miner axon → validator query):

  ```rust
  /// Serialized strategy for a specific game state.
  /// Game-type agnostic — the deserializer knows the game from the subnet.
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct WireStrategy {
      pub game_type: String,
      pub info_bytes: Vec<u8>,                    // serialized info set
      pub actions: Vec<(Vec<u8>, Probability)>,   // (serialized edge, probability)
  }

  /// Trait for games that support wire serialization.
  pub trait WireSerializable {
      type E: CfrEdge + Serialize + DeserializeOwned;
      type I: CfrInfo + Serialize + DeserializeOwned;

      fn serialize_info(info: &Self::I) -> Vec<u8>;
      fn deserialize_info(bytes: &[u8]) -> Result<Self::I>;
      fn serialize_edge(edge: &Self::E) -> Vec<u8>;
      fn deserialize_edge(bytes: &[u8]) -> Result<Self::E>;
  }
  ```

  The `WireStrategy` type is what flows between miners and validators over
  the network. It's opaque to the transport layer — only the game-specific
  deserializer knows how to interpret the bytes.

- Whole-system effect: enables strategy profiles to be transmitted over the
  network between miners and validators. Without this, the off-chain
  evaluation loop can't work.
- State: no runtime state — serialization utilities.
- Wiring contract:
  - Trigger: miner serializes strategy response, validator deserializes
  - Callsite: myosu-miner axon handler, myosu-validator query handler
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: round-trip serialization tests pass
- Required tests:
  - `cargo test -p myosu-games wire::tests::wire_strategy_roundtrip`
  - `cargo test -p myosu-games wire::tests::wire_strategy_json_format`
  - `cargo test -p myosu-games wire::tests::invalid_bytes_error`
- Pass/fail:
  - `WireStrategy` serializes to JSON and deserializes back identically
  - Action probabilities preserved to f64 precision
  - Invalid bytes produce clear error, not panic
  - Empty action list is valid (terminal state has no actions)
- Blocking note: miners and validators must agree on a serialization format.
  Without this, they can't communicate strategy data.
- Rollback condition: robopoker's Edge/Info types don't implement Serialize
  (they may need the "client" feature flag).

---

## B. Game Registry

### AC-GT-03: Runtime Game Selection

- Where: `crates/myosu-games/src/registry.rs (new)`
- How: Implement a `GameRegistry` that maps `game_type` strings to game engine
  implementations. Since `CfrGame: Copy` prevents trait objects, use an enum
  dispatch pattern:

  ```rust
  /// Known game types that myosu supports.
  #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
  pub enum GameType {
      NlheHeadsUp,
      NlheSixMax,
      LiarsDice,
      Custom(String),
  }

  impl GameType {
      pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
          match bytes {
              b"nlhe_hu" => Some(Self::NlheHeadsUp),
              b"nlhe_6max" => Some(Self::NlheSixMax),
              b"liars_dice" => Some(Self::LiarsDice),
              _ => Some(Self::Custom(String::from_utf8_lossy(bytes).into())),
          }
      }
      pub fn to_bytes(&self) -> Vec<u8> { ... }
      pub fn num_players(&self) -> u8 { ... }
  }
  ```

  The registry doesn't need to hold instances — it maps game types to
  factories/constructors. Actual game instances are created by miners
  and validators when they join a subnet.

- Whole-system effect: miners and validators select their game engine based
  on which subnet they join. Without a registry, there's no way to map
  the on-chain `game_type` bytes to an implementation.
- State: no runtime state — enum and match dispatch.
- Wiring contract:
  - Trigger: miner/validator reads subnet game_type from chain
  - Callsite: myosu-miner/src/main.rs, myosu-validator/src/main.rs
  - State effect: correct game engine selected
  - Persistence effect: N/A
  - Observable signal: `GameType::from_bytes(b"nlhe_hu")` returns `NlheHeadsUp`
- Required tests:
  - `cargo test -p myosu-games registry::tests::known_game_types`
  - `cargo test -p myosu-games registry::tests::unknown_game_type_custom`
  - `cargo test -p myosu-games registry::tests::roundtrip_bytes`
  - `cargo test -p myosu-games registry::tests::num_players_correct`
- Pass/fail:
  - `from_bytes(b"nlhe_hu")` → `NlheHeadsUp`
  - `from_bytes(b"nlhe_6max")` → `NlheSixMax`
  - `from_bytes(b"unknown_game")` → `Custom("unknown_game")`
  - `NlheHeadsUp.num_players()` → 2
  - `to_bytes()` → `from_bytes()` roundtrip is identity
- Blocking note: miners need to know which solver to run. Without game
  type mapping, they can't select the right engine for their subnet.
- Rollback condition: game_type encoding on-chain doesn't match registry expectations.

### AC-GT-04: Standalone Exploitability Function

- Where: `crates/myosu-games/src/exploit.rs (new)`
- How: Expose exploitability as a standalone function that validators can call
  without constructing a full Profile. Wrap robopoker's `Profile::exploitability()`:

  ```rust
  /// Compute exploitability of a strategy defined by an action distribution function.
  ///
  /// Returns exploitability in game-native utility units.
  /// For poker: milli-big-blinds per hand (mbb/h).
  ///
  /// The `strategy_fn` closure maps info sets to action distributions.
  /// This allows validators to test miner strategies without having
  /// the full Profile object — just the query interface.
  pub fn compute_exploitability<G, E, T, I, P>(
      profile: &P,
      tree: Tree<T, E, G, I>,
  ) -> Utility
  where
      P: Profile<T = T, E = E, G = G, I = I>,
      G: CfrGame<E = E, T = T>,
      E: CfrEdge,
      T: CfrTurn,
      I: CfrInfo<E = E, T = T>,
  {
      profile.exploitability(tree)
  }
  ```

  This is essentially a thin wrapper that makes the call site cleaner for
  validators. The real work is in robopoker's `Profile::exploitability()`.

  Also expose a **sampled exploitability** function for large games where
  full tree traversal is too expensive:

  ```rust
  /// Sample-based exploitability estimation.
  /// Generates `n_samples` random game states, computes best-response
  /// value at each, and returns the average.
  pub fn sampled_exploitability<G, E, T, I, P>(
      profile: &P,
      n_samples: usize,
      rng: &mut impl Rng,
  ) -> Utility
  where
      P: Profile<T = T, E = E, G = G, I = I>,
      ...
  {
      // Sample random game states by playing random actions from root
      // At each terminal state, compute best-response deviation
      // Average over samples
  }
  ```

- Whole-system effect: the validator oracle's core scoring function. Without
  this, validators can't compute how close a miner's strategy is to Nash.
- State: no runtime state — pure computation.
- Wiring contract:
  - Trigger: validator evaluation loop calls this per miner
  - Callsite: myosu-validator/src/oracle/
  - State effect: N/A (pure function)
  - Persistence effect: N/A
  - Observable signal: returns f64 exploitability value
- Required tests:
  - `cargo test -p myosu-games exploit::tests::rps_nash_exploitability_zero`
  - `cargo test -p myosu-games exploit::tests::rps_biased_exploitability_positive`
  - `cargo test -p myosu-games exploit::tests::sampled_converges_to_exact`
- Pass/fail:
  - RPS Nash equilibrium (1/3, 1/3, 1/3) → exploitability ≈ 0.0
  - RPS biased strategy (always rock) → exploitability > 0
  - Sampled exploitability with 10,000 samples within 5% of exact value
  - Invalid profile (NaN probabilities) → error, not panic
- Blocking note: this is the validator's scoring function. The entire
  incentive mechanism depends on accurate exploitability measurement.
- Rollback condition: robopoker's exploitability computation is too slow for
  the game sizes we need, or doesn't support 2+ player games correctly.

---

## C. Validation

### AC-GT-05: RPS Reference Implementation Test Suite

- Where: `crates/myosu-games/src/lib.rs (extend)`, `crates/myosu-games/tests/ (new)`
- How: Use robopoker's built-in Rock-Paper-Scissors implementation (`rbp-mccfr::rps`)
  to validate the entire trait system end-to-end:

  1. Create an RPS solver using robopoker's `rps::RpsSolver`
  2. Train for 1000 iterations
  3. Verify strategy converges to (1/3, 1/3, 1/3)
  4. Serialize the strategy via `WireStrategy`
  5. Deserialize and verify round-trip
  6. Compute exploitability → should be ≈ 0.0
  7. Verify `GameType` registry recognizes RPS (or Custom)

  This test exercises every AC in this spec against a known-solvable game
  with a known Nash equilibrium.

- Whole-system effect: proves the entire trait system works before we add
  poker complexity. If RPS doesn't work, nothing will.
- State: test harness only.
- Wiring contract:
  - Trigger: `cargo test -p myosu-games`
  - Callsite: `crates/myosu-games/tests/rps_integration.rs`
  - State effect: N/A (test)
  - Persistence effect: N/A
  - Observable signal: all RPS tests pass
- Required tests:
  - `cargo test -p myosu-games rps_integration::train_rps_to_nash`
  - `cargo test -p myosu-games rps_integration::rps_exploitability_near_zero`
  - `cargo test -p myosu-games rps_integration::rps_strategy_wire_roundtrip`
  - `cargo test -p myosu-games rps_integration::rps_biased_strategy_exploitable`
- Pass/fail:
  - After 1000 training iterations, each RPS action has probability within
    [0.30, 0.37] (converging to 1/3)
  - Exploitability < 0.01 after training
  - WireStrategy serialization preserves all action probabilities
  - Always-rock strategy has exploitability > 0.3 (beaten by always-paper)
  - Full pipeline: train → serialize → deserialize → exploit → score works
- Blocking note: if we can't validate the trait system on a trivial game,
  we have no business applying it to poker.
- Rollback condition: robopoker's RPS module is not compatible with the
  wire serialization layer.

---

## Operational Controls

Phase order:
1. GT-01 (re-exports) — traits importable, GameConfig/Query/Response defined
2. GT-02 (wire) — serialization layer works
3. GT-03 (registry) — game type mapping works
4. GT-04 (exploitability) — standalone exploit function works
5. GT-05 (RPS validation) — end-to-end test passes

Gate rules:
- GT-01 must compile before any other GT-* AC
- GT-02 depends on GT-01 types
- GT-04 depends on GT-01 trait re-exports
- GT-05 depends on all of GT-01..04

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| rbp-mccfr import | Version mismatch or feature flag | Yes | Yes | Build error |
| Serialization | Non-Serialize types in robopoker | Yes | Yes | Need "client" feature |
| Exploitability | NaN/Inf in probability distributions | Yes | Yes | Return error |
| Game registry | Unknown game type from chain | Yes | No | Custom variant handles it |

## Decision Log

- 2026-03-16: Thin-wrap robopoker traits instead of parallel abstraction —
  robopoker's `rbp-mccfr` is already game-agnostic with correct trait bounds.
  Duplicating it would mean maintaining two trait hierarchies.
- 2026-03-16: Enum dispatch instead of trait objects — `CfrGame: Copy` prevents
  `dyn CfrGame`. Enum dispatch (`GameType` enum) is simpler and equally
  extensible for our use case.
- 2026-03-16: RPS as validation target — known Nash equilibrium (1/3, 1/3, 1/3)
  means we can verify exploitability = 0 with certainty.
- 2026-03-16: Sampled exploitability alongside exact — full-game-tree
  exploitability is O(|game tree|) which is intractable for NLHE. Sampled
  version enables validators to score large games.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `use myosu_games::{CfrGame, Profile}` compiles | Re-exports | GT-01 |
| 2 | GameConfig serializes to/from JSON | Config types | GT-01 |
| 3 | WireStrategy round-trips through serde_json | Serialization | GT-02 |
| 4 | `GameType::from_bytes(b"nlhe_hu")` → NlheHeadsUp | Registry | GT-03 |
| 5 | RPS Nash exploitability < 0.01 | Exploitability | GT-04 |
| 6 | Train RPS → serialize → deserialize → exploit → score | End-to-end | GT-05 |
