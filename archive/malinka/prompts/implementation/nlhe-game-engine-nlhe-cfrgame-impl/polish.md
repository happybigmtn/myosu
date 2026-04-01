# Implement CfrGame for NLHE (heads-up) Lane — Fixup

Fix only the current slice for `nlhe-game-engine-nlhe-cfrgame-impl`.

Current Slice Contract:
Plan file:
- `genesis/plans/008-nlhe-game-engine.md`

Child work item: `nlhe-game-engine-nlhe-cfrgame-impl`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# NLHE Game Engine Implementation

**Plan ID:** 008
**Status:** New
**Priority:** FOUNDATION — core product value proposition

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `crates/myosu-games-poker/` will implement a working NLHE (No-Limit Hold'em) thin-wrap of robopoker's MCCFR solver. A strategy query (given a game state) will return a recommended action. The `GameRenderer` trait will render poker state to the TUI. This is the narrowest wedge — the core value of Myosu is a working game that can solve a situation and recommend an action.

---

## Progress

- [ ] Bring `crates/myosu-games-poker/` from worktree into main workspace
- [ ] Implement `CfrGame` trait for NLHE via robopoker thin-wrap
- [ ] Implement `Encoder` trait for NLHE wire serialization
- [ ] Implement `GameRenderer` for poker state → TUI rendering
- [ ] Wire into `myosu-games` game registry
- [ ] Add comprehensive tests (not just doc tests)
- [ ] Verify strategy query returns correct action for known test cases

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Implement Kuhn Poker (2-player, 3 cards) first as the reference implementation, then extend to full NLHE.
  Rationale: Kuhn Poker has only 12 info sets and a known Nash equilibrium. Full NLHE is the target, but validating against a simple game first avoids MCCFR implementation errors.
  Rationale source: `specs/031626-02a-game-engine-traits.md` uses RPS as validation target; Kuhn Poker is analogous for imperfect-information games.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Use robopoker's `RemoteProfile` as the primary MCCFR driver, not a local/copy.
  Rationale: The spec (`specs/031626-02b-poker-engine.md`) explicitly decided against a wrapper function for `RemoteProfile`. The thin-wrap should expose the full `RemoteProfile` API, not abstract it away.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Checkpoint format uses `bincode` (not JSON) for serialization.
  Rationale: Bincode is more compact, faster, and appropriate for binary game state. JSON was explicitly rejected.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Port `crates/myosu-games-poker/` to main workspace
Copy from `.worktrees/autodev-live/crates/myosu-games-poker/` into `crates/myosu-games-poker/`. Add to workspace `Cargo.toml`. Verify `cargo check -p myosu-games-poker` passes.

Proof: `cargo check -p myosu-games-poker` passes with zero warnings.

Key files:
- `crates/myosu-games-poker/src/lib.rs`
- `crates/myosu-games-poker/src/kuhn.rs` (Kuhn Poker implementation)
- `crates/myosu-games-poker/src/nlhe.rs` (NLHE implementation)

### M2: Implement `CfrGame` for Kuhn Poker
Kuhn Poker implements 3-card poker with known Nash equilibrium. The exploitability of a Nash equilibrium strategy is exactly 1/72 chips per game.

Proof: `cargo test -p myosu-games-poker -- kuhn` runs the exploitability test; result is within 1e-6 of 1/72.

Key files:
- `crates/myosu-games-poker/src/kuhn.rs:CfrGame impl`

### M3: Implement `CfrGame` for NLHE (heads-up)
Heads-up NLHE via robopoker `RemoteProfile`. Implement the `CfrGame` trait that wraps the MCCFR solver.

Proof: `cargo test -p myosu-games-poker -- nlhe` runs the strategy query test; given a known flop situation, returns a recommended action.

Key files:
- `crates/myosu-games-poker/src/nlhe.rs:CfrGame impl`
- `crates/myosu-games-poker/src/robopoker_bridge.rs` (thin-wrap of rbp-core)

### M4: Implement `GameRenderer` for poker state
Wire the TUI rendering for poker state (hole cards, board cards, pot, positions).

Proof: `cargo check -p myosu-games-poker` includes `GameRenderer` impl; no missing trait implementations.

Key files:
- `crates/myosu-games-poker/src/renderer.rs`

### M5: Wire into game registry and verify strategy query
Register NLHE in `myosu-games` game registry. Verify end-to-end: game state → strategy query → recommended action.

Proof: `cargo test -p myosu-games-poker -- strategy_query` passes; output is a valid poker action (fold/call/raise) for a known test case.

### M6: Comprehensive test suite
Add unit tests, integration tests, and at least one property test for NLHE game state.

Proof: `cargo test -p myosu-games-poker` passes with ≥ 50 tests; coverage report shows ≥ 60% line coverage on `src/nlhe.rs` and `src/kuhn.rs`.

---

## Context and Orientation

Current state:
```
crates/myosu-games-poker/
  EXISTS in: .worktrees/autodev-live/
  NOT in main workspace
  Status: scaffold — compiles, passes cargo test, does not run MCCFR
```

Target architecture:

```
crates/myosu-games-poker/src/
├── lib.rs                  # exports, re-exports
├── kuhn.rs                 # CfrGame impl for 3-card Kuhn Poker
├── nlhe.rs                 # CfrGame impl for heads-up NLHE
├── robopoker_bridge.rs     # thin-wrap of rbp-core/rbp-mccfr
├── renderer.rs             # GameRenderer impl for poker
├── encoding.rs             # Encoder impl (bincode)
└── tests/
    ├── kuhn_tests.rs       # Nash equilibrium validation
    ├── nlhe_tests.rs       # Strategy query tests
    └── roundtrip_tests.rs  # Serialization round-trip
```

The `myosu-games` crate re-exports the `GameConfig` and trait types. `myosu-games-poker` implements those traits for NLHE. The chain's game-solver pallet queries `myosu-games` via the `StrategyQuery`/`StrategyResponse` types.

---

## Plan of Work

1. Copy `myosu-games-poker` from worktree to main
2. Add to workspace `Cargo.toml`
3. Fix any compilation errors
4. Implement Kuhn Poker `CfrGame` as reference (known equilibrium)
5. Verify Kuhn equilibrium exploitability is correct
6. Implement NLHE `CfrGame` thin-wrap of robopoker
7. Implement `Encoder` using bincode
8. Implement `GameRenderer`
9. Wire into game registry
10. Add comprehensive tests
11. Verify strategy query end-to-end

---

## Concrete Steps

```bash
# Copy from worktree
cp -r .worktrees/autodev-live/crates/myosu-games-poker crates/
git add crates/myosu-games-poker/

# Add to workspace
echo '  "crates/myosu-games-poker",' >> Cargo.toml

# Check compilation
cargo check -p myosu-games-poker 2>&1 | head -50

# Run existing tests
cargo test -p myosu-games-poker

# Run exploitability test (Kuhn Poker known equilibrium)
cargo test -p myosu-games-poker -- kuhn_equilibrium
# Expected: exploitability ≈ 0.013888... (1/72)

# Run strategy query test
cargo test -p myosu-games-poker -- strategy_query
# Expected: fold/call/raise action for known board state
```

---

## Validation

- `cargo check -p myosu-games-poker` passes
- `cargo test -p myosu-games-poker` shows ≥ 50 passing tests
- Kuhn Poker exploitability test: result within 1e-6 of 1/72
- Strategy query for NLHE: returns valid action (fold/call/raise) for test case
- `cargo clippy -p myosu-games-poker -- -D warnings` passes
- `test -f crates/myosu-games-poker/src/renderer.rs` — `GameRenderer` implemented

---

## Failure Scenarios

| Scenario | Handling |
|----------|----------|
| Robopoker `RemoteProfile` API changed | Pin to specific robopoker revision; update thin-wrap if API is stable |
| Bincode serialization incompatible with future checkpoint format | Keep bincode for now; plan migration to scale-codec in Phase 3 |
| MCCFR convergence too slow for tests | Use Kuhn Poker (12 info sets) as the fast test target; mock NLHE for CI |
| GameRenderer doesn't match TUI expectations | Test against the TUI's `Renderable` trait contract explicitly |


Workflow archetype: implement

Review profile: foundation

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: NLHE CfrGame trait implementation via robopoker RemoteProfile thin-wrap
- How: CfrGame impl for heads-up NLHE returns a recommended action for a known flop situation
- Required tests: cargo test -p myosu-games-poker -- nlhe
- Verification plan: Strategy query test returns a valid poker action (fold/call/raise) for known test case
- Rollback condition: Robopoker RemoteProfile API changes or strategy query returns invalid action

Proof commands:
- `cargo test -p myosu-games-poker -- nlhe`

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
