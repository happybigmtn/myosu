# `games:traits` Lane Spec

**Lane**: `games:traits`
**Date**: 2026-03-27
**Status**: Kept and current

---

## Purpose and Boundary

`games:traits` is the trait and wire-format surface for game engines in Myosu.
It owns:

- public re-exports of robopoker's game-agnostic CFR traits
- `Probability` and `Utility` re-exports from `rbp-core`
- Myosu game configuration types: `GameConfig`, `GameType`, `GameParams`
- miner-validator wire types: `StrategyQuery<I>` and `StrategyResponse<E>`
- tests and doctests for the above

It does not own:

- concrete poker or other game implementations
- on-chain pallet logic
- miner, validator, or gameplay binaries

---

## Current Source Inventory

| Surface | Path | Current reality |
|---------|------|-----------------|
| Crate manifest | `crates/myosu-games/Cargo.toml` | Honest workspace member with explicit `[lib]` section |
| Trait surface | `crates/myosu-games/src/traits.rs` | Small, focused module with typed config and strategy wire types |
| Crate entry | `crates/myosu-games/src/lib.rs` | Thin re-export layer |
| Upstream deps | `rbp-core`, `rbp-mccfr` | Pinned git dependencies to `happybigmtn/robopoker` rev `04716310143094ab41ec7172e6cea5a2a66744ef` |
| Workspace policy | root `Cargo.toml` | Uses workspace `edition = "2024"` |

---

## Verified Proof Shape

`cargo test -p myosu-games -- --list` on 2026-03-27 reports:

- 10 unit tests
- 4 doctests

Additionally, targeted property-based verification now exists:

```bash
cargo test -p myosu-games serialization_roundtrip
```

That command currently passes 3 property tests covering:

- `GameType` byte roundtrip
- `GameConfig` serde roundtrip
- `StrategyResponse<String>` serde roundtrip

The named unit tests cover:

- `GameConfig` serialization
- `GameType` byte parsing and roundtrip
- `GameType::num_players`
- trait re-export availability
- `StrategyQuery` / `StrategyResponse` roundtrip and validation

This is enough to keep the lane trusted as a leaf crate.

---

## What Changed Since The Earlier Bootstrap Snapshot

The earlier artifact focused on absolute local path dependencies into a sibling
robopoker checkout. That is no longer true.

Current reality:

- robopoker dependencies are already pinned via git revision
- the crate has an explicit `[lib] crate-type = ["lib"]`
- the crate is portable at the dependency layer without a local robopoker clone

That means the lane is no longer blocked on dependency portability work.

---

## Current Risks

### 1. Thin Re-Export Coupling

The crate still re-exports raw robopoker traits directly:

```rust
pub use rbp_core::{Probability, Utility};
pub use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile};
```

This is intentional and workable, but it means upstream robopoker API changes
will surface directly at the Myosu boundary.

### 2. Limited Game Variant Coverage

The current `GameType` enum covers:

- `NlheHeadsUp`
- `NlheSixMax`
- `LiarsDice`
- `Custom(String)`

This is a clean Stage 0 surface, but it is still narrower than the larger game
portfolio described elsewhere in the repo narrative.

### 3. Wire Roundtrips Are Type-Level, Not Yet Engine-Level

`StrategyQuery` and `StrategyResponse` roundtrip tests exist, but they are still
unit-level tests around serialization and validation. They do not yet prove
integration against a real game engine implementation.

---

## Next Approved Slices

### Slice 1: Expand Property-Based Serialization Coverage
The first property-based slice is now in place. The next upgrade is to broaden
that coverage further, especially around `StrategyQuery` and any future game
registry expansion.

Proof target:

```bash
cargo test -p myosu-games -- serialization_roundtrip
```

### Slice 2: Real Engine Integration Probe
Exercise `StrategyQuery` / `StrategyResponse` against a concrete game adapter
once `myosu-games-poker` exists.

Proof target:

```bash
cargo test -p myosu-games-poker -- strategy_query
```

### Slice 3: Broaden Game Registry Surface Carefully
Add new `GameType` variants only when a downstream implementation is ready or
there is an explicit placeholder policy.

---

## Validation Commands

```bash
cargo check -p myosu-games
cargo test -p myosu-games traits::tests::serialization_roundtrip_strategy_response --quiet
cargo test -p myosu-games -- --list
```
