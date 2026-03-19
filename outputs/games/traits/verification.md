# `games:traits` Verification — Slice 1

## Proof Commands That Passed

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo check -p myosu-games` | 0 | Passed |
| `cargo test -p myosu-games` | 0 | 10 unit + 4 doctest passed |
| `cargo fetch` | 0 | Fetched 3 robopoker packages from git without local path |

## Risks Reduced

- **Risk 1 (Robopoker Absolute Path Coupling):** Reduced. Both `rbp-core` and `rbp-mccfr` now resolved via git URL + pinned `rev` instead of absolute filesystem paths. The crate is now portable — any developer with network access can build it without a local robopoker clone.
- **Risk 6 (No Version Pin on Robopoker After Slice 1 Migration):** Reduced. `rev = "04716310143094ab41ec7172e6cea5a2a66744ef"` pins to the current HEAD commit. `cargo update` will not auto-advance to a newer commit unless explicitly invoked.

## Risks That Remain

- **Risk 2 (Robopoker API Surface Leakage):** Unchanged. The re-export surface `pub use rbp_core::{Probability, Utility}` and `pub use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile}` remains. Future slices would need an adaptation layer to decouple from robopoker internals.
- **Risk 3 (Probability Floating-Point Assumption):** Unchanged. `is_valid()` continues to use epsilon 0.001.
- **Risk 4 (GameType Byte Encoding Is Off-Chain Convention):** Unchanged. The byte encoding conventions in `GameType::from_bytes`/`to_bytes` remain coupled to the chain pallet's storage format.
- **Risk 5 (Edition 2024 Forces Non-Stable Rust):** Unchanged. The workspace still pins `edition = "2024"`. Rust 2024 is not yet stable as of early 2026.

## Next Slice

**Slice 3 — Resolve Edition 2024**

Either downgrade the workspace `edition` to `"2021"` or add a `rust-toolchain.toml` pinning a nightly/beta toolchain. This is a tooling hygiene item with no impact on crate logic. It is a prerequisite for contributor friendliness (Rust 2024 is not stable as of early 2026).
