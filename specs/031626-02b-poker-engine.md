# Specification: Poker Engine — NLHE Implementation via Robopoker v1.0.0

Source: Master spec AC-GE-02, robopoker fork crate analysis
Status: Draft
Date: 2026-03-30
Depends-on: GT-01..05 (game engine traits must exist)

## Purpose

Implement the No-Limit Hold'em poker engine by wrapping robopoker fork's
`rbp-nlhe` and `rbp-gameplay` crates. This produces the first concrete game
that miners can solve, validators can score, and players can play.

Robopoker already implements all CFR traits (`NlheGame: CfrGame`,
`NlheEncoder: Encoder`, `NlheProfile: Profile`). The poker engine spec
adds: wire serialization for poker-specific types, exploitability integration,
strategy query handling, and the training pipeline interface.

The primary consumers are `myosu-miner` (training + serving), `myosu-validator`
(exploitability scoring), and `myosu-play` (human vs bot).

**Key design constraint**: keep the poker engine as a thin owned wrapper around
the robopoker fork instead of re-implementing MCCFR locally. Stage 0 now
depends on the live forked crate surface, not a hypothetical upstream-only
patch queue.

## Whole-System Goal

Current state:
- `myosu-games` provides trait re-exports and wire serialization (GT-01..05)
- robopoker fork provides `rbp-nlhe` with `NlheSolver`, `NlheEncoder`,
  `NlheProfile`, `NlheGame`, `NlheEdge`, `NlheTurn`, `NlheInfo`
- robopoker's `NlheProfile` already implements `Profile::exploitability()`
- `crates/myosu-games-poker/` already exists as the live stage-0 poker wrapper
  consumed by gameplay, miner, and validator paths

This spec adds:
- the truthful contract for the poker wrapper crate that now exists
- any remaining seam-hardening around wire format, artifact loading, and
  strategy request/response handling
- an ownership map that matches the real in-repo module layout

If all ACs land:
- A miner can create an `NlheSolver`, train it, and serve strategy queries
- A validator can receive action distributions and compute exploitability
- The gameplay layer can query strategies for bot play

Still not solved here:
- PLO, short deck, or other poker variants — separate future specs
- Database persistence (PostgreSQL) — miners use in-memory or file checkpoints
- Clustering pipeline — uses pre-computed abstractions from robopoker
- Multi-way (6-max, full ring) — heads-up only for bootstrap

12-month direction:
- Multiple poker variant subnets (HU, 6-max, PLO, short deck)
- Pre-computed abstraction sharing across miners
- Subgame solving for real-time play refinement

## Why This Spec Exists As One Unit

- The solver wrapper, query handler, wire serialization, and exploitability
  integration are all needed for one outcome: "a miner can serve poker
  strategies that a validator can score"
- Testing requires all pieces: create solver → train → query → score
- Each piece is small (~100-200 lines) — splitting would create trivial specs

## Scope

In scope:
- `myosu-games-poker` crate in the workspace
- NlheSolver creation and configuration
- Training iteration interface (step, batch, checkpoint)
- Strategy query (info → action distribution via `averaged_distribution`)
- Wire serialization for NlheInfo and NlheEdge
- Exploitability computation for miner strategies
- GameType::NlheHeadsUp registration

Out of scope:
- Clustering pipeline (requires PostgreSQL, deferred)
- Database persistence — file-based checkpoints only
- Multi-way poker (6-max, full ring)
- PLO, short deck, or tournament variants
- Subgame solving — uses blueprint strategy only for now

## Current State

- `crates/myosu-games-poker/src/lib.rs` — live poker wrapper re-export surface
- `crates/myosu-games-poker/src/solver.rs` — live `PokerSolver` wrapper and
  checkpoint-backed training surface
- `crates/myosu-games-poker/src/wire.rs` — live request/response codec helpers
- `crates/myosu-games-poker/src/request.rs` — live strategy request types
- `crates/myosu-games-poker/src/state.rs` and `src/action.rs` — live gameplay
  state and action surfaces shared with `myosu-play`
- `crates/myosu-games-poker/src/renderer.rs` — live NLHE renderer for the TUI
  and pipe surfaces
- `crates/myosu-games-poker/src/artifacts.rs` — live encoder artifact loader
  and manifest contract
