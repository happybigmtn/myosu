# `games:poker-engine` Lane Spec

## Lane Boundary

`games:poker-engine` is the **NLHE poker engine integration surface** for the myosu game-solving chain. It owns:

- The `myosu-games-poker` crate (greenfield)
- `PokerSolver` wrapper around `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`
- `StrategyQuery` / `StrategyResponse` bridge for miner-validator communication
- Wire serialization for `NlheInfo` and `NlheEdge` types (bincode)
- Exploitability computation via `Profile::exploitability()` using `Tree::build`
- `GameType::NlheHeadsUp` registration
- File-based checkpoint format (4-byte magic `MYOS` + u32 version + bincode)

`games:poker-engine` does **not** own:

- `myosu-games` trait definitions (owned by `games:traits`)
- Robopoker internals (lives in `/home/r/coding/robopoker/`)
- Any miner or validator binary (owned by `services:miner` and `services:validator-oracle`)
- Any poker variant beyond NLHE heads-up (NLHE 6-max, PLO, Short Deck, Tournament — owned by `games:variant-family`)

---

## Platform-Facing Purpose

The poker-engine lane delivers the **first concrete game that miners can solve and validators can score**. The user-visible outcomes are:

- A miner can create an `NlheSolver`, train it for N iterations, checkpoint it to disk, and serve `StrategyQuery` → `StrategyResponse` over the network
- A validator can send a `StrategyQuery` to a miner axon, receive an action distribution, and compute exploitability in milli-big-blinds per hand (mbb/h)

The lane sits between `games:traits` (which defines the generic `StrategyQuery`/`StrategyResponse` types) and the miner/validator service binaries (which call the solver).

---

## How Surfaces Fit Together

```
Miner binary                          Validator binary
    │                                       │
    ▼                                       │
PokerSolver                           remote_poker_exploitability
  ├─ NlheSolver (robopoker)                 │   (query_fn)
  ├─ NlheProfile                            │
  ├─ NlheEncoder                            │
  │                                         │
  ▼                                         │
handle_query(WireStrategy)                  │
  ├─ deserialize NlheInfo ◄──────────────────┼── WireStrategy{info_bytes}
  ├─ solver.strategy(&info)                 │
  └─ serialize Vec<(NlheEdge, Prob)>        │
       │                                    │
       ▼                                    │
WireStrategy{actions} ───────────────────────┘
  (axon response)
```

**Solver wrapper** (`src/solver.rs`): Owns `NlheSolver`, exposes `train(iterations)`, `strategy(&NlheInfo)`, `exploitability()`, `epochs()`, `save/load`.

**Query handler** (`src/query.rs`): Owns `handle_query(&WireStrategy) -> Result<WireStrategy>`. Stateless over PokerSolver; delegates to `strategy()`.

**Wire serialization** (`src/wire.rs`): Owns bincode roundtrip for `NlheInfo` and `NlheEdge`. Requires `serde` feature on robopoker crates.

**Exploitability** (`src/exploit.rs`): Owns `poker_exploitability` and `remote_poker_exploitability`. Uses `Tree::build` + `profile.exploitability()`.

**Training interface** (`src/training.rs`): Owns `TrainingSession` for batch iteration + checkpoint management.

---

## Currently Trusted Inputs

| File | Trust Signal |
|------|-------------|
| `crates/myosu-games/Cargo.toml` | Git dependency on robopoker at rev `04716310143094ab41ec7172e6cea5a2a66744ef`; workspace member; `cargo test -p myosu-games` passes |
| `crates/myosu-games/src/traits.rs` | All 10 unit tests + 4 doctests pass |
| `/home/r/coding/robopoker/crates/nlhe/src/lib.rs` | `Flagship` type alias is `pub type Flagship` — publicly reachable as `rbp_nlhe::Flagship` |
| `/home/r/coding/robopoker/crates/nlhe/src/info.rs` | `NlheInfo` derives serde conditionally (`#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`); `serde` feature must be enabled |
| `/home/r/coding/robopoker/crates/nlhe/src/edge.rs` | `NlheEdge` same serde conditional |
| `/home/r/coding/robopoker/crates/mccfr/src/lib.rs` | `pub use regret::*`, `pub use policy::*`, `pub use sample::*` — `PluribusRegret`, `LinearWeight`, `PluribusSampling` reachable via `rbp_mccfr::*` |
| `specsarchive/031626-02b-poker-engine.md` | Source spec AC-PE-01..04; describes solver wrapper, query, wire, exploitability |
| `specsarchive/031626-14-poker-variant-family.md` | Documents future variants; NLHE HU is prerequisite |

