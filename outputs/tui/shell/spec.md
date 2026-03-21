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
| `shell.rs` | Five-panel layout, event coordination, screen transitions | Basic (10 tests) — draw, log, stop, layout calculation |
| `screens.rs` | Screen enum, navigation state machine | Good (18 tests) — all transitions, command routing, history |
| `input.rs` | InputLine buffer, history, completion, cursor | Good (20+ tests) — all key handlers, history, tab completion |
| `renderer.rs` | `GameRenderer` trait, `Renderable` trait | Good (mock-based tests) — trait object safety, roundtrip |
| `schema.rs` | JSON game-state schema (GameState, LegalAction, etc.) | Good (10+ tests) — NLHE, Riichi, LiarsDice roundtrip |
| `events.rs` | EventLoop (crossterm + tokio bridge) | Partial (2 `#[ignore]` TTY tests, 3 unit tests) |
| `theme.rs` | 8-token color palette | Good (7 tests) — color distinctness, style methods |
| `pipe.rs` | Pipe mode driver for agent protocol | Good (5 tests) — ANSI detection, state parsing |

---

## Broken or Missing Surfaces

### 1. Schema — Game Coverage Claims Exceed Tests

**Claim** (schema.rs docstring): "Universal machine-readable game state for **all 20 games**"
**Proof**: `all_game_types_have_schema` test lists 10 game types, but only NLHE, Riichi, and LiarsDice have full roundtrip tests. The remaining 17 games (`nlhe_6max`, `short_deck`, `plo_hu`, `backgammon`, `bridge`, `hanabi`, `leduc`, etc.) have zero test coverage.

**Impact**: Agents reading schema for unsupported games receive structurally-valid but semantically-untested JSON.

**Required**: Per-game roundtrip tests or explicit `#[unimplemented]` markers for each `game_type`.

### 2. Events — TTY-Dependent Tests Ignored

**Claim**: Event loop correctly bridges crossterm and tokio
**Proof**: Two tests (`key_event_handled`, `async_response_received`) are `#[ignore]` because they require a real TTY. No alternative proof exists (mock, mocktty, or integration test).

**Impact**: The async event loop has never been proven to work under automation/CI.

**Required**: Either headless test harness or `#[cfg(test)]` mock that simulates crossterm's EventStream.

### 3. Shell — No Integration Test for Screen Transitions

**Claim**: Screen manager correctly routes commands from shell
**Proof**: `screens.rs` has comprehensive unit tests. `shell.rs` has one test (`handle_slash_clear`) that exercises a slash command. No test verifies the shell's `handle_key` → `handle_submit` → `screens.apply_command` chain.

**Impact**: The integration between input layer and screen navigation is unproven.

**Required**: Integration test that types a game command in Lobby and verifies screen transition to Game.

### 4. Shell — `draw` Method Not Tested for All Screens

**Claim**: Shell renders correctly for all 8 screens
**Proof**: `shell_draw_basic` only tests Game screen. No test verifies rendering for Onboarding, Lobby, Stats, Coaching, History, Wallet, or Spectate.

**Impact**: Non-Game screen rendering is unexercised.

**Required**: Visual/structural tests for each screen variant.

---

## Proof/Check Shape for the Lane

The lane is **proven** when:

```
✓ All unit-test modules (screens, input, renderer, schema, theme, pipe) pass `cargo test`
✓ Event loop has either:
    - Headless integration test proving event delivery, OR
    - TTY-mock test covering key/ resize/ update paths
✓ Shell has integration test covering Lobby→Game transition via typed input
✓ Shell has render test for each of the 8 screen variants
✓ Schema has per-game roundtrip test or explicit unimplemented marker for each game_type
```

---

## Next Implementation Slices

Proof gate note: use exact cargo invocations here, not human shorthand. On this
toolchain, `cargo test events:: --no-ignore` and `cargo test shell:: --integration`
fail fast, while filters such as `cargo test schema::all_game_types`,
`cargo test shell::shell_draw_`, and `cargo test pipe::is_plain_text` select
zero tests.

### Slice 1: Event Loop Headless Test
**File**: `crates/myosu-tui/src/events.rs`
**Action**: Add `MockEventStream` test helper that produces synthetic `CrosstermEvent` values without requiring a TTY. Replace `#[ignore]` tests with mocked versions.
**Proof gate**: `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui events:: -- --include-ignored`

### Slice 2: Shell Integration Test
**File**: `crates/myosu-tui/src/shell.rs`
**Action**: Add test that creates Shell with Lobby screen, simulates typing "1" followed by Enter, verifies `current_screen()` returns Game.
**Proof gate**: `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui shell::`

### Slice 3: Schema Per-Game Coverage
**File**: `crates/myosu-tui/src/schema.rs`
**Action**: For each game_type in `all_game_types_have_schema`, either add full roundtrip test or add `#[unimplemented]` comment with tracking issue. At minimum, add NLHE heads-up and 6-max variants.
**Proof gate**: `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui all_game_types_have_schema`

### Slice 4: Screen Render Tests
**File**: `crates/myosu-tui/src/shell.rs`
**Action**: Add `shell_draw_lobby`, `shell_draw_onboarding`, `shell_draw_stats`, etc. verifying buffer content for each screen.
**Proof gate**: `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui shell_draw_`

### Slice 5: Pipe Mode ANSI Enforcement
**File**: `crates/myosu-tui/src/pipe.rs`
**Action**: Add property test: for all inputs, `pipe_output()` result passed through `is_plain_text()` returns true.
**Proof gate**: `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui is_plain_text`