- robopoker remains the underlying solver/engine source at
  `/home/r/coding/robopoker/`

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| NLHE game engine | `rbp-gameplay::Game` | reuse | Copy struct, efficient, correct |
| NLHE solver | `rbp-nlhe::NlheSolver` | reuse | Full MCCFR with sampling schemes |
| Strategy storage | `rbp-nlhe::NlheProfile` | reuse | BTreeMap-based, has exploitability |
| Info encoding | `rbp-nlhe::NlheEncoder` | reuse | Isomorphism → abstraction mapping |
| Training loop | `rbp-mccfr::Solver::step()` | reuse | One CFR iteration |
| Exploitability | `rbp-mccfr::Profile::exploitability()` | reuse | Best-response computation |
| Subgame solving | `NlheSolver::subgame()` | reference | For future real-time play |

## Non-goals

- Re-implementing MCCFR — use robopoker's implementation directly
- Custom abstraction pipeline — use robopoker's default encoding
- Hand evaluation — use robopoker's `Evaluator`
- Multi-way support — heads-up only for bootstrap

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Poker crate | Implemented | crates/myosu-games-poker/src/lib.rs |
| Solver wrapper | Implemented | crates/myosu-games-poker/src/solver.rs |
| Wire impl | Implemented | crates/myosu-games-poker/src/wire.rs |
| Strategy request/response types | Implemented | crates/myosu-games-poker/src/request.rs |
| Gameplay state/actions | Implemented | crates/myosu-games-poker/src/state.rs, crates/myosu-games-poker/src/action.rs |
| TUI/pipe renderer | Implemented | crates/myosu-games-poker/src/renderer.rs |
| Artifact loading | Implemented | crates/myosu-games-poker/src/artifacts.rs |
| Dedicated `query.rs` / `training.rs` split | Not present | current functionality lives in `solver.rs`, `request.rs`, and `wire.rs` |

---

## A. Solver Integration

### AC-PE-01: Poker Solver Wrapper

- Where: `crates/myosu-games-poker/src/solver.rs (new)`, `Cargo.toml (new)`
- How: Create a `PokerSolver` struct wrapping `NlheSolver` with Pluribus-style
  configuration (the default in robopoker):

  ```rust
  use rbp_nlhe::*;
  use rbp_mccfr::*;

  /// Configured NLHE solver using Pluribus sampling and linear discounting.
  pub type Flagship = NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>;

  pub struct PokerSolver {
      solver: Flagship,
  }

  impl PokerSolver {
      /// Create a new solver with empty profile and encoder.
      pub fn new(encoder: NlheEncoder) -> Self { ... }

      /// Load solver from checkpoint file.
      pub fn load(path: &Path) -> Result<Self> { ... }

      /// Save solver state to checkpoint file.
      pub fn save(&self, path: &Path) -> Result<()> { ... }

      /// Run N training iterations.
      pub fn train(&mut self, iterations: usize) { ... }

      /// Query the trained strategy for an info set.
      pub fn strategy(&self, info: &NlheInfo) -> Vec<(NlheEdge, Probability)> {
          self.solver.profile().averaged_distribution(info).into_iter().collect()
      }

      /// Compute exploitability of current strategy.
      pub fn exploitability(&self) -> Utility {
          let tree = Tree::build(&self.solver.encoder(), NlheGame::root());
          self.solver.profile().exploitability(tree)
      }

      /// Get current training epoch count.
      pub fn epochs(&self) -> usize { self.solver.profile().epochs() }
  }
  ```

  Checkpoint format: 4-byte magic (`MYOS`) + 4-byte version (u32) + bincode
  serialization of `NlheProfile`. On load, verify magic and version; reject
  with clear error if mismatched ("checkpoint version 1, expected 2; re-train
  required"). The encoder is reconstructed from abstraction tables (separate
  from checkpoint, pinned to a hash-checked versioned artifact).

- Whole-system effect: the miner binary wraps this struct. Without it, there's
  no way to train or query poker strategies.
- State: NlheProfile (regrets, weights, strategies per info set).
- Wiring contract:
  - Trigger: miner creates PokerSolver on startup
  - Callsite: myosu-miner/src/main.rs
  - State effect: solver holds training state in memory
  - Persistence effect: checkpoint files on disk
  - Observable signal: `solver.epochs()` increases after training
