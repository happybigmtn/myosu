# `games:poker-engine` Verification

## Build Verification

```
$ cargo build -p myosu-games-poker
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
```

**Result**: PASS

## Test Results

```
$ cargo test -p myosu-games-poker -- --test-threads=1
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

## Test Breakdown

### Passing Tests (5)

| Test | Reason |
|------|--------|
| `query::tests::handle_invalid_info_bytes` | Doesn't call `train()` |
| `wire::tests::all_edge_variants_serialize` | Compile-time check |
| `wire::tests::nlhe_edge_roundtrip` | Compile-time check |
| `wire::tests::nlhe_info_roundtrip` | Uses `NlheInfo::default()` which doesn't require encoder lookup |
| `solver::tests::create_empty_solver` | Only constructs solver, no training |

### Failing Tests (10)

All failing tests call `train()` which triggers `NlheSolver::step()`. This calls `encoder.abstraction()` which panics because `NlheEncoder::default()` has no isomorphism→abstraction mappings.

**Root Cause**: `crates/nlhe/src/encoder.rs:33`
```rust
pub fn abstraction(&self, obs: &Observation) -> Abstraction {
    self.0.get(&Isomorphism::from(*obs))
        .copied()
        .expect("isomorphism not found in abstraction lookup")
}
```

The `NlheEncoder` requires database-backed abstraction mappings (loaded via `Hydrate` trait when `database` feature is enabled). Without the database, `default()` returns an empty map and lookups fail.

## Required for Full Test Pass

The tests require the `database` feature on `rbp-nlhe` to load isomorphism→abstraction mappings from PostgreSQL. This is external infrastructure not part of this slice.

## Individual Test Commands (from review.md)

```bash
cargo test -p myosu-games-poker solver::tests::create_empty_solver  # PASS
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip     # PASS
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip     # PASS
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize # PASS
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes # PASS
# Remaining tests require database-backed encoder
```

## Recommended Next Steps

1. **Database Integration**: Enable `database` feature on `rbp-nlhe` and configure PostgreSQL connection to load abstraction mappings
2. **Alternative**: Create test fixture with minimal encoder mappings for unit testing
3. **Integration Tests**: Add integration tests that run with the database feature enabled
