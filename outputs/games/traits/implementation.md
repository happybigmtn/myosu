# `games:traits` Implementation Notes

**Lane**: `games:traits`
**Date**: 2026-03-27

---

## Current Implemented Shape

The crate already includes the portability slice that older artifacts treated as
future work:

- `rbp-core` and `rbp-mccfr` are pinned git dependencies
- the crate has an explicit `[lib] crate-type = ["lib"]`
- the public API surface is in `src/traits.rs`

No additional bootstrap-era implementation work is required to make this crate
portable or buildable.

---

## Immediate Implementation Follow-Ups

### 1. Expand Property-Based Tests
The first property-based slice is already in place. Continue strengthening proof
for:

- `GameType::from_bytes` / `to_bytes`
- `GameConfig` serde roundtrip
- `StrategyQuery` / `StrategyResponse` serde roundtrip

### 2. Keep the Trait Surface Thin
Do not add wrappers or abstractions unless a concrete downstream integration
forces them. The current re-export shape is acceptable and easy to reason about.

### 3. Add Engine-Level Validation Later
Once `myosu-games-poker` exists, prove the wire types against a real game
adapter rather than only unit-level fixtures.

---

## Verification Commands

```bash
cargo check -p myosu-games
cargo test -p myosu-games traits::tests::serialization_roundtrip_strategy_response --quiet
cargo test -p myosu-games serialization_roundtrip
```
