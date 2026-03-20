# `games:poker-engine` Verification

## Build Verification

```
$ cargo build -p myosu-games-poker
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
```

**Result**: PASS

## Test Results

```
$ cargo test -p myosu-games-poker
running 15 tests
test exploit::tests::random_strategy_high_exploit ... FAILED
test exploit::tests::remote_matches_local ... FAILED
test exploit::tests::trained_strategy_low_exploit ... FAILED
test query::tests::handle_invalid_info_bytes ... ok
test query::tests::handle_valid_query ... FAILED
test query::tests::response_probabilities_sum_to_one ... FAILED
test solver::tests::checkpoint_roundtrip ... FAILED
test solver::tests::create_empty_solver ... ok
test solver::tests::exploitability_decreases ... FAILED
test solver::tests::strategy_is_valid_distribution ... FAILED
test solver::tests::train_100_iterations ... FAILED
test training::tests::session_checkpoint_frequency ... FAILED
test wire::tests::all_edge_variants_serialize ... ok
test wire::tests::nlhe_edge_roundtrip ... ok
test wire::tests::nlhe_info_roundtrip ... ok

result: 5 passed; 10 failed; 0 ignored
```

## Root Cause

All 10 failing tests invoke `train()`, which calls `NlheSolver::step()`. This triggers tree building via `TreeBuilder`, which calls `encoder.seed()` → `encoder.root()` → `encoder.abstraction()`. The `abstraction()` method performs a lookup in `NlheEncoder`'s internal `BTreeMap<Isomorphism, Abstraction>`:

```rust
// robopoker crates/nlhe/src/encoder.rs:29-34
pub fn abstraction(&self, obs: &Observation) -> Abstraction {
    self.0
        .get(&Isomorphism::from(*obs))
        .copied()
        .expect("isomorphism not found in abstraction lookup")
}
```

`NlheEncoder::default()` returns an empty `BTreeMap` — the `#[derive(Default)]` on the struct provides only zero-value initialization. The isomorphism→abstraction mapping **must** be loaded from PostgreSQL via the `Hydrate` trait (available when `database` feature is enabled on `rbp-nlhe`).

**Structural constraint**: `NlheEncoder` has no public constructor accepting mappings. The internal `BTreeMap` field is private (not `pub`). The only entry point for a populated encoder is `rbp_database::Hydrate::hydrate()`, which requires an async PostgreSQL connection.

## Test Breakdown

### Passing Tests (5)

| Test | Why It Passes |
|------|---------------|
| `solver::tests::create_empty_solver` | Only constructs solver; no `train()` call |
| `query::tests::handle_invalid_info_bytes` | No `train()` call |
| `wire::tests::nlhe_info_roundtrip` | Uses `NlheInfo::default()`; no encoder lookup |
| `wire::tests::nlhe_edge_roundtrip` | Compile-time `Serialize`/`Deserialize` check |
| `wire::tests::all_edge_variants_serialize` | Compile-time check |

### Failing Tests (10)

All call `train()` → `step()` → tree building → encoder lookup → panic.

## Automated Proof Commands Summary

| Command | Outcome |
|---------|---------|
| `cargo build -p myosu-games-poker` | PASS |
| `cargo test -p myosu-games-poker` | 5 PASS, 10 FAIL |

Individual test commands:

```bash
cargo test -p myosu-games-poker solver::tests::create_empty_solver     # PASS
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip       # PASS
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip       # PASS
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize # PASS
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes  # PASS
cargo test -p myosu-games-poker solver::tests::train_100_iterations     # FAIL
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution # FAIL
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip     # FAIL
cargo test -p myosu-games-poker solver::tests::exploitability_decreases  # FAIL
cargo test -p myosu-games-poker query::tests::handle_valid_query        # FAIL
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one # FAIL
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit # FAIL
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit # FAIL
cargo test -p myosu-games-poker exploit::tests::remote_matches_local     # FAIL
cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency # FAIL
```

## Unblock Path

**Status**: `database` feature is now enabled in `myosu-games-poker/Cargo.toml`.

The tests require **PostgreSQL infrastructure** with the abstraction schema initialized (`rbp_database::ISOMORPHISM` table). The `database` feature enables the `Hydrate` trait for loading encoder mappings, but the data must still come from a PostgreSQL instance containing the k-means clustering results.

To run the full test suite:
```bash
# 1. Set up PostgreSQL and run k-means clustering pipeline (external to this slice)
# 2. Set DB_URL environment variable
export DB_URL="postgres://user:pass@host:port/db"
cargo test -p myosu-games-poker
```

**Infrastructure requirements** (outside `games:poker-engine` slice scope):
- PostgreSQL instance
- `rbp_database::ISOMORPHISM` table populated via k-means clustering
- `rbp_database::ABSTRACTION`, `rbp_database::STREET`, `rbp_database::BLUEPRINT` tables

## Fixup Applied

The `database` feature has been enabled on `rbp-nlhe` in `myosu-games-poker/Cargo.toml`. This unlocks:
- `rbp_database::Hydrate` trait availability for `NlheEncoder` and `NlheProfile`
- Async encoder hydration from PostgreSQL when `DB_URL` is set
- The full test suite can pass once PostgreSQL infrastructure is available

## Relationship to Pre-existing Documentation

This verification confirms and granularizes the known limitation documented in `implementation.md`:

> The NLHE solver (`rbp_nlhe::Flagship`) requires database-backed isomorphism→abstraction mappings to function. `NlheEncoder::default()` creates an empty mapping, causing `train()` to panic at `encoder.rs:33` with "isomorphism not found in abstraction lookup".

The failure is **deterministic** and **expected** without PostgreSQL infrastructure. The implementation code is correct per the spec; the test suite requires environment-level infrastructure that is not part of this slice.

**Fixup update**: The `database` feature is now enabled, making the PostgreSQL hydration path available. The tests still require a PostgreSQL instance with k-means data to pass.