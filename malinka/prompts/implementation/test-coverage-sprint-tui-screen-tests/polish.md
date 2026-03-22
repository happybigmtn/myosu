# Add screen transition tests to myosu-tui Lane — Fixup

Fix only the current slice for `test-coverage-sprint-tui-screen-tests`.

Current Slice Contract:
Plan file:
- `genesis/plans/005-test-coverage-sprint.md`

Child work item: `test-coverage-sprint-tui-screen-tests`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Test Coverage Sprint

**Plan ID:** 005
**Status:** New
**Priority:** CRITICAL — this unblocks all Phase 1 and Phase 2 work

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `myosu-tui` will have a comprehensive integration test suite covering screen transitions, shell state, event handling, and theme rendering. `myosu-games` will have property-based tests for `GameConfig` and `GameType` round-trip serialization. Every PR touching these crates will be automatically tested.

This plan is Phase 0 — it fixes the inverted quality problem where the chain pallet has 4,000+ tests and the user-facing code has none.

---

## Progress

- [ ] Add screen transition tests to `myosu-tui`
- [ ] Add shell state tests to `myosu-tui`
- [ ] Add event loop tests to `myosu-tui`
- [ ] Add property-based round-trip tests to `myosu-games`
- [ ] Integrate into Fabro quality gates (proof commands must run these tests)

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Start with `myosu-tui` tests because it has zero coverage and is the primary user-facing surface.
  Rationale: A 3,574-line crate with zero tests is a silent breakage risk. The TUI is also the most interaction-heavy code, making it ideal for integration tests.
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

Current test state:
```
crates/myosu-tui/src/
  3,574 lines
  0 test files
  0 #[test] annotations

crates/myosu-games/src/traits.rs
  371 lines
  9 doc tests (#[test] inline in documentation)
  No property-based tests
  No integration tests
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
5. Update Fabro proof commands to run these tests

---

## Concrete Steps

```bash
# Check current test count
cargo test -p myosu-tui -- --list 2>&1 | rg 'test' | wc -l
cargo test -p myosu-games -- --list 2>&1 | rg 'test' | wc -l

# Add proptest to myosu-games
echo 'proptest = "1"' >> crates/myosu-games/Cargo.toml
cargo test -p myosu-games

# Create test files
touch crates/myosu-tui/src/tests/mod.rs
touch crates/myosu-tui/src/tests/screen_tests.rs
touch crates/myosu-tui/src/tests/shell_state_tests.rs
touch crates/myosu-tui/src/tests/event_tests.rs

# Run new tests
cargo test -p myosu-tui
cargo test -p myosu-games

# Update Fabro proof command (in the relevant run config)
# Change: cargo test -p myosu-games
# To: cargo test -p myosu-games -- serialization_roundtrip
# And: cargo test -p myosu-tui
```

---

## Validation

- `cargo test -p myosu-tui` shows ≥ 20 passing tests
- `cargo test -p myosu-games -- serialization_roundtrip` shows ≥ 100 passing property tests
- All tests pass on clean `cargo test`
- No warnings (all dead code removed or allowed)

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| `crossterm` event simulation doesn't work in headless CI | Use `crossterm::event::read()` polling with a mock event source |
| Property test finds a serialization bug | Fix the serialization/deserialization implementation |
| Test environment differs from production | Use a feature flag for integration tests that require a real TTY |


Workflow archetype: implement

Review profile: ux

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Screen transition integration tests for the five-panel shell in myosu-tui
- How: Add tests verifying mode transitions (normal, pipe, command) in the screen manager
- Required tests: cargo test -p myosu-tui -- screen
- Verification plan: cargo test -p myosu-tui -- screen runs at least 5 tests and all pass
- Rollback condition: Screen transition tests fail or coverage drops below 5 test cases

Proof commands:
- `cargo test -p myosu-tui -- screen`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
