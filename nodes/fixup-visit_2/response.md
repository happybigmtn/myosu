## Fixup Summary

**Root Cause Confirmed**: The 10 failing tests all call `train()`, which requires a populated `NlheEncoder`. The encoder's isomorphism→abstraction mapping is the output of k-means clustering stored in PostgreSQL. `NlheEncoder::default()` creates an empty map, causing a panic at `encoder.rs:33`.

**What the fixup applied**:

1. **Enabled `database` feature** on `rbp-nlhe` in `crates/myosu-games-poker/Cargo.toml`
   - This unlocks the `rbp_database::Hydrate` trait for loading encoder mappings from PostgreSQL
   - The infrastructure path is now available, even though PostgreSQL itself is not present

2. **Updated `outputs/games/poker-engine/implementation.md`**:
   - Updated `database` feature status from "needed" to "enabled"
   - Added fixup entry to Files Modified section

3. **Updated `outputs/games/poker-engine/verification.md`**:
   - Added "Fixup Applied" section documenting the `database` feature enablement
   - Updated "Unblock Path" to reflect current status
   - Clarified that PostgreSQL with k-means data is required for full test suite pass

**Current state**: 5 tests pass (those that don't call `train()`), 10 fail (all require `train()` → populated encoder → PostgreSQL).

`★ Insight ─────────────────────────────────────`
The poker engine uses a **separation between computation and data** pattern common in production ML systems: the expensive k-means clustering (run once, offline) produces lookup tables stored in PostgreSQL, and the solver loads them at runtime via the `Hydrate` trait. This means the `train()` function has a **mandatory infrastructure dependency** that cannot be satisfied by code alone — it requires the database artifact from a prior clustering run.
`─────────────────────────────────────────────────`

The implementation code is correct per the spec. The test failures are **deterministic and expected** without PostgreSQL infrastructure. The `database` feature is now enabled, which is the prerequisite for the PostgreSQL hydration path.