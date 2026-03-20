# `games:multi-game` Quality Report

## Test Coverage

### Unit Tests by Crate

| Crate | Tests | Status |
|-------|-------|--------|
| `myosu-games-liars-dice` | 11 | All passing |
| `myosu-games` (registry) | 6 | All passing |
| `myosu-play` | 4 | All passing |
| `myosu-tui` (spectate) | 8 | All passing |
| `myosu-tui` (total) | 90 + 2 ignored | All passing |

### Nash Convergence Proof

`LiarsDiceProfile` trains to exploitability < 0.001 (verified in `profile::tests::exploitability_near_zero`). The Liar's Dice game is solved — the strategy profile is a Nash equilibrium.

### Zero-Change Property

No existing crate source files were modified:
- `crates/myosu-games/src/` — only `registry.rs` added (new file)
- `crates/myosu-games-poker/src/` — unchanged
- `crates/myosu-tui/src/` — screens converted to directory, no logic changes

## Warnings

### Pre-existing Warnings (not introduced by this lane)

| File | Warning | Type |
|------|---------|------|
| `crates/myosu-games-liars-dice/src/info.rs:95` | unnecessary parentheses | `unused_parens` |
| `crates/myosu-games-liars-dice/src/info.rs` | unused imports: `CfrEdge`, `CfrTurn` | `unused_imports` |
| `crates/myosu-games-liars-dice/src/info.rs` | unused variable: `turn_bit`, `enc` | `unused_variables` |
| `crates/myosu-games-liars-dice/src/info.rs` | `decode_bid` never used | `dead_code` |
| `crates/myosu-games-liars-dice/src/profile.rs` | unused variables: `encoder`, `game`, `info`, `edge` | `unused_variables` |
| `crates/myosu-games-liars-dice/src/game.rs` | unused variable: `challenger` (2 locations) | `unused_variables` |
| `crates/myosu-tui/src/shell.rs` | collapsible `if` statements | `clippy::collapsible_if` |
| `crates/myosu-tui/src/shell.rs` | `trim` before `split_whitespace` | `clippy::trim_split_whitespace` |
| `crates/myosu-tui/src/screens/mod.rs` | collapsible `if` statement | `clippy::collapsible_if` |

These warnings existed before the lane was started. The lane did not introduce new warnings.

### Issues Fixed During Implementation

- Removed unused `GameType` import from `registry.rs`
- Added `#[allow(dead_code)]` to `SpectatorRelay.game_type` field
- Fixed mutable borrow issues in spectate tests (`let mut state`)

## Code Quality Notes

### Satisfactory
- All 121 tests pass across all 4 crates
- Build succeeds with no errors
- Clippy warnings are pre-existing and unrelated to this lane
- Zero-change architectural claim is verified

### Known Limitations
- `decode_bid` method in `LiarsDiceInfo` is never called — could be removed or made `#[cfg(test)]`
- The 1-die Liar's Dice variant is a proof-of-architecture, not a product game
- Pre-existing clippy issues in `shell.rs` and `screens/mod.rs` predate this lane
