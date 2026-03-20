# `games:poker-engine` Quality — Slice 1

## Complexity Assessment

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Abstraction surface | Low | Thin wrapper over robopoker primitives |
| State management | Low | `PokerSolver` owns single `NlheSolver` instance |
| Async/concurrency | None | Synchronous API, no shared state |
| External I/O | Low | File-based checkpoint save/load only |
| Error surface | Medium | Custom error enum + anyhow for propagation |

## Architecture Decisions

### 1. Checkpoint Format Versioned

```
Bytes 0-3:   b"MYOS" (magic)
Bytes 4-7:   u32 version (currently 1)
Bytes 8-15:  u64 profile length
Bytes 16+:   bincode-encoded NlheProfile
```

Version prefix enables future format evolution without breaking existing checkpoints.

### 2. Wire Serialization Uses bincode 1.x API

```rust
// serialize/deserialize (not encode_to_vec/decode_from_slice)
bincode::serialize(&self.solver.profile())
bincode::deserialize(&profile_data)
```

bincode 1.3 API changed method names. Using correct API ensures compatibility.

### 3. Strategy Query Returns u64-Encoded Edges

```rust
pub fn strategy(&self, info: &NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, Probability)>
```

Returns concrete `NlheEdge` type for internal use. Wire format converts to `u64` for transport (see `query.rs`).

### 4. Training Session Manages Checkpoint Lifecycle

`TrainingSession` wraps training loop with periodic checkpointing. `TrainingConfig` uses builder pattern for ergonomic configuration:

```rust
let session = TrainingSession::builder()
    .max_iterations(10_000)
    .checkpoint_every(1_000)
    .checkpoint_dir("./checkpoints")
    .build()?;
```

## Known Quality Issues

| Issue | Severity | Notes |
|-------|----------|-------|
| Dead code warnings | Low | `WireEncode` trait, `chips_to_mbbh`, `CorruptedFile` variant unused |
| Robopoker encoder panic | High | Standalone `NlheEncoder::seed()` panics — upstream bug |
| No integration tests | Medium | Tests use internal APIs; no end-to-end game flow tests |

## Dead Code Details

```rust
// solver.rs:42 — CorruptedFile variant never constructed
pub enum SolverError {
    CorruptedFile(String),  // future use for malformed checkpoints
}

// wire.rs:33 — WireEncode trait for future typed encoding
pub trait WireEncode { ... }  // wire.rs has standalone functions instead

// exploit.rs:73 — chips_to_mbbh utility function
pub fn chips_to_mbbh(utility: Utility) -> Utility { utility * 1000.0 }
```

These are intentional预留 (reserved for future use) and should not be removed until actually needed.

## Recommendations for Slice 2

1. **File robopoker bug report** — upstream issue for encoder initialization
2. **Add integration test with game flow** — use `PokerSolver` through full game lifecycle
3. **Consider removing dead code** — `WireEncode` trait can be removed if not planned for use
4. **Add checkpoint migration** — when format version bumps, add migration path
