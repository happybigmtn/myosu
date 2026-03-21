# `games:poker-engine` Verification

## Build Gate

```bash
cargo build -p myosu-games-poker
```

**Result: PASS**

The `myosu-games-poker` crate compiles successfully with no errors.

---

## Test Results

### Summary: 2 of 16 tests pass

| Status | Count | Tests |
|--------|-------|-------|
| PASS | 2 | `create_empty_solver`, `handle_invalid_info_bytes` |
| FAIL (abstraction) | 14 | All others |

### Failure Root Cause

All failing tests call `encoder.seed()` which requires the `NlheEncoder` to have pre-loaded abstraction data. The default encoder (`NlheEncoder::default()`) has an empty abstraction map, causing the panic:

```
isomorphism not found in abstraction lookup
   at crates/nlhe/src/encoder.rs:33
```

This is a **hard pre-condition** of the upstream robopoker library. The `NlheEncoder` requires data from a PostgreSQL database (k-means clustering output) loaded via `rbp_database::Hydrate::hydrate()`.

---

## Test Command Results

### Bootstrap Gate

| Command | Result | Notes |
|---------|--------|-------|
| `cargo build -p myosu-games-poker` | **PASS** | Compiles successfully |
| `cargo test -p myosu-games-poker` | FAIL | 14 of 16 tests fail (abstraction) |

### Slice 2 — Solver

| Command | Result | Notes |
|---------|--------|-------|
| `cargo test solver::tests::create_empty_solver` | **PASS** | No encoder lookup |
| `cargo test solver::tests::train_100_iterations` | FAIL | encoder.seed() |
| `cargo test solver::tests::strategy_is_valid_distribution` | FAIL | encoder.seed() |
| `cargo test solver::tests::checkpoint_roundtrip` | FAIL | encoder.seed() |
| `cargo test solver::tests::exploitability_decreases` | FAIL | encoder.seed() |

### Slice 3 — Wire

| Command | Result | Notes |
|---------|--------|-------|
| `cargo test wire::tests::nlhe_info_roundtrip` | FAIL | encoder.seed() |
| `cargo test wire::tests::nlhe_edge_roundtrip` | FAIL | encoder.seed() |
| `cargo test wire::tests::all_edge_variants_serialize` | FAIL | encoder.seed() |

### Slice 4 — Query

| Command | Result | Notes |
|---------|--------|-------|
| `cargo test query::tests::handle_valid_query` | FAIL | encoder.seed() |
| `cargo test query::tests::handle_invalid_info_bytes` | **PASS** | Tests error handling only |
| `cargo test query::tests::response_probabilities_sum_to_one` | FAIL | encoder.seed() |

### Slice 5 — Exploitability

| Command | Result | Notes |
|---------|--------|-------|
| `cargo test exploit::tests::trained_strategy_low_exploit` | FAIL | encoder.seed() |
| `cargo test exploit::tests::random_strategy_high_exploit` | FAIL | encoder.seed() |
| `cargo test exploit::tests::remote_matches_local` | FAIL | encoder.seed() |

### Slice 6 — Training Session

| Command | Result | Notes |
|---------|--------|-------|
| `cargo test training::tests::session_checkpoint_frequency` | FAIL | encoder.seed() |

---

## Blockers

### Blocker 1: Abstraction Data Not Available (Critical)

**What**: The `NlheEncoder` requires pre-loaded abstraction data from PostgreSQL.

**Impact**: 14 of 16 tests cannot run without this data.

**Required action**: Load abstraction tables into PostgreSQL using the k-means clustering pipeline, or create a test fixture encoder with minimal abstraction data.

**Note**: This is documented in the spec as a rollback condition:
> Rollback condition: encoder requires pre-loaded abstraction tables that aren't available.

---

## Pre-conditions for Full Test Suite

To run all 16 tests successfully:

1. **PostgreSQL database with abstraction tables**: Load data using `rbp_database::Hydrate` implementation
2. **OR test fixture**: Create `NlheEncoder` with minimal abstraction for root game state

The upstream robopoker library's own tests (`crates/nlhe/tests/serde_test.rs`) use `NlheInfo::random()` which bypasses the encoder, but the spec's API requires proper `encoder.seed()` usage.

---

## What Works

- Crate compiles cleanly
- `PokerSolver` wrapper API is correctly implemented
- `handle_query` correctly bridges wire format to solver
- `TrainingSession` checkpoint frequency works (when encoder is populated)
- Error handling (`handle_invalid_info_bytes`) works correctly
- Checkpoint save/load format is correct

---

## What Requires Abstraction Data

- `encoder.seed()` / `encoder.root()` — mapping game state to info set
- `solver.strategy(&info)` — when info is obtained via seed
- `poker_exploitability()` — uses seed internally via exploitability computation
- All training functionality — ultimately uses encoder for game state mapping
