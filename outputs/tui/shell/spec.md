# tui:shell — Lane Specification

## Purpose and User-Visible Outcome

The `tui:shell` lane owns the terminal user interface for Myosu. It provides:

1. **Visual shell** — Five-panel layout (header, transcript, state, declaration, input) rendered via ratatui
2. **Screen navigation** — State machine managing Onboarding → Lobby → Game → Stats/Coaching/History/Wallet/Spectate transitions
3. **Input handling** — Command line with history, tab completion, and slash-command routing
4. **Event loop** — Async bridge between crossterm terminal events and tokio channels
5. **Schema contract** — JSON game-state schema for agent protocol (pipe mode)

**User-visible behavior**: A player launching `myosu-play` sees a lobby, selects a game, plays via typed commands, and can invoke `/analyze`, `/stats`, `/history`, `/wallet`, `/spectate` as overlays or transitions.

---

## Lane Boundary

```
                    ┌─────────────────────────────────────────┐
                    │            tui:shell (THIS LANE)         │
                    │                                         │
  upstream          │  ┌───────┐  ┌──────┐  ┌───────┐        │
  game-engine  ───► │  │ shell │  │input │  │screens│        │
  (untrusted)       │  └───┬───┘  └──┬───┘  └───┬───┘        │
                    │      │         │          │              │
                    │  ┌───┴─────────┴──────────┴────┐        │
                    │  │      renderer (trait)       │        │
                    │  │  ┌─────────────────────┐  │        │
                    │  │  │  GameRenderer impl    │  │        │
                    │  │  │  (per-game, untrusted)│  │        │
                    │  │  └─────────────────────┘  │        │
                    │  └────────────────────────────┘        │
                    │                                         │
                    │  ┌────────┐  ┌─────────┐               │
                    │  │ schema │  │ events  │               │
                    │  └────────┘  └─────────┘               │
                    │                                         │
                    │  ┌────────┐  ┌────────┐                │
                    │  │ theme  │  │  pipe  │                │
                    │  └────────┘  └────────┘                │
                    └─────────────────────────────────────────┘
```

**Trusted inputs**: All modules in `crates/myosu-tui/src/` are treated as trusted leaf surfaces. The lane boundary ends at the `GameRenderer` trait object — concrete game implementations (NLHE, Riichi, etc.) live downstream.

**Untrusted output**: `Box<dyn GameRenderer>` — the shell accepts any object-safe `GameRenderer`. Downstream game crates must prove their renderer via integration tests.

---

## Current Trusted Inputs

| Module | Responsibility | Test Coverage |
|--------|---------------|---------------|
| `shell.rs` | Five-panel layout, event coordination, screen transitions | Good (16 `shell_state` tests + broader unit coverage) — draw, log, layout, slash clear, lobby submit routing, non-Game screens, help overlay |
| `screens.rs` | Screen enum, navigation state machine | Good (18 tests) — all transitions, command routing, history |
| `input.rs` | InputLine buffer, history, completion, cursor | Good (20+ tests) — all key handlers, history, tab completion |
| `renderer.rs` | `GameRenderer` trait, `Renderable` trait | Good (mock-based tests) — trait object safety, roundtrip |
| `schema.rs` | JSON game-state schema (GameState, LegalAction, etc.) | Better (14 tests) — NLHE HU, NLHE 6-max, Riichi, LiarsDice, custom action roundtrip |
| `events.rs` | EventLoop (crossterm + tokio bridge) | Good (6 tests) — headless key, resize, injected update, sender clone, variants |
| `theme.rs` | 8-token color palette | Good (7 tests) — color distinctness, style methods |
| `pipe.rs` | Pipe mode driver for agent protocol | Good (5 tests) — ANSI detection, state parsing |

---

## Broken or Missing Surfaces

### 1. Schema — Game Coverage Claims Exceed Tests

**Claim** (schema.rs docstring): "Universal machine-readable game state for **all 20 games**"
**Proof**: Full roundtrip coverage now exists for NLHE heads-up, NLHE 6-max, Riichi,
and Liar's Dice, plus direct custom-action roundtrips. The rest of the listed
game types still only have shallow placeholder proof through `all_game_types_have_schema`.

**Impact**: Agents reading schema for unsupported games receive structurally-valid but semantically-untested JSON.

**Required**: Per-game roundtrip tests or explicit `#[unimplemented]` markers for each `game_type`.

---

## Proof/Check Shape for the Lane

The lane is **proven** when:

```
✓ All unit-test modules (screens, input, renderer, schema, theme, pipe) pass `cargo test`
✓ Event loop has headless proof for key/ resize/ update paths
✓ Shell has integration test covering Lobby→Game transition via typed input
✓ Shell has render test for each of the 8 screen variants
△ Schema has deep proof for the active examples and explicit handling for the long tail
```

---

## Next Implementation Slices

### Slice 1: Schema Per-Game Coverage
**File**: `crates/myosu-tui/src/schema.rs`
**Action**: For each game_type in `all_game_types_have_schema`, either add full roundtrip test or add `#[unimplemented]` comment with tracking issue. The next additions should target one more real non-poker game and one more poker variant beyond the current heads-up and 6-max proof.
**Proof gate**: `cargo test schema::all_game_types`

### Slice 2: Pipe Mode ANSI Enforcement
**File**: `crates/myosu-tui/src/pipe.rs`
**Action**: Add property test: for all inputs, `pipe_output()` result passed through `is_plain_text()` returns true.
**Proof gate**: `cargo test pipe::is_plain_text`

### Completed this pass: Event Loop Headless Test
**File**: `crates/myosu-tui/src/events.rs`
**Action**: Added a stream-driven test harness that injects synthetic `CrosstermEvent` values without requiring a TTY. The former ignored tests now run headlessly.
**Proof gate**: `cargo test -p myosu-tui`

### Completed this pass: Shell Integration Test
**File**: `crates/myosu-tui/src/shell.rs`
**Action**: Added a shell-level test that simulates typing `1` in Lobby and pressing Enter, proving `handle_key` → `handle_submit` → `screens.apply_command` moves to Game.
**Proof gate**: `cargo test -p myosu-tui`

### Completed this pass: Shell Render Breadth
**File**: `crates/myosu-tui/src/shell.rs`
**Action**: Added a stable `shell_state` target with 16 passing tests, including Game rendering, all non-Game screens, and help overlay bounds.
**Proof gate**: `cargo test -p myosu-tui shell_state`

### Completed this pass: Schema Depth Increase
**File**: `crates/myosu-tui/src/schema.rs`
**Action**: Added `nlhe_6max` roundtrip proof plus `LegalAction::Custom` and `AgentAction::Custom` roundtrips.
**Proof gate**: `cargo test -p myosu-tui schema`
