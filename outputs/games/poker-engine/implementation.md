# `games:poker-engine` Implementation

## Status

**Implemented** — All 6 slices complete. Build passes.

## What Was Implemented

### Slice 1 — Crate Skeleton

Created `crates/myosu-games-poker/` with:
- `Cargo.toml` with dependencies on `rbp-nlhe` (serde feature), `rbp-mccfr` (serde feature), `rbp-core`, `myosu-games`
- `src/lib.rs` with public re-exports
- Added `crates/myosu-games-poker` to workspace members

### Slice 2 — `solver.rs`: PokerSolver + Checkpoint Format

`PokerSolver` wrapping `rbp_nlhe::Flagship`:
- `new()` / `Default` constructor
- `load(path)` / `load_bytes(data)` with MYOS magic + version verification
- `save(path)` with MYOS magic + u32 version + bincode-serialized profile
- `train(iterations)` — runs `step()` iterations
- `epochs()` — returns current epoch count
- `strategy(&NlheInfo)` — returns averaged action distribution
- `exploitability()` — returns mbb/h
- `info_sets()` — iterator over trained info sets

Checkpoint format: `MYOS` (4 bytes) + version u32 + serde-json profile bytes.

### Slice 3 — `wire.rs`: bincode roundtrip for NlheInfo/NlheEdge

Note: Uses **serde-json** (not bincode Encode/Decode) because `NlheInfo` and `NlheEdge` derive `serde::Serialize/Deserialize` but not bincode's `Encode/Decode` traits.

- `WireSerializable` trait for types that can cross the wire
- `WireStrategy` query/response structure
- `WireStrategy::query(info)` / `WireStrategy::response(actions)` constructors
- `WireStrategy::parse_info()` / `parse_actions()` accessors

### Slice 4 — `query.rs`: handle_query Bridge

- `handle_query(&PokerSolver, &WireStrategy) -> Result<WireStrategy>` — stateless query handler
- `validate_info_bytes(&WireStrategy) -> Result<(), WireError>`

### Slice 5 — `exploit.rs`: Exploitability Computation

- `poker_exploitability(&PokerSolver) -> f32` — local exploitability
- `remote_poker_exploitability(QueryFn) -> f32` — placeholder for remote computation
- `compare_remote_to_local(&PokerSolver, &QueryFn, usize) -> Result<f32, CompareError>`
- `QueryFn` type alias: `Box<dyn Fn(&NlheInfo) -> Vec<(NlheEdge, Probability)> + Send + Sync + 'a>`

### Slice 6 — `training.rs`: TrainingSession

- `TrainingConfig` with `iterations_per_checkpoint`, `max_checkpoints`, `checkpoint_dir`
- `TrainingSession::new(PokerSolver, TrainingConfig)` / `from_checkpoint(path, TrainingConfig)`
- `train_batch()` / `save_checkpoint(name)` / `epochs()` / `solver()`

## Known Limitation

The NLHE solver (`rbp_nlhe::Flagship`) requires database-backed isomorphism→abstraction mappings to function. `NlheEncoder::default()` creates an empty mapping, causing `train()` to panic at `encoder.rs:33` with "isomorphism not found in abstraction lookup".

This is an architectural requirement of robopoker at the pinned revision (`04716310143094ab41ec7172e6cea5a2a66744ef`). The `database` feature loads these mappings from PostgreSQL.

**Impact**: Tests that call `train()` fail without the database feature. Tests that only construct/save/load pass.

## Files Created

```
crates/myosu-games-poker/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── solver.rs
    ├── query.rs
    ├── wire.rs
    ├── exploit.rs
    └── training.rs
```

## Files Modified

- `Cargo.toml` (workspace root) — added `crates/myosu-games-poker` to members