---

## Current Broken / Missing Surfaces

### Critical: `myosu-games-poker` Crate Does Not Exist

`crates/myosu-games-poker/` is **not present** in the workspace. All submodules (`solver.rs`, `query.rs`, `wire.rs`, `exploit.rs`, `training.rs`) are unimplemented.

**Impact**: No poker integration can exist without this crate. This is the primary blocker.

### Critical: `serde` Feature Must Be Verified on Robopoker Crates

`NlheInfo` and `NlheEdge` only derive `Serialize`/`Deserialize` when the `serde` feature is enabled on robopoker crates. The current `Cargo.toml` for `myosu-games` does not specify feature flags for `rbp-core` or `rbp-mccfr`. If `serde` is not enabled, wire serialization (AC-PE-03) will fail to compile.

**Mitigation**: The first slice must verify `serde` feature availability and add it to the dependency declaration if missing.

### Missing: `myosu-games-poker` Not in Workspace Members

`Cargo.toml` workspace members only list `crates/myosu-games` and `crates/myosu-tui`. `crates/myosu-games-poker` must be added.

### Missing: Checkpoint Format Not Implemented

The spec calls for a 4-byte `MYOS` magic + u32 version + bincode profile serialization. Unimplemented.

---

## Code Boundaries and Deliverables

### Crate Structure

```
crates/myosu-games-poker/
├── Cargo.toml              # Workspace member; depends on rbp-nlhe, rbp-mccfr (git), myosu-games
└── src/
    ├── lib.rs              # Re-exports public API
    ├── solver.rs           # PokerSolver, epoch management, save/load with MYOS checkpoint format
    ├── query.rs            # handle_query(WireStrategy) -> WireStrategy
    ├── wire.rs             # bincode roundtrip for NlheInfo and NlheEdge
    ├── exploit.rs          # poker_exploitability, remote_poker_exploitability
    └── training.rs         # TrainingSession, batch iteration + checkpoint management
```

### Public API Surface

```rust
// lib.rs
pub use solver::{PokerSolver, Flagship};
pub use query::handle_query;
pub use wire::{WireSerializable, Poker};
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use training::TrainingSession;

// Re-export from myosu-games for convenience
pub use myosu_games::{StrategyQuery, StrategyResponse, GameType, GameConfig};
```

---

## Proof / Check Shape

### Bootstrap Proof (lane integrity check)

```bash
# After myosu-games-poker exists:
cargo build -p myosu-games-poker
cargo test -p myosu-games-poker
```

### Milestone Checks (AC-PE-01 through PE-04)

| Milestone | Validates | AC |
|-----------|-----------|-----|
| Create PokerSolver, train 100 iterations, query strategy | Solver lifecycle | PE-01 |
| Save checkpoint, load, verify same epoch count | Persistence | PE-01 |
| Serialize NlheInfo → bytes → deserialize → identical | Wire format | PE-03 |
| Handle WireStrategy query → valid response | Query handler | PE-02 |
| Trained strategy has lower exploitability than random | Scoring | PE-04 |
| Full pipeline: train → serve → query → score | End-to-end | All |

### Required Test Inventory

