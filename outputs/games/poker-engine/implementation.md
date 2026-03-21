# `games:poker-engine` Implementation

## Status: Complete (Blocked on Abstraction Data)

The `myosu-games-poker` crate has been implemented according to the spec. All library code compiles successfully. Tests fail due to a pre-condition documented in the spec as a rollback condition: the `NlheEncoder` requires pre-loaded abstraction data from a PostgreSQL database.

---

## Files Created

```
crates/myosu-games-poker/
├── Cargo.toml           # Crate manifest with robopoker git dependency
└── src/
    ├── lib.rs           # Module declarations and public re-exports
    ├── solver.rs        # PokerSolver wrapper with train/save/load
    ├── wire.rs         # WireSerializable trait + WireStrategy
    ├── query.rs        # handle_query bridge
    ├── exploit.rs       # Exploitability computation
    └── training.rs      # TrainingSession with checkpointing
```

---

## Implementation Details

### PokerSolver (`solver.rs`)

Wrapper around `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`.

**Public API:**
- `new()` / `default()` — Creates solver with empty encoder/profile
- `from_parts(encoder, profile)` — Creates solver from existing state
- `train(iterations)` — Runs CFR training iterations
- `epochs()` — Returns current epoch count
- `strategy(&info)` — Returns action distribution for info set
- `exploitability()` — Computes mbb/h via full tree evaluation
- `encoder()` — Returns reference to encoder
- `save(path)` — Checkpoint to file (MYOS magic + version + JSON)
- `load(path)` — Restore from checkpoint file

**Checkpoint Format:**
```
4 bytes: "MYOS" magic
4 bytes: u32 version (1)
8 bytes: u64 payload length
N bytes: serde_json(encoder, profile)
```

### Wire Serialization (`wire.rs`)

`WireSerializable` trait provides JSON roundtrip for `NlheInfo` and `NlheEdge`.

`WireStrategy` struct:
```rust
pub struct WireStrategy {
    pub info_bytes: Vec<u8>,      // Serialized NlheInfo
    pub actions_bytes: Vec<u8>,    // Serialized Vec<(NlheEdge, Probability)>
}
```

### Query Handler (`query.rs`)

`handle_query(&WireStrategy, &PokerSolver) -> Result<WireStrategy, QueryError>`

Receives a query with serialized `NlheInfo`, looks up strategy via `solver.strategy()`, returns response with action distribution.

### Exploitability Computation (`exploit.rs`)

- `poker_exploitability(&PokerSolver) -> Utility` — Local exploitability via `solver.exploitability()`
- `remote_poker_exploitability(query_fn, encoder) -> Result<Utility>` — Remote computation via query function
- `profile_exploitability(encoder, profile) -> Utility` — Direct computation from profile

### Training Session (`training.rs`)

`TrainingSession` wraps `PokerSolver` with configurable checkpoint frequency:
- `new(checkpoint_every, checkpoint_dir)` — Creates session
- `from_solver(solver, ...)` — Creates from existing solver
- `train(iterations)` — Training with periodic checkpoints
- `save_checkpoint()` — Manual checkpoint save
- `load_latest_checkpoint()` — Restore most recent checkpoint

---

## Dependencies

```toml
rbp-nlhe = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef", features = ["serde"] }
rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef", features = ["serde"] }
myosu-games = { path = "../myosu-games" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tempfile = { workspace = true }  # dev dependency
```

---

## Test Status

**14 of 16 tests fail** due to missing abstraction data. The error:
```
isomorphism not found in abstraction lookup
```

This occurs in `encoder.seed()` / `encoder.root()` when the `NlheEncoder`'s internal `BTreeMap<Isomorphism, Abstraction>` is empty (as with `NlheEncoder::default()`).

The spec's rollback condition states:
> Rollback condition: encoder requires pre-loaded abstraction tables that aren't available.

**Passing tests:**
- `create_empty_solver` — Only checks epochs(), no encoder lookup
- `handle_invalid_info_bytes` — Tests error handling without encoder lookup

**Failing tests** — All call `encoder.seed()` which requires abstraction data:
- All exploitability tests
- All query tests
- All wire serialization tests
- Training session tests

---

## Abstraction Data Requirement

The `NlheEncoder` maintains a `BTreeMap<Isomorphism, Abstraction>` mapping suit-isomorphic hand representations to strategic abstraction buckets (k-means clustering output). This data is loaded via `rbp_database::Hydrate` from PostgreSQL:

```rust
// From rbp_nlhe::encoder.rs
impl rbp_database::Hydrate for NlheEncoder {
    async fn hydrate(client: Arc<Client>) -> Self {
        let sql = const_format::concatcp!("SELECT obs, abs FROM ", rbp_database::ISOMORPHISM);
        let lookup = client.query(sql, &[])...
    }
}
```

Without this data, the encoder cannot map game states to info sets. This is a hard pre-condition of the upstream library, not a bug in this implementation.

---

## Build Verification

```bash
cargo build -p myosu-games-poker  # ✓ Compiles successfully
```

Test compilation and execution requires either:
1. A PostgreSQL database with abstraction tables loaded, or
2. A test encoder fixture (not currently available in upstream)
