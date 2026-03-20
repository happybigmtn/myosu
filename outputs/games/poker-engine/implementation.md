# `games:poker-engine` Implementation

## Slice Implemented

**Slice 1-6: Full `myosu-games-poker` crate implementation**

This implementation delivers the complete `myosu-games-poker` crate as specified in `spec.md` and reviewed in `review.md`.

---

## What Was Built

### Crate Structure

```
crates/myosu-games-poker/
├── Cargo.toml              # Workspace member; git deps on robopoker at rev 04716310143094ab41ec7172e6cea5a2a66744ef
└── src/
    ├── lib.rs              # Public API re-exports
    ├── solver.rs           # PokerSolver wrapper + MYOS checkpoint format
    ├── query.rs            # handle_query bridge
    ├── wire.rs             # bincode roundtrip for NlheInfo/NlheEdge
    ├── exploit.rs          # Exploitability computation
    └── training.rs          # TrainingSession with checkpoint management
```

### Module Details

#### `solver.rs` — PokerSolver

- **Type**: `PokerSolver` wrapping `rbp_nlhe::Flagship = NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`
- **Creation**: `PokerSolver::new()` creates a solver with default profile and encoder
- **Training**: `train(iterations)` runs CFR iterations via `step()`
- **Strategy query**: `strategy(&NlheInfo)` returns `Vec<(NlheEdge, f64)>`
- **Exploitability**: `exploitability()` computes Nash equilibrium distance via `Solver::exploitability()`
- **Persistence**: `save(path)` / `load(path)` with 4-byte `MYOS` magic + u32 version + bincode
- **Epoch tracking**: `epochs()` returns total iterations trained

#### `wire.rs` — Serialization

- **WireSerializable trait**: Generic serialize/deserialize for `NlheInfo` and `NlheEdge`
- **Bincode format**: Direct encoding for network transport
- **Action distribution**: `serialize_action_distribution` / `deserialize_action_distribution`

#### `query.rs` — Query Handler

- **handle_query**: Stateless bridge taking `WireStrategy` (serialized `NlheInfo`) → returning `WireStrategy` (action distribution)
- **Validation**: `validate_info_bytes` for checking query validity
- **Response parsing**: `parse_response` for deserializing action distributions

#### `exploit.rs` — Exploitability

- **poker_exploitability**: Direct exploitability from solver (in mbb/h)
- **remote_poker_exploitability**: Approximate computation for remote strategies via sampling

#### `training.rs` — Training Session

- **TrainingConfig**: Configurable iterations per epoch, checkpoint frequency, checkpoint directory
- **TrainingSession**: Stateful trainer with auto-checkpointing and epoch tracking
- **resume_from**: Resume training from a checkpoint file

---

## Key Design Decisions

### 1. Checkpoint Format

The `MYOS` checkpoint format stores:
- 4 bytes: `MYOS` magic
- 4 bytes: version (u32 LE)
- Remaining: bincode-encoded `NlheProfile`

This allows version-aware loading and future format evolution.

### 2. Strategy Return Type

`strategy()` returns `Vec<(NlheEdge, f64)>` (not a custom type) for simplicity and compatibility with standard library collections. Probabilities are converted from internal `f32` to `f64` for API stability.

### 3. Encoder Initialization

`NlheEncoder::default()` creates an empty lookup table. In production, the encoder requires database hydration via `NlheEncoder::hydrate(client)`. This is consistent with robopoker's architecture where abstraction mappings are loaded from PostgreSQL.

---

## Dependencies

- **robopoker**: Git dependency at `04716310143094ab41ec7172e6cea5a2a66744ef` (same as `myosu-games`)
- **rbp-nlhe**: With `serde` feature enabled for `NlheInfo`/`NlheEdge` serialization
- **myosu-games**: For `StrategyQuery`/`StrategyResponse` types (though currently re-exported directly)

---

## Verified Compilation

```bash
cargo build -p myosu-games-poker  # Exit 0
```

All source files compile without errors. Some tests require database-hydrated encoders to pass fully.

---

## Limitations

1. **Database dependency for full testing**: The `exploitability()` method requires a database-hydrated `NlheEncoder` with isomorphism→abstraction mappings. Without this, tests that call `exploitability()` will panic. This is consistent with robopoker's architecture.

2. **Single variant only**: This implementation is for NLHE heads-up only (as specified). Extensions for 6-max, PLO, etc. belong to `games:variant-family`.

3. **No async training**: The current implementation uses synchronous `step()` calls. Production training would use the async hydrating trainer from robopoker.

---

## Alignment with Spec

| Spec Requirement | Implementation Status |
|-----------------|----------------------|
| `PokerSolver` wrapper | ✅ Complete |
| `train(iterations)` | ✅ Complete |
| `strategy(&NlheInfo)` | ✅ Complete |
| `exploitability()` | ✅ Complete |
| `epochs()` | ✅ Complete |
| `save/load` with MYOS format | ✅ Complete |
| `handle_query` bridge | ✅ Complete |
| bincode roundtrip | ✅ Complete |
| `TrainingSession` | ✅ Complete |
| serde feature on rbp-nlhe | ✅ Enabled |
| Workspace member added | ✅ Complete |
