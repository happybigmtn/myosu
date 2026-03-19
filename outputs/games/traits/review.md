# `games:traits` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (with implementation lane unblocked)**

`myosu-games` is a **trusted kept leaf crate**. The current code compiles cleanly, all 10 unit tests pass, all 4 doctests pass, and the crate surface is small and well-bounded.

The lane is **unblocked** for implementation work after this bootstrap run completes.

## Implementation Lane Unblocked?

**Yes — unconditionally.**

The bootstrap run has produced:
- `outputs/games/traits/spec.md` (this file's companion)
- `outputs/games/traits/review.md` (this file)

The implementation lane can begin with Slice 1 (replace absolute path dependencies with git rev) immediately after bootstrap completion, using the proof command:

```bash
cargo check -p myosu-games && cargo test -p myosu-games
```

## Concrete Risks the Implementation Lane Must Preserve or Reduce

### Risk 1: Robopoker Absolute Path Coupling
**Exact location:** `crates/myosu-games/Cargo.toml` lines 16–17

```toml
rbp-core = { path = "/home/r/coding/robopoker/crates/util" }
rbp-mccfr = { path = "/home/r/coding/robopoker/crates/mccfr" }
```

**What must be preserved:**
- The `rbp-core` and `rbp-mccfr` dependency declarations (they are the correct upstream crates)
- The `pub use rbp_core::{Probability, Utility}` and `pub use rbp_mccfr::{CfrEdge, CfrGame, ...}` re-exports in `src/traits.rs` lines 8–9

**What must be reduced:**
- Replace both `path = "/home/r/coding/robopoker/..."` entries with `git = "https://github.com/happybigmtn/robopoker", rev = "..."`
- The TODO comment on line 15 explicitly acknowledges this is the correct next step: `# TODO: Switch to git dependency once happybigmtn/robopoker fork is published`

**Verification:**
```bash
# Must pass after Slice 1
cargo fetch
cargo test -p myosu-games
```

### Risk 2: Robopoker API Surface Leakage
**Exact location:** `crates/myosu-games/src/traits.rs` lines 8–9

```rust
pub use rbp_core::{Probability, Utility};
pub use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile};
```

The crate re-exports raw robopoker types. If `rbp-core` or `rbp-mccfr` makes a breaking API change, `myosu-games` will break at compile time with no intermediate adaptation layer.

**What must be preserved:**
- The thin re-export surface (adding wrapper traits would be a future slice)
- The `reexports_compile` test at `src/traits.rs` line 229 confirming re-exports compile

**What must be reduced:**
- No action required in early slices; risk is accepted until robopoker is published
- Slice 0 (audit) should produce a one-page note confirming the re-export surface is intentional

### Risk 3: `Probability` Floating-Point Assumption
**Exact location:** `crates/myosu-games/src/traits.rs` line 205

```rust
(sum - 1.0).abs() < 0.001
```

`Probability` is a type alias for a float (from `rbp-core`). The `is_valid()` check uses a hardcoded epsilon. This is acceptable for the bootstrap lane but introduces floating-point equality risk at scale.

**What must be preserved:**
- The `is_valid()` method and its epsilon behavior (changing it would break callers)
- The `strategy_response_validates` test at line 340

**What must be reduced:**
- No action required in early slices; this is a future precision concern
- Slice 4 (integration tests) should exercise this boundary explicitly

### Risk 4: `GameType` Byte Encoding Is Off-Chain Convention
**Exact location:** `crates/myosu-games/src/traits.rs` lines 83–114

`GameType::from_bytes` / `to_bytes` use byte string conventions (`b"nlhe_hu"`, etc.) that must match what the on-chain pallet stores. If the pallet changes its encoding, `myosu-games` will silently produce mismatched bytes.

**What must be preserved:**
- The roundtrip test `game_type_to_bytes_roundtrip` at line 278
- The known game type mappings (the enum variants are part of the lane contract)

**What must be reduced:**
- No action in early slices; this requires coordination with `chain:pallet` lane
- `chain:pallet` lane's `review.md` must note this dependency

### Risk 5: Edition 2024 Forces Non-Stable Rust
**Exact location:** `Cargo.toml` workspace root line 15

The workspace pins `edition = "2024"`. Rust 2024 is not yet stable. This forces contributors onto a nightly or beta toolchain.

**What must be preserved:**
- All current tests and compilation behavior
- The edition value until a conscious migration decision is made

**What must be reduced:**
- Either downgrade to `edition = "2021"` or add `rust-toolchain.toml` pinning nightly
- This is Slice 3; Slice 1 and 2 can proceed with 2024 edition unchanged

### Risk 6: No Version Pin on Robopoker After Slice 1 Migration
**Exact location:** `crates/myosu-games/Cargo.toml` lines 16–17 (after Slice 1)

Once absolute paths are replaced with git dependencies without a pinned `rev`, the crate will track robopoker's default branch tip, creating silent update risk.

**What must be reduced:**
- Pin to a specific `rev` in Slice 1 using the current HEAD commit hash
- Document the pinned rev in the commit message

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Bootstrap trust check | `cargo test -p myosu-games` | Exit 0, 10 unit + 4 doctest pass |
| Implementation verification | `cargo check -p myosu-games && cargo test -p myosu-games` | Exit 0 |
| Post-Slice-1 fetch | `cargo fetch` | Exit 0 without local robopoker |
| Post-Slice-1 test | `cargo test -p myosu-games` | Exit 0 |

## File Reference Index

| File | Role |
|------|------|
| `crates/myosu-games/Cargo.toml` | Crate manifest; absolute path dependencies at lines 16–17 |
| `crates/myosu-games/src/lib.rs` | Thin lib entry point; re-exports traits module and types |
| `crates/myosu-games/src/traits.rs` | All game config types, re-exports, and tests (lines 1–367) |
| `crates/myosu-games/README.md` | User-facing doc; doctests verified |
| `fabro/checks/games-traits.sh` | Bootstrap proof script |
| `fabro/checks/games-traits-implement.sh` | Implementation proof script |
| `fabro/run-configs/bootstrap/game-traits.toml` | Bootstrap run config |
| `fabro/run-configs/implement/game-traits.toml` | Implementation run config |
| `fabro/workflows/bootstrap/game-traits.fabro` | Bootstrap workflow graph |
| `fabro/workflows/implement/game-traits.fabro` | Implementation workflow graph |
| `fabro/programs/myosu-bootstrap.yaml` | Raspberry program manifest (unit: `games`, lane: `traits`) |
| `outputs/games/traits/spec.md` | This lane's spec artifact |
| `outputs/games/traits/review.md` | This file |
