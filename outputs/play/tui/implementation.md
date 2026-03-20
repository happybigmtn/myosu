# `play:tui` Implementation Artifact

## Slice Coverage

This implementation delivers **Slices 1 and 2** of the `play:tui` lane:

| Slice | Module | Status | Files |
|-------|--------|--------|-------|
| Slice 1 | `myosu-play` binary skeleton | **Done** | `crates/myosu-play/`, `crates/myosu-play/src/main.rs` |
| Slice 2 | `NlheRenderer` + truth stream | **Done** | `crates/myosu-games-poker/` |

Slices 3–7 remain future work (TrainingTable, BlueprintBackend, SolverAdvisor, Recorder, chain integration).

---

## Slice 1: `myosu-play` Binary Skeleton

### What Was Built

- **Crate**: `crates/myosu-play/` — workspace member binary
- **Entry point**: `crates/myosu-play/src/main.rs` with `#[tokio::main]` async runtime
- **CLI flags**: `--train` (local practice), `--chain` (future miner-connected), `--pipe` (future agent protocol)
- **Bot delay**: `--bot-delay-ms` flag (default 300ms)
- **Shell wiring**: `Shell::new()` from `myosu-tui::shell`; `shell.update_completions(&renderer)`; `shell.run(&renderer, tick_rate)` event loop
- **Tick rate**: 60fps (`Duration::from_millis(16)`)

### Design Decisions

- **Error handling**: `--chain` and `--pipe` bail with "not yet implemented" rather than panicking
- **No TTY required for build verification**: `Shell::run()` handles its own event loop internally; no `ratatui::Terminal` wrapper needed in `main.rs`
- **`bot_delay_ms` unused**: The parameter is accepted but not yet wired to the training loop

---

## Slice 2: `NlheRenderer` + Truth Stream

### What Was Built

#### `crates/myosu-games-poker/` Crate

```
src/
├── lib.rs              # pub mod renderer, pub mod truth_stream, pub use NlheRenderer
├── renderer.rs         # NlheRenderer (GameRenderer impl) + NlheState enum + tests
└── truth_stream.rs    # TruthStreamEmitter + LogLine + LogLineType + tests
```

#### `NlheRenderer` (`renderer.rs`)

- **`NlheState` enum** with variants: `Idle`, `Preflop { to_call, has_decision }`, `Flop { to_call, has_decision }`, `BotThinking`, `Showdown { hero_wins }`
- **`has_decision` field**: Added to `Preflop` and `Flop` to distinguish "awaiting hero decision" from "awaiting bot action"
- **`context_label` caching**: `&str` return to satisfy `GameRenderer::context_label` trait bound; updated on `new()`, `preflop()`, `flop()`, `set_state()`
- **Constructors**: `NlheRenderer::preflop(hand_num)` and `NlheRenderer::flop(hand_num)` both set `has_decision: true`
- **`GameRenderer` trait**: All 7 required methods implemented — `game_label`, `context_label`, `symbol`, `completions`, `parse_input`, `pipe_output`, `render`, `desired_height`, `declaration`
- **`desired_height`**: Returns 4 when state is `Preflop`/`Flop`/`BotThinking`/`Showdown`, 0 when `Idle`
- **Completions**: Non-empty when `has_decision` is true; empty when `Idle` or `BotThinking`
- **Declaration**: `THE SYSTEM AWAITS YOUR DECISION` (preflop/flop with decision), `BOT THINKING...` (bot turn), `SHOWDOWN` (showdown), `IDLE` (idle)
- **`parse_input`**: Accepts `f`/`fold`, `c`/`call`, `r`/`raise` + optional amount, `x`/`check`
- **`pipe_output`**: Structured text format with `board:`, `hero:`, `solver:`, `pot:` fields

#### `TruthStreamEmitter` (`truth_stream.rs`)

- **`LogLineType` enum**: `Action`, `Fold`, `StreetTransition`, `Showdown`, `Result`, `Error`, `Fallback`, `Blank`
- **`LogLine` struct**: `text: String`, `line_type: LogLineType`, `pot_at_line: u32`
- **`LogLine::formatted_line(terminal_width)`**: Right-aligns pot with spaces between text and `pot N`
- **`LogLine::style()`**: Returns `Style` with `Color::Rgb` — dim gray for Fold/Fallback, lighter for Result, red for Error, street gray
- **`TruthStreamEmitter`**: Collects `Vec<LogLine>`; tracks `current_pot` and `current_street`; `reset()` clears lines and resets pot/street

### Design Decisions

- **`has_decision` binding**: The combined match arm `NlheState::Preflop { .. } | NlheState::Flop { has_decision, .. }` failed at compile time because `has_decision` isn't bound in the first arm. Split into separate match arms to fix.
- **`context_label` as cached `String`**: The trait requires `&str` but the state enum doesn't carry a hand number string. Solved by adding `context_label: String` field to `NlheRenderer` and updating it on every state transition.
- **Suit constants unused**: `SUIT_SPADES`, `SUIT_HEARTS`, `SUIT_DIAMONDS`, `SUIT_CLUBS` are defined but unused — helpers for future card rendering, not removed to avoid churn.
- **`emit_action` verb format**: `LogLine::action` appends `to {amount}bb` to the verb, so test verbs use "raises" (not "raises to") to produce correct output `raises to 6bb`.

### Workspace Changes

- **Root `Cargo.toml`**: Added `crates/myosu-games-poker` and `crates/myosu-play` to `members` array
- **`myosu-games-poker/Cargo.toml`**: Package `myosu-games-poker`; depends on `myosu-tui`, `ratatui`, `serde`, `thiserror`, `rbp-core` (git)
- **`myosu-play/Cargo.toml`**: Binary `myosu-play`; depends on `myosu-tui`, `myosu-games-poker`, `ratatui`, `crossterm`, `rbp-core` (git), standard workspace deps

### robopoker Dependency Note

`rbp-core` is pulled via git from `https://github.com/happybigmtn/robopoker` at rev `04716310143094ab41ec7172e6cea5a2a66744ef`. This is a temporary git dependency. The review correctly notes this should eventually be migrated to a proper crate registry version once the robopoker repo is cleaned up.

---

## Test Results

```
cargo test -p myosu-games-poker
  22 passed; 0 failed; 0 ignored

cargo build -p myosu-play
  Finished 'dev' profile [unoptimized + debuginfo] target(s) in 2m 39s
```

Warnings (non-blocking):
- `unused variable: label` in `render_declaration` — helper variable for future label styling
- `unused constants SUIT_*` and `unused function render_card/render_hidden/render_slot` — scaffold helpers
- `unused variable: emitter` in two truth_stream tests — emitters constructed for future use

---

## Upstream Contracts Maintained

| Contract | Source | Status |
|----------|-------|--------|
| `GameRenderer` trait | `myosu-tui` (82 tests pass) | Satisfied — all 7 methods implemented |
| `Shell` API | `myosu-tui::shell` | Satisfied — `Shell::new()`, `update_completions`, `run` |
| `crate` workspace membership | root `Cargo.toml` | Satisfied — both crates in `members` |

---

