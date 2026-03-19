# `games:poker-engine` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The lane is correctly scoped, well-specified, and blocked only on implementation work — not on specification or design decisions. All source materials are coherent and internally consistent. No blockers require reopening the spec.

---

## Proof Expectations

The following commands must all exit 0 before the lane is considered complete:

```bash
# Bootstrap gate (lane integrity)
cargo build -p myosu-games-poker
cargo test -p myosu-games-poker

# Slice 2 — solver
cargo test -p myosu-games-poker solver::tests::create_empty_solver
cargo test -p myosu-games-poker solver::tests::train_100_iterations
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip
cargo test -p myosu-games-poker solver::tests::exploitability_decreases

# Slice 3 — wire
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize

# Slice 4 — query
cargo test -p myosu-games-poker query::tests::handle_valid_query
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one

# Slice 5 — exploitability
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit
cargo test -p myosu-games-poker exploit::tests::remote_matches_local

# Slice 6 — training session
cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency
```

---

## Remaining Blockers

### Blocker 1: `myosu-games-poker` Crate Is Entirely Greenfield (Critical)

**Location**: `crates/myosu-games-poker/` does not exist.

**What must happen**: The implementation lane must create all source files. This is normal for a bootstrap lane — not a design problem.

**Risk if ignored**: Lane cannot progress at all.

### Blocker 2: `serde` Feature on Robopoker Crates (High)

**Location**: `crates/myosu-games-poker/Cargo.toml` — robopoker dependency declaration.

**What must happen**: The first slice must verify that `rbp-nlhe` and its transitive dependencies expose serde-serializable types when the `serde` feature is enabled. The conditional `#[cfg_attr(feature = "serde", derive(...))]` on `NlheInfo` and `NlheEdge` means serialization will silently fail to compile if the feature is not enabled.

**Risk if ignored**: Slice 3 (wire serialization) will fail to compile with a type-error, not a clear feature-flag error. This is a confusing failure mode.

### Blocker 3: `myosu-games-poker` Not in Workspace Members (Medium)

**Location**: `Cargo.toml` workspace root.

**What must happen**: Add `crates/myosu-games-poker` to workspace `members`.

**Risk if ignored**: `cargo build -p myosu-games-poker` fails with "package not found".

---

## Risks the Implementation Lane Must Preserve

1. **Robopoker git rev coupling**: The `rbp-nlhe` git rev in `myosu-games-poker` must match the rev used by `myosu-games`. If the implementation lane bumps the rev in `myosu-games-poker` without also bumping it in `myosu-games`, binary incompatibility will result. The implementation lane must treat the git rev as a global constant, not a per-crate parameter.

2. **`Flagship` type stability**: The `Flagship` type alias in robopoker (`NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`) is the intended solver type. If robopoker changes its public exports at the pinned rev, the wrapper breaks. No mitigation exists short of vendoring or a local patch — INV-006 tracks upstream submission.

3. **Checkpoint format versioning**: The `MYOS` magic + u32 version format must include a version field that is checked on load. A future rev that changes the serialized format must bump the version and produce a clear error on version mismatch. The implementation lane must not skip this.

4. **Exploitability computation time**: Full exploitability computation on the full NLHE game tree is expensive. The `remote_poker_exploitability` function builds a synthetic profile from query responses — this may be O(info_sets) and could exceed validator time budgets. The implementation lane should add a timeout or sampling mode rather than allowing unbounded computation.

---

## Risks the Implementation Lane Should Reduce

1. **Debug iteration speed**: Using `Flagship` (Pluribus sampling) for development is slow. The implementation lane should support a `DebugSolver` variant (`VanillaRegret + UniformWeight + ExternalSampling`) behind a feature flag so tests run faster in CI.

2. **Empty profile exploitability**: `Profile::exploitability()` on an untrained (empty) profile may return NaN. The implementation lane should handle this gracefully and return an error or `Infinity` rather than propagating NaN.

---

## Is the Lane Ready for an Implementation-Family Workflow Next?

**Yes — with conditions.**

The specification is stable and self-contained. The implementation lane can begin with Slice 1 (crate skeleton) immediately. The only prerequisite from the upstream `games:traits` lane is satisfied (git dependency migration is done).

The conditions for proceeding:

1. The `serde` feature verification in Slice 1 must succeed before Slice 3 is started
2. The robopoker git rev must be treated as a shared constant across all crates that depend on it
3. All 16 test commands above must exit 0 before the lane is marked complete

If any of those conditions cannot be met, the lane must be reopened before proceeding.

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | `games:poker-engine` depends on `games:traits` for `StrategyQuery`, `StrategyResponse`, `GameType`, `GameConfig`. Relationship is stable. |
| `games:variant-family` | Depends on `games:poker-engine` for the NLHE HU baseline. 6-max, PLO, Short Deck, Tournament each add a `CfrGame` impl — they reuse the wrapper, query, wire, and exploitability surfaces. |
| `services:miner` | Consumer of `games:poker-engine`. The miner binary calls `PokerSolver::train()`, `handle_query()`, and `PokerSolver::save()`. The interface is defined by this lane. |
| `services:validator-oracle` | Consumer of `games:poker-engine`. Calls `remote_poker_exploitability()` and `handle_query()` (as a client). The query wire format is defined here. |

The `services:miner` and `services:validator-oracle` lanes are blocked until `games:poker-engine` produces a working crate — they cannot be meaningfully implemented without a concrete game engine to connect to.
