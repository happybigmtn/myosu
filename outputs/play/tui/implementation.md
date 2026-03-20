# `play:tui` Lane — Slice 1 Implementation

## Slice: `tui-implement/slice-1-binary-skeleton`

**Date**: 2026-03-20
**Status**: Complete

---

## What Was Implemented

### 1. `crates/myosu-play/` Crate Scaffold

Created the `myosu-play` binary crate with:
- `Cargo.toml` — workspace member, depends on `myosu-tui`, `ratatui`, `crossterm`, `tokio`, `clap`
- `src/main.rs` — CLI entry point with `--train` / `--chain` / `--pipe` mode dispatch

### 2. Stub `NlheRenderer` Implementing `GameRenderer` Trait

A minimal `GameRenderer` implementation with hardcoded NLHE state that proves the render loop works:
- `render_state()` — renders 2 lines of poker state (pot, hero cards, board, bot info)
- `desired_height()` — returns 4 when hand active, 0 otherwise
- `declaration()` — returns `THE SYSTEM AWAITS YOUR DECISION` or `NO ACTIVE HAND`
- `completions()` — provides tab-completion for `fold`, `call`, `raise`, `check`
- `parse_input()` — parses shorthand input (`f`, `c`, `r`, `k`) to full actions
- `clarify()` — prompts for raise amount when input is ambiguous
- `pipe_output()` — returns plain-text state for `--pipe` mode
- `game_label()` — returns `"NLHE-HU"`
- `context_label()` — returns `"HAND 1"`

### 3. Shell Wiring

- `Shell::with_screen(Screen::Game)` creates shell in game mode
- `terminal.draw()` renders via `shell.draw()` into ratatui buffer
- Successfully proves the 5-panel layout renders without panic

### 4. Workspace Integration

- Added `crates/myosu-play` to workspace `members` in root `Cargo.toml`

---

## Files Created/Modified

| File | Change |
|------|--------|
| `crates/myosu-play/Cargo.toml` | Created — new binary crate |
| `crates/myosu-play/src/main.rs` | Created — CLI + stub renderer |
| `Cargo.toml` (workspace) | Modified — added `crates/myosu-play` to members |

---

## Architecture Notes

### Why `GameRenderer` trait is object-safe

The `GameRenderer` trait in `myosu-tui` requires no `Sized` bounds and has only object-safe methods (`&self`). This allows `Box<dyn GameRenderer>` to be created and passed to `Shell::draw()`.

### Slice 1 limitation: no event loop

The full async event loop integration is deferred to later slices. Slice 1 proves:
1. The binary builds
2. The render loop executes without panic
3. The `Shell` + `GameRenderer` integration compiles

### `myosu-games-poker` crate (Slice 2)

The `NlheRenderer` in Slice 1 is a stub. The real NLHE renderer with proper card/suit rendering and game-state logic will live in `crates/myosu-games-poker/` (Slice 2).

---

## Deps Graph

```
myosu-play (binary)
├── myosu-tui (shell, GameRenderer trait)
│   ├── ratatui
│   ├── crossterm
│   └── tokio
├── clap (CLI)
└── tracing-subscriber (logging)
```

---

## Next Slice

**Slice 2**: `NlheRenderer` with hardcoded states in `crates/myosu-games-poker/`
- Move the stub `NlheRenderer` to a dedicated `myosu-games-poker` crate
- Add hardcoded preflop/-flop/turn/river/showdown states
- Prove `cargo test -p myosu-games-poker` passes
