# `games:poker-engine` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Greenfield `myosu-games-poker` Crate**

Created the `myosu-games-poker` crate wrapping robopoker's `rbp_nlhe::Flagship` NLHE MCCFR solver with full training, query, and persistence infrastructure.

## What Changed

### New Files

| File | Purpose |
|------|---------|
| `crates/myosu-games-poker/Cargo.toml` | Crate manifest with robopoker git dependency |
| `crates/myosu-games-poker/src/lib.rs` | Public API re-exports |
| `crates/myosu-games-poker/src/solver.rs` | `PokerSolver` wrapper with training + checkpoint |
| `crates/myosu-games-poker/src/wire.rs` | bincode serialization for `NlheInfo`/`NlheEdge` |
| `crates/myosu-games-poker/src/query.rs` | `handle_query` bridge for miner-validator |
| `crates/myosu-games-poker/src/exploit.rs` | Exploitability computation (local + remote) |
| `crates/myosu-games-poker/src/training.rs` | `TrainingSession` with checkpoint management |

### `Cargo.toml` (workspace root)

Added `crates/myosu-games-poker` to `members` array.

### `crates/myosu-games-poker/Cargo.toml`

```toml
[dependencies]
myosu-games = { path = "../myosu-games" }
serde = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
bincode = "1.3"
rbp-nlhe = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef", features = ["serde"] }
rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef", features = ["serde"] }
rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }

[dev-dependencies]
tempfile = "3"
```

## Key Implementation Details

### `PokerSolver` (solver.rs)

- Wraps `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`
- Training via `step()` (MCCFR iteration)
- Strategy queries via `strategy(&NlheInfo)` returning `Vec<(NlheEdge, Probability)>`
- Exploitability via `TreeBuilder` + `profile.exploitability()`
- Checkpoint format: `b"MYOS"` (4 bytes) + u32 version + bincode profile

### Wire Serialization (wire.rs)

- `WireSerializable` trait provides `to_bytes()`/`from_bytes()` via bincode
- `NlheInfo` and `NlheEdge` implement `WireSerializable`
- Extension trait `WireEncode` for typed encoding/decoding

### Query Bridge (query.rs)

- `handle_query(&StrategyQuery<Vec<u8>>, &PokerSolver)` → `StrategyResponse<u64>`
- Deserializes `NlheInfo` from wire format, queries solver, returns (encoded_edge, probability) pairs
- Validates probability distribution sums to 1.0

### Exploitability (exploit.rs)

- `poker_exploitability(&PokerSolver)` — local computation via profile's `exploitability()` method
- `remote_poker_exploitability(query_fn, encoder)` — queries remote source and computes synthetic profile exploitability
- Returns utility scaled to mbb/h (milli-big-blinds per hand)

### Training Session (training.rs)

- `TrainingSession` manages iterative training with configurable checkpoint frequency
- `TrainingConfig` builder for `max_iterations`, `checkpoint_every`, `checkpoint_dir`
- Automatic checkpoint loading on resume

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 2 | Add integration tests with real MCCFR game | Pending |
| Slice 3 | Add `GameType`/`GameConfig` plumbing for miner protocol | Pending |
| Slice 4 | Add streaming checkpoint support for large profiles | Pending |
