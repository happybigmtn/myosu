# tui:shell — Lane Review

## Judgment Summary

| Surface | Status | Rationale |
|---------|--------|-----------|
| `screens.rs` | **KEEP** | Well-tested, correct screen state machine |
| `input.rs` | **KEEP** | Comprehensive test coverage, correct line editing |
| `renderer.rs` | **KEEP** | Object-safe trait with adequate mock coverage |
| `schema.rs` | **REOPEN** | Game coverage claim (20 games) exceeds proof (3 games fully tested) |
| `events.rs` | **REOPEN** | TTY dependency blocks CI proof; no headless alternative |
| `shell.rs` | **REOPEN** | Integration between input and screen navigation untested; non-Game screens unrendered |
| `theme.rs` | **KEEP** | 8-token palette tested for distinctness |
| `pipe.rs` | **KEEP** (with caveats) | ANSI enforcement untested via property test |

---

## Detailed Assessments

### screens.rs — KEEP

**Coverage**: 18 tests covering all 8 screens, all transitions, history tracking, overlay behavior.

**Correctness**: State machine correctly enforces preconditions (e.g., `/stats` only valid from Game). Navigation history properly maintained for back-supporting screens.

**Issues**: None identified. Production-quality module.

---

### input.rs — KEEP

**Coverage**: 20+ tests covering every key handler (Enter, Backspace, Delete, Arrow keys, Tab, Ctrl-A/E/U/K/W), history navigation, tab completion cycles, slash-command detection.

**Correctness**: InputLine correctly implements emacs-style editing. History avoids consecutive duplicates. Tab completion cycles through matches.

**Issues**: None identified. Production-quality module.

---

### renderer.rs — KEEP

**Coverage**: Trait object safety verified. Mock-based tests for all 8 `GameRenderer` methods. Roundtrip tests for `parse_input`, `clarify`, `completions`, `pipe_output`.

**Correctness**: `GameRenderer` is correctly object-safe (no async methods, no generics). The trait cleanly separates shell concerns from game-specific rendering.

**Issues**: None identified. The trait boundary is the correct trust interface.

---

### schema.rs — REOPEN

**Coverage Gap**: Only NLHE, Riichi, and LiarsDice have full roundtrip tests. The docstring claims "all 20 games" but 17 game types have zero proof.

**Claim vs Proof**:
- `all_game_types_have_schema` test creates empty JSON state per game type, but this only proves `serde_json::json!()` works — not that the schema correctly serializes/deserializes game-specific fields.
- `LegalAction` variants for 17 untested games (e.g., `Bridge` bidding, `Hanabi` hints) have never been exercised.

**Risk**: An agent parsing schema for an untested game type may receive structurally-valid but semantically-broken JSON.

**Required Proof**:
1. Full roundtrip test for each `LegalAction` variant used by each game
2. `#[unimplemented("game_type: reason")]` markers for deliberately unsupported games
3. Integration test proving agent can parse a real game state for each game_type

---

### events.rs — REOPEN

**Coverage Gap**: Two tests are `#[ignore]` due to TTY requirement. No alternative proof mechanism exists.

**Claim vs Proof**:
- `EventLoop::new(tick_rate)` creates a tokio task reading from `crossterm::event::EventStream`
- The task joins on loop break, properly closes receiver on Drop
- **But**: The event delivery path (crossterm → mpsc → consumer) is completely unexercised in CI

**Risk**: Event loop may silently fail under headless/scripted environments (e.g., `script` command, CI containers).

**Required Proof**:
1. `MockEventStream` that produces synthetic `CrosstermEvent` values without terminal
2. Test proving tick, key, resize, and update events all traverse the channel correctly
3. Integration test with `PipeMode::run_once()` demonstrating end-to-end async behavior

---

### shell.rs — REOPEN

**Coverage Gap**: Shell is the integration point for all other modules but has minimal integration tests.

**Claim vs Proof**:
- `handle_key` → `handle_submit` → `screens.apply_command` chain is **never tested in combination**
- `draw` tested only for Game screen; Onboarding, Lobby, Stats, Coaching, History, Wallet, Spectate rendering is **completely unexercised**
- Help overlay rendering is **unexercised**

**Risk**:
1. Input text routing from `InputLine` to `ScreenManager` could silently break
2. Screen-specific layout (e.g., state panel collapsing to 0 height on non-Game screens) is never verified
3. Help overlay bounds calculation (`help_width = 50.min(...)`) could render incorrectly

**Required Proof**:
1. Integration test: Lobby screen + typed "1" + Enter → assert Game screen
2. Render test for each of the 8 screens verifying buffer content
3. Help overlay test verifying centered box renders within terminal bounds

---

### theme.rs — KEEP

**Coverage**: 7 tests verifying all 8 color tokens are pairwise distinct, style methods produce expected modifiers.

**Correctness**: Palette is semantically mapped (fg, converge, diverge, etc.) and visually distinguishable even without color.

**Issues**: None identified.

---

### pipe.rs — KEEP (with caveats)

**Coverage**: 5 tests for ANSI detection and input parsing. `is_plain_text()` correctly identifies escape sequences.

**Correctness**: Pipe mode correctly uses `writeln!` + `flush()` for agent protocol. ANSI detection is straightforward string scan.

**Caveat**: No property test ensuring `GameRenderer::pipe_output()` always produces plain text. This is a contract that downstream game renderers must honor — worth adding as a trait-level test.

**Required (optional)**: Property test on `GameRenderer` mock proving `pipe_output()` never contains `\x1b[`.

---

## Proof Deficiency Summary

| Module | Proof Claim | Actual Proof | Gap Severity |
|--------|-------------|--------------|-------------|
| screens | All 8 screens, all transitions | 18 tests, exhaustive | None |
| input | All key handlers, history, completion | 20+ tests | None |
| renderer | Trait object-safe, all methods | Mock-based tests | None |
| schema | All 20 games | Only 3 games fully tested | **HIGH** |
| events | Async event loop works | 2 tests ignored, no headless alternative | **HIGH** |
| shell | Integration works | 10 basic tests, no integration chain | **HIGH** |
| theme | 8 distinct tokens | 7 tests | None |
| pipe | ANSI-free output | 5 unit tests | **LOW** |

---

## Recommendation

**Keep** `screens`, `input`, `renderer`, `theme` — these are production-quality trusted leaf surfaces.

**Reopen** `schema`, `events`, `shell` — each has a high-severity proof gap that must be addressed before these modules can be treated as fully proven trusted surfaces.

The lane cannot be declared "bootstrapped" until all three reopened modules achieve CI-compatible proof.