```
cargo test -p myosu-games-poker solver::tests::create_empty_solver
cargo test -p myosu-games-poker solver::tests::train_100_iterations
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip
cargo test -p myosu-games-poker solver::tests::exploitability_decreases
cargo test -p myosu-games-poker query::tests::handle_valid_query
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one
cargo test -p myyosu-games-poker wire::tests::nlhe_info_roundtrip
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit
cargo test -p myosu-games-poker exploit::tests::remote_matches_local
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1 — Create `myosu-games-poker` Crate Skeleton

**Files**: `crates/myosu-games-poker/Cargo.toml`, `crates/myosu-games-poker/src/lib.rs`

Add crate to workspace members. Cargo.toml:
- Dependency on `rbp-nlhe` and `rbp-mccfr` (git) at same rev as `myosu-games` (`04716310143094ab41ec7172e6cea5a2a66744ef`)
- **Verify and enable `serde` feature** on the robopoker crates (needed for AC-PE-03)
- Dependency on `myosu-games`
- `crate-type = ["lib"]`

`lib.rs` re-exports stub types initially.

**Critical first check**: Attempt `cargo build -p myosu-games-poker --features rbp_nlhe/serde` to confirm serde works. If not, the wire slice (AC-PE-03) is blocked.

**Proof**: `cargo build -p myosu-games-poker` exits 0 with empty lib.

### Slice 2 — `solver.rs`: `PokerSolver` + Checkpoint Format

**File**: `crates/myosu-games-poker/src/solver.rs`

Implement `PokerSolver` wrapping `rbp_nlhe::Flagship`. Implement `save(path)` with `MYOS` magic + version + bincode. Implement `load(path)` with version verification. Implement `train(iterations)`, `strategy(&NlheInfo)`, `exploitability()`, `epochs()`.

**Proof**: `cargo test -p myosu-games-poker solver::tests::exploitability_decreases`

### Slice 3 — `wire.rs`: bincode roundtrip for `NlheInfo`/`NlheEdge`

**File**: `crates/myosu-games-poker/src/wire.rs`

Implement bincode roundtrip. Requires `serde` feature confirmed in Slice 1.

**Proof**: `cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip`

### Slice 4 — `query.rs`: `handle_query` Bridge

**File**: `crates/myosu-games-poker/src/query.rs`

Implement `handle_query(&WireStrategy) -> Result<WireStrategy>`. Depends on wire (Slice 3) and solver (Slice 2).

**Proof**: `cargo test -p myosu-games-poker query::tests::handle_valid_query`

### Slice 5 — `exploit.rs`: Exploitability Computation

**File**: `crates/myosu-games-poker/src/exploit.rs`

Implement `poker_exploitability(profile, encoder)` and `remote_poker_exploitability(query_fn, encoder)`. Depends on solver (Slice 2).

**Proof**: `cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit`

### Slice 6 — `training.rs`: `TrainingSession`

**File**: `crates/myosu-games-poker/src/training.rs`

Wrap solver for batch training with configurable checkpoint frequency.

**Proof**: `cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency`

---

## Dependency Order (Phase Gate)

```
games:traits  ──(git dep on robopoker, stable)──►  [myosu-games-poker slices 1-6]
     │
     └── path──►git migration already done ✓
```

Slice 1 (crate skeleton) depends on `games:traits` being stable — it is. The git dep migration is already complete.

Slices 2–6 must be done in order: PE-01 (solver) before PE-02 (query) or PE-04 (exploit); PE-03 (wire) before PE-02.

---

## What the Implementation Lane May Change First

The implementation lane is **allowed to change first**:

1. The `myosu-games-poker` crate structure and all source files (greenfield)
2. The `Flagship` type alias (can substitute `VanillaRegret` / `UniformWeight` / `ExternalSampling` for faster debug iteration — the public API is unchanged)
3. Checkpoint format (magic bytes, version number — not on critical path)
4. `serde` vs. another format (only affects wire.rs)

The implementation lane must **not change first** (requires wider coordination):

- The `CfrGame`/`CfrEdge`/`Profile` trait signatures from robopoker
- The `StrategyQuery`/`StrategyResponse` wire format (owned by `games:traits`)
- The robopoker git rev used by `myosu-games` (changing it breaks compatibility)
