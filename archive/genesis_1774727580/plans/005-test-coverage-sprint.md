# Test Coverage Sprint

**Plan ID:** 005
**Status:** In Progress
**Priority:** CRITICAL — this unblocks all Phase 1 and Phase 2 work

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `myosu-tui` will have a comprehensive integration test suite covering screen transitions, shell state, event handling, and theme rendering. `myosu-games` will have property-based tests for `GameConfig` and `GameType` round-trip serialization. Every PR touching these crates will be automatically tested.

This plan is Phase 0. It fixes the inverted quality problem where user-facing
code has thinner, less trustworthy coverage than the underlying chain surfaces.

---

## Progress

- [x] Add screen transition tests to `myosu-tui`
- [x] Add shell state tests to `myosu-tui`
- [x] Add event loop tests to `myosu-tui`
- [x] Add property-based round-trip tests to `myosu-games`
- [ ] Integrate into repo verification commands and CI

---

## Surprises & Discoveries

- The old ignored `events.rs` tests were not the real shell gap. The bigger missing proof was
  render breadth: non-Game screens and the help overlay had no targeted verification.
- The most useful shell-state target was not a new test module, but a stable `shell_state`
  prefix inside `shell.rs` itself. That gave a focused proof command without adding indirection.
- `schema.rs` was already broader than the review snapshot implied. Adding a real `nlhe_6max`
  roundtrip and custom-action roundtrips reduced the risk, but did not close the docstring gap
  around "all 20 games."

---

## Decision Log

- Decision: Start with `myosu-tui` tests because it is the primary user-facing surface and still has important coverage gaps.
  Rationale: The TUI is the most interaction-heavy code, and some of its existing tests are shallow or ignored.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Use `crossterm` event simulation for input testing rather than mocking the terminal.
  Rationale: The TUI already uses `crossterm` events; simulating those events in tests is the most faithful approach.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Add screen transition tests to `myosu-tui`
Test that the five-panel shell transitions correctly between modes (normal, pipe, command).

Proof: `cargo test -p myosu-tui -- screen` runs ≥ 5 tests covering mode transitions. All pass.

Key files:
- `crates/myosu-tui/src/screens.rs` — Screen manager
- `crates/myosu-tui/src/shell.rs` — Shell state machine

### M2: Add shell state tests to `myosu-tui`
Test that shell state (header, transcript, declaration, input) updates correctly on events.

Proof: `cargo test -p myosu-tui -- shell_state` runs ≥ 10 tests covering state mutations. All pass.

Current proof: `cargo test -p myosu-tui shell_state` passes 16 tests covering logging, layout,
Lobby→Game routing, help toggling, Game rendering, all non-Game screens, and help overlay bounds.

### M3: Add event loop tests to `myosu-tui`
Test that the event loop handles keyboard input, resize events, and pipe mode correctly.

Proof: `cargo test -p myosu-tui -- event_loop` runs ≥ 5 tests covering event handling. All pass.

### M4: Add property-based round-trip tests to `myosu-games`
Use `proptest` to test `GameType::from_bytes(g.to_bytes()) == g` for random game configurations.

Proof: `cargo test -p myosu-games -- serialization_roundtrip` runs ≥ 100 property tests. All pass.

Key files:
- `crates/myosu-games/src/traits.rs:GameType`
- `crates/myosu-games/src/traits.rs:GameConfig`

---

## Context and Orientation

Current test state must be measured against the live repo, not against old assumptions:
```
The implementation pass should inventory:
- current unit and integration tests in `myosu-tui`
- ignored tests that should become headless and CI-safe
- current inline tests in `myosu-games`
- missing property-based coverage in serialization and query types
```

The `myosu-tui` test strategy:
- Create `crates/myosu-tui/src/tests/` directory
- Use `crossterm::event::poll()` simulation with a mock clock
- Test state transitions in isolation from rendering
- Use `ratatui::mock::MockTerminal` for rendering tests

The `myosu-games` test strategy:
- Add `proptest = "1"` dependency to `crates/myosu-games/Cargo.toml`
- Test `GameType` and `GameConfig` round-trip serialization
- Test that `StrategyQuery` and `StrategyResponse` serialize/deserialize correctly

---

## Plan of Work

1. Add `proptest` dependency to `myosu-games/Cargo.toml`
2. Create `crates/myosu-tui/src/tests/` with screen, shell, and event integration tests
3. Create property tests for game serialization in `crates/myosu-games/src/tests/`
4. Run all tests and fix failures
5. Update CI and verification commands to run these tests
6. Reduce high-risk `myosu-tui` schema proof gaps when they block lane trust, even if they are
   not strictly part of the `myosu-games` property-test scope

---

## Concrete Steps

```bash
# Check current test count
cargo test -p myosu-tui -- --list 2>&1 | rg 'test' | wc -l
cargo test -p myosu-games -- --list 2>&1 | rg 'test' | wc -l

# Add proptest to myosu-games
echo 'proptest = "1"' >> crates/myosu-games/Cargo.toml
cargo test -p myosu-games traits::tests::serialization_roundtrip_strategy_response --quiet

# Create test files
touch crates/myosu-tui/src/tests/mod.rs
touch crates/myosu-tui/src/tests/screen_tests.rs
touch crates/myosu-tui/src/tests/shell_state_tests.rs
touch crates/myosu-tui/src/tests/event_tests.rs

# Run new tests
cargo test -p myosu-tui
cargo test -p myosu-games traits::tests::serialization_roundtrip_strategy_response --quiet

# Update the verification entrypoints
# Ensure CI and local verification run:
# cargo test -p myosu-games -- serialization_roundtrip
# cargo test -p myosu-tui
```

---

## Validation

- `cargo test -p myosu-tui` shows ≥ 20 passing tests
- `cargo test -p myosu-games -- serialization_roundtrip` shows ≥ 100 passing property tests
- `cargo test -p myosu-tui shell_state` shows ≥ 10 passing shell-state tests
- All tests pass on clean `cargo test`
- No warnings (all dead code removed or allowed)

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| `crossterm` event simulation doesn't work in headless CI | Use `crossterm::event::read()` polling with a mock event source |
| Property test finds a serialization bug | Fix the serialization/deserialization implementation |
| Test environment differs from production | Use a feature flag for integration tests that require a real TTY |
