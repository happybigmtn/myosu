# tui:shell — Lane Review

## Judgment Summary

| Surface | Status | Rationale |
|---------|--------|-----------|
| `screens.rs` | **KEEP** | Well-tested, correct screen state machine |
| `input.rs` | **KEEP** | Comprehensive test coverage, correct line editing |
| `renderer.rs` | **KEEP** | Object-safe trait with adequate mock coverage |
| `schema.rs` | **REOPEN** | Proof is deeper now, but the "all 20 games" claim still outruns the tested set. |
| `events.rs` | **KEEP** | Headless synthetic-stream tests now cover key, resize, update, and sender clone paths. |
| `shell.rs` | **KEEP** | Shell-state coverage now exercises Game render, all non-Game screens, and help overlay bounds. |
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

**Coverage Gap**: The schema now has deeper proof than before, but the breadth claim still exceeds the tested set.

**Claim vs Proof**:
- Full roundtrip examples now exist for NLHE heads-up, NLHE 6-max, Riichi, and Liar's Dice.
- `LegalAction::Custom` and `AgentAction::Custom` are now exercised directly.
- `all_game_types_have_schema` still creates shallow placeholder JSON per game type, which is not enough to justify the "all 20 games" docstring.

**Risk**: An agent parsing schema for an untested game type may receive structurally-valid but semantically-broken JSON.

**Required Proof**:
1. Full roundtrip test for each `LegalAction` variant used by each game
2. `#[unimplemented("game_type: reason")]` markers for deliberately unsupported games
3. Integration test proving agent can parse a real game state for each game_type

---

### events.rs — KEEP

**Coverage Upgrade**: The old ignored TTY tests have been replaced by a
stream-driven harness that injects synthetic `CrosstermEvent` values.

**Current Proof**:
- key event delivery
- resize event delivery
- async update delivery
- cloned update sender usage
- update-event variant coverage

**Residual Risk**: There is still no full terminal integration proof, but the
headless proof is now strong enough to keep this module trusted.

---

### shell.rs — KEEP

**Coverage Upgrade**: Shell is no longer the weak point in the lane proof.

**Claim vs Proof**:
- `handle_key` → `handle_submit` → `screens.apply_command` chain is now tested
  for the Lobby → Game path
- `cargo test -p myosu-tui shell_state` now exercises 16 shell-state tests
- Game rendering is verified with transcript, declaration, input, and state-panel content in one buffer
- Onboarding, Lobby, Stats, Coaching, History, Wallet, and Spectate are all rendered in a non-Game loop that proves the state panel stays collapsed
- Help overlay rendering is verified for visible content and in-bounds placement

**Residual Risk**:
1. Buffer-content assertions still use a mock renderer, so this is shell trust, not full per-game trust
2. The help overlay proof checks bounds and visible content, not exact box glyph geometry across terminals

**Judgment**: This module is now trustworthy enough to stay in the keep set.

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
| schema | All 20 games | Active examples have deeper proof; long tail still shallow | **MEDIUM** |
| events | Async event loop works | Headless proof exists | **LOW** |
| shell | Integration works | 16 shell-state tests cover render breadth and overlay bounds | **LOW** |
| theme | 8 distinct tokens | 7 tests | None |
| pipe | ANSI-free output | 5 unit tests | **LOW** |

---

## Recommendation

**Keep** `screens`, `input`, `renderer`, `theme` — these are production-quality trusted leaf surfaces.

**Keep** `shell` and `events` — the shell/state proof is now strong enough to trust the lane surface.

**Reopen** `schema` — it is materially improved, but still over-claims breadth relative to tested game semantics.

The lane still is not fully bootstrapped until the remaining schema-depth gap is reduced.
