# Implementation: games:poker-engine (Slice 1-6)

## Overview

Created `myosu-games-poker` crate implementing NLHE poker solver functionality with checkpoint persistence, wire serialization, query handling, exploitability computation, and training session management.

## Crate Structure

```
crates/myosu-games-poker/
├── Cargo.toml          # Dependencies: rbp-*, serde, thiserror, bincode
└── src/
    ├── lib.rs          # Public API re-exports
    ├── solver.rs       # PokerSolver + MYOS checkpoint format
    ├── wire.rs         # WireSerializable trait for bincode roundtrip
    ├── query.rs        # handle_query function for miner-validator communication
    ├── exploit.rs      # poker_exploitability, remote_poker_exploitability
    └── training.rs     # TrainingSession with checkpointing
```

## Key Implementation Details

### Solver (`solver.rs`)

- Wraps `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`
- MYOS checkpoint format: 4-byte "MYOS" magic + u32 version + bincode(NlheProfile)
- Methods: `train()`, `strategy(&NlheInfo)`, `exploitability()`, `epochs()`, `save()`, `load()`

### Wire Serialization (`wire.rs`)

- `WireSerializable` trait for bincode roundtrip of `NlheInfo` and `NlheEdge`
- Requires `serde` feature on rbp-nlhe

### Query Handler (`query.rs`)

- `WireStrategy` struct with `info_bytes` and `action_bytes` (JSON outer, bincode inner)
- Stateless `handle_query()` function for miner-validator communication

### Exploitability (`exploit.rs`)

- `poker_exploitability(&Flagship)` - local solver
- `remote_poker_exploitability(&QueryFn, &NlheEncoder)` - remote solver via query function

### Training Session (`training.rs`)

- `TrainingSession` wraps `PokerSolver` with configurable checkpoint frequency
- Builder pattern for `TrainingConfig`

## Dependencies

```toml
rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "..." }
rbp-nlhe = { git = "...", features = ["serde"] }
rbp-mccfr = { git = "...", features = ["serde"] }
```

## Workspace Integration

Added to `Cargo.toml` workspace members.

## Notes

- Uses robopoker git dependency at specific rev for NLHE solver types
- Conditional serde via `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`
- Test infrastructure uses tempfile for checkpoint file handling