- Required tests:
  - `cargo test -p myosu-games-poker solver::tests::create_empty_solver`
  - `cargo test -p myosu-games-poker solver::tests::train_100_iterations`
  - `cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution`
  - `cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip`
  - `cargo test -p myosu-games-poker solver::tests::exploitability_decreases`
- Pass/fail:
  - Empty solver has 0 epochs
  - After 100 iterations, epochs == 100
  - Strategy query returns probabilities summing to ~1.0 (within float epsilon)
  - Save → load checkpoint → same epoch count and strategy
  - Exploitability after 1000 iterations < exploitability after 10 iterations
  - Strategy for a given info set is non-uniform after training
- Blocking note: this is the core of what miners do. Without a working solver,
  no strategies exist to score or play against.
- Rollback condition: robopoker's Flagship type aliases are not public, or
  NlheProfile is not serializable.

### AC-PE-02: Strategy Query Handler

- Where: `crates/myosu-games-poker/src/solver.rs`,
  `crates/myosu-games-poker/src/request.rs`,
  `crates/myosu-games-poker/src/wire.rs`
- How: Implement the bridge between network queries and solver lookups:

  ```rust
  use myosu_games::{StrategyQuery, StrategyResponse, WireStrategy};

  impl PokerSolver {
      /// Handle a wire query: deserialize info, look up strategy, serialize response.
      pub fn handle_query(&self, wire: &WireStrategy) -> Result<WireStrategy> {
          let info: NlheInfo = deserialize_info(&wire.info_bytes)?;
          let actions = self.strategy(&info);
          Ok(WireStrategy {
              game_type: "nlhe_hu".into(),
              info_bytes: wire.info_bytes.clone(),
              actions: actions.iter()
                  .map(|(e, p)| (serialize_edge(e), *p))
                  .collect(),
          })
      }
  }
  ```

  The query handler is what the miner's axon endpoint calls when a validator
  sends a strategy query.

- Whole-system effect: enables validators to query miners over the network.
  Without this, the off-chain evaluation loop can't function.
- State: no new state — delegates to PokerSolver.
- Wiring contract:
  - Trigger: HTTP/gRPC request to miner axon
  - Callsite: myosu-miner axon handler
  - State effect: N/A (read-only query)
  - Persistence effect: N/A
  - Observable signal: valid WireStrategy response returned
- Required tests:
  - `cargo test -p myosu-games-poker query::tests::handle_valid_query`
  - `cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes`
  - `cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one`
- Pass/fail:
  - Valid info bytes → response with action distribution
  - Invalid info bytes → error, not panic
  - All response probabilities ≥ 0 and sum to ~1.0
  - Response game_type matches query game_type
- Blocking note: the miner axon needs a request handler.
- Rollback condition: NlheInfo serialization format incompatible with validator's query format.

### AC-PE-03: Poker Wire Serialization

- Where: `crates/myosu-games-poker/src/wire.rs (new)`
- How: Implement `WireSerializable` for poker types:

  ```rust
  impl WireSerializable for Poker {
      type E = NlheEdge;
      type I = NlheInfo;

      fn serialize_info(info: &NlheInfo) -> Vec<u8> {
          bincode::serialize(info).expect("NlheInfo serializes")
      }
      fn deserialize_info(bytes: &[u8]) -> Result<NlheInfo> {
          bincode::deserialize(bytes).map_err(|e| ...)
      }
      fn serialize_edge(edge: &NlheEdge) -> Vec<u8> {
          bincode::serialize(edge).expect("NlheEdge serializes")
      }
      fn deserialize_edge(bytes: &[u8]) -> Result<NlheEdge> {
          bincode::deserialize(bytes).map_err(|e| ...)
      }
  }
  ```

  Requires robopoker types to implement `Serialize`/`Deserialize`. If they
  don't by default, use the `client` feature flag on `rbp-gameplay` and
  `rbp-cards` which enables serde derives.

- Whole-system effect: enables poker strategies to cross the network boundary.
- State: no runtime state — serialization functions.
- Wiring contract:
  - Trigger: miner serializes response, validator deserializes
  - Callsite: query handler (PE-02) and validator oracle
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: round-trip serialization preserves all data
- Required tests:
  - `cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip`
  - `cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip`
  - `cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize`
