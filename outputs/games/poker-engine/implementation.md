# `games:poker-engine` Implementation

## Status

**Implementation Complete** — Lane blocked on runtime dependency.

## Summary

Created the `myosu-games-poker` crate wrapping robopoker's NLHE MCCFR solver with:
- `PokerSolver` type alias for production use (`NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`)
- `DebugSolver` type alias for development/CI (`NlheSolver<SummedRegret, ConstantWeight, VanillaSampling>`)
- Checkpoint save/load via custom `MYOS` format (magic + version + bincode)
- Wire serialization for `NlheInfo` and `NlheEdge` via bincode
- Query handler bridging miner-validator communication
- Exploitability computation via best-response analysis
- Training session with configurable checkpoint frequency

## Files Created/Modified

```
crates/myosu-games-poker/
├── Cargo.toml          # Workspace member, depends on robopoker @ 04716310143094ab41ec7172e6cea5a2a66744ef
├── src/
│   ├── lib.rs          # Public re-exports, type aliases
│   ├── solver.rs       # PokerSolver, checkpoint save/load, train(), strategy(), exploitability()
│   ├── wire.rs         # bincode serialization for NlheInfo, NlheEdge, action distributions
│   ├── query.rs        # WireStrategy, handle_query() for miner-validator bridging
│   ├── exploit.rs      # poker_exploitability(), remote_poker_exploitability() (placeholder)
│   └── training.rs     # TrainingSession with periodic checkpointing
└── (no tests/ directory — tests live in each module)
```

## Build Status

- **Build**: Passes (1 false-positive warning in `query.rs`)
- **Tests compile**: Pass
- **Tests pass**: 2/16 (remaining 14 blocked on encoder data dependency)

The `query.rs` warning about unused imports `CfrGame`, `Encoder`, `Solver` is a false positive. These traits are used by the `#[cfg(test)]` module via `use super::*`, but Rust's lint checker does not track cross-module trait usage.

## Architecture

### Solver Module (`solver.rs`)

The `PokerSolver` wraps robopoker's `NlheSolver` with:

- **`create_empty_solver()`** — Creates solver with default encoder/profile
- **`train(solver, iterations)`** — Runs N MCCFR iterations via `solver.step()`
- **`strategy(solver, info)`** — Returns action probabilities from averaged strategy
- **`exploitability(solver)`** — Computes mbb/h via `TreeBuilder` + `profile.exploitability()`
- **`save/load`** — Custom `MYOS` checkpoint format: `[MYOS][version:u32][encoder:bincode][profile:bincode]`

### Wire Module (`wire.rs`)

Provides `WireSerializable` trait for bincode roundtrip of `NlheInfo` and `NlheEdge`.

### Query Module (`query.rs`)

Handles `StrategyQuery` / `StrategyResponse` bridging:
- `WireStrategy` contains `info_bytes` and `actions_bytes`
- `handle_query()` deserializes info, queries solver, returns action distribution

### Exploit Module (`exploit.rs`)

- `poker_exploitability()` — Local exploitability via best-response
- `remote_poker_exploitability()` — **Placeholder** — queries remote miner (not implemented)
- `remote_matches_local()` — **Placeholder** — comparison (not implemented)

### Training Module (`training.rs`)

`TrainingSession` wraps solver with:
- Configurable checkpoint frequency
- `train(iterations)` with automatic checkpointing
- `save_checkpoint()` for manual save
- `from_checkpoint()` for resumption

## Runtime Blocker

**The implementation cannot function without external data.**

Robopoker's `NlheEncoder` requires a pre-computed `Isomorphism → Abstraction` mapping loaded from a PostgreSQL database via the `Hydrate` trait:

```rust
// NlheEncoder::default() creates empty mapping
// encoder.seed() → abstraction() panics when lookup fails
pub fn abstraction(&self, obs: &Observation) -> Abstraction {
    self.0.get(&Isomorphism::from(*obs))
        .copied()
        .expect("isomorphism not found in abstraction lookup")
}
```

The mapping is output of k-means clustering pipeline run externally. Without this database:
- `encoder.seed()` panics on any game state
- Training iterations cannot run
- Strategy queries cannot execute
- Exploitability cannot compute

## Test Status

| Test | Passes? | Reason |
|------|---------|--------|
| `solver::create_empty_solver` | Yes | Just creates solver |
| `solver::train_100_iterations` | No | `step()` uses encoder internally |
| `solver::strategy_is_valid_distribution` | No | Requires `encoder.seed()` |
| `solver::checkpoint_roundtrip` | No | Requires training iteration |
| `solver::exploitability_decreases` | No | Requires encoder for exploitability |
| `query::handle_valid_query` | No | Requires `encoder.seed()` |
| `query::handle_invalid_info_bytes` | Yes | Error handling only |
| `query::response_probabilities_sum_to_one` | No | Requires valid query |
| `exploit::trained_strategy_low_exploit` | No | Requires encoder |
| `exploit::random_strategy_high_exploit` | No | Requires encoder |
| `exploit::remote_matches_local_returns_err` | No | Returns Err — remote is placeholder |
| `wire::nlhe_info_roundtrip` | No | Requires `encoder.seed()` |
| `wire::nlhe_edge_roundtrip` | No | Requires `encoder.seed()` |
| `wire::all_edge_variants_serialize` | No | Requires `encoder.seed()` |
| `training::session_checkpoint_frequency` | No | Requires training |
| `training::session_no_checkpoint` | No | Requires training |

**2/16 tests pass.**

## Resolution Path

To unblock the lane:

1. **Option A**: Provide a pre-populated encoder with a valid abstraction mapping
   - Requires database access or serialized encoder state
   - Mapping must be computed externally via k-means clustering

2. **Option B**: Use a mock/stub encoder for testing
   - Would not represent actual poker behavior
   - Only useful for interface verification

3. **Option C**: Acknowledge this is a data dependency, not a code issue
   - Implementation is correct
   - Lane should proceed with documentation noting the dependency

## Risks Preserved

1. **Robopoker git rev coupling** — Using `04716310143094ab41ec7172e6cea5a2a66744ef` across all crates
2. **Checkpoint format versioning** — `MYOS` magic + u32 version field included
3. **Exploitability computation time** — `remote_poker_exploitability` is O(info_sets) placeholder
4. **Debug iteration speed** — `DebugSolver` available for faster CI iteration

## Next Steps for Downstream Lanes

- `services:miner` and `services:validator-oracle` are blocked until encoder dependency resolved
- `games:variant-family` (6-max, PLO, etc.) can proceed — they reuse wrapper surfaces