- Pass/fail:
  - NlheInfo serializes and deserializes to identical value
  - All NlheEdge variants (Fold, Check, Call, Raise(odds), Shove, Draw) roundtrip
  - Serialized NlheInfo size < 1KB (compact representation)
  - Corrupted bytes → clear error
- Blocking note: without serialization, miners and validators can't communicate.
- Rollback condition: robopoker types don't support serde even with feature flags.

### AC-PE-04: Poker Exploitability Integration

- Where: `crates/myosu-games-poker/src/exploit.rs (new)`
- How: Expose poker-specific exploitability that validators call:

  ```rust
  /// Compute exploitability of a poker strategy profile.
  /// Returns milli-big-blinds per hand (mbb/h).
  pub fn poker_exploitability(profile: &NlheProfile, encoder: &NlheEncoder) -> Utility {
      let tree = Tree::build(encoder, NlheGame::root());
      profile.exploitability(tree)
  }

  /// Compute exploitability from a query function (for remote strategies).
  /// The query_fn simulates what a validator gets from a miner's axon.
  pub fn remote_poker_exploitability(
      query_fn: impl Fn(&NlheInfo) -> Vec<(NlheEdge, Probability)>,
      encoder: &NlheEncoder,
  ) -> Utility {
      // Build a synthetic Profile from the query function
      // Compute exploitability using Profile::exploitability()
      ...
  }
  ```

  The `remote_poker_exploitability` function is the key innovation: it lets
  validators compute exploitability of a *remote* strategy (accessed via
  network queries) without having the full Profile object.

- Whole-system effect: the validator's scoring function for poker miners.
- State: no runtime state — computation.
- Wiring contract:
  - Trigger: validator evaluation loop
  - Callsite: `crates/myosu-validator/src/validation.rs`
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: returns f64 exploitability value
- Required tests:
  - `cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit`
  - `cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit`
  - `cargo test -p myosu-games-poker exploit::tests::remote_matches_local`
- Pass/fail:
  - Strategy trained 10,000 iterations has exploitability < 500 mbb/h
  - Random strategy has exploitability > 200 mbb/h (significantly exploitable)
  - Remote exploitability (via query function) within 5% of local exploitability
  - Exploitability is always non-negative
- Blocking note: without exploitability, validators have no way to score miners.
- Rollback condition: exploitability computation takes > 60 seconds for HU NLHE.

---

## Operational Controls

Phase order:
1. PE-01 (solver wrapper) — can create and train
2. PE-03 (wire serialization) — can serialize/deserialize types
3. PE-02 (query handler) — can handle network queries
4. PE-04 (exploitability) — can score strategies

Gate rules:
- PE-01 must work before PE-02 or PE-04 (they use the solver)
- PE-03 must work before PE-02 (query handler needs serialization)

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| Solver creation | Missing encoder abstractions | Yes | Yes | Clear error on startup |
| Training OOM | Game tree too large for memory | Yes | Yes | Configurable batch size |
| Checkpoint | Disk write fails | Yes | Yes | Retry or error |
| Serialization | Type mismatch between versions | Yes | Yes | Version in wire format |
| Exploitability | NaN from empty profile | Yes | Yes | Return Infinity or error |

## Decision Log

- 2026-03-16: Use Pluribus-style solver (PluribusRegret, LinearWeight,
  PluribusSampling) as default — best known configuration for NLHE.
- 2026-03-16: Bincode for wire serialization — compact, fast, well-supported.
  JSON is too verbose for action distributions.
- 2026-03-16: File-based checkpoints, not PostgreSQL — simpler for bootstrap.
  Database persistence is a future spec.
- 2026-03-16: Heads-up only — smallest NLHE variant, fastest to solve. 6-max
  and full ring are separate subnets with separate specs.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Create PokerSolver, train 100 iterations, query strategy | Solver lifecycle | PE-01 |
| 2 | Save checkpoint, load, verify same strategy | Persistence | PE-01 |
| 3 | Serialize NlheInfo → bytes → deserialize → identical | Wire format | PE-03 |
| 4 | Handle WireStrategy query → valid response | Query handler | PE-02 |
| 5 | Trained strategy has lower exploitability than random | Scoring | PE-04 |
| 6 | Full pipeline: train → serve → query → score | End-to-end | All |
