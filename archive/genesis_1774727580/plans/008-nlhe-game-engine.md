# NLHE Game Engine Implementation

**Plan ID:** 008
**Status:** In Progress — `myosu-games-poker` now exists in the workspace with NLHE state, action parsing, a real `GameRenderer` implementation, a profile-backed robopoker query bridge, a `bincode` wire codec for strategy transport, deterministic recommended-action helpers for known profile-backed cases, a request-side inference adapter that lowers `hero cards + board + action history + bucket` into robopoker `Partial`/`NlheInfo`, manifest-backed encoder artifact verification and loading, a first honest `PokerSolver` wrapper with checkpoint, query, and training entrypoints, and a built-in `myosu-games` registry surface for known game types; full `CfrGame` integration and proof against a complete abstraction artifact are still pending
**Priority:** FOUNDATION — core product value proposition

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, `crates/myosu-games-poker/` will implement a working NLHE (No-Limit Hold'em) thin-wrap of robopoker's MCCFR solver. A strategy query (given a game state) will return a recommended action. The `GameRenderer` trait will render poker state to the TUI. This is the narrowest wedge — the core value of Myosu is a working game that can solve a situation and recommend an action.

---

## Progress

- [x] Create `crates/myosu-games-poker/` in the main workspace
- [ ] Implement `CfrGame` trait for NLHE via robopoker thin-wrap
- [x] Add a manifest-backed abstraction artifact loader and verifier
- [x] Implement `GameRenderer` for poker state → TUI rendering
- [x] Wire built-in game types into `myosu-games` registry
- [x] Add a first real test surface for action parsing, snapshot formatting, and TUI renderer behavior
- [x] Add a first real profile-backed strategy query path for precomputed `NlheInfo`
- [x] Add a binary wire codec for `NlheInfoKey`, `StrategyQuery`, and `StrategyResponse`
- [x] Verify strategy query returns correct action for known profile-backed test cases
- [x] Add a request-side inference adapter for cards, replay history, and abstraction bucket
- [x] Add encoder-backed bucket derivation from serialized lookup artifacts
- [x] Add a first honest solver wrapper with checkpoint, query, and typed training errors

---

## Surprises & Discoveries

- The first honest slice was smaller than the old plan text implied. There was no active
  `myosu-games-poker` crate at all, but there also was no registry surface inside `myosu-games`
  yet, so the truthful first move was crate creation plus a concrete NLHE state/rendering wedge.
- The `GameRenderer` trait does not live in `myosu-games`; it already lives in `myosu-tui`. The
  poker crate should implement that contract directly instead of inventing a second renderer
  abstraction.
- Creating the poker crate exposed a latent dependency bug in `myosu-games`: it used
  `serde_json` without enabling `std` or `alloc`. Fixing that made the base game-abstraction crate
  honest for downstream consumers.
- The first honest solver-facing slice is narrower than full `CfrGame` integration. We can already
  expose a useful bridge over `rbp-nlhe` by accepting precomputed `NlheInfo` keys and returning
  normalized `StrategyResponse<NlheEdge>` values from a stored `NlheProfile`.
- The plan's old "implement Encoder using bincode" language was hiding two different jobs. Binary
  transport for query/response payloads is now solved with a concrete `wire.rs` module, but that
  is still not the same thing as constructing a correct `rbp_nlhe::NlheEncoder` from real
  abstraction artifacts.
- Because `rbp_nlhe::NlheProfile` exposes its encounter map publicly, we can verify
  recommendation logic honestly today. A handcrafted profile for a known `NlheInfo` now proves
  that the transport/query layer can select the expected best action without pretending the
  snapshot→info bridge already exists.
- The truthful bridge from Myosu-side state into robopoker is not `NlheSnapshot` alone. The repo
  now has a separate request surface carrying the missing replay inputs explicitly: hero position,
  observed cards, exact action history, and abstraction bucket. That is enough to build
  `Partial`, then `NlheInfo`, then a wire-safe strategy query.
- The local crate can now consume serialized encoder artifacts directly. That gives us a truthful
  path from observed cards to abstraction bucket without patching upstream robopoker, and proves
  the missing upstream constructor can be worked around safely inside `myosu-games-poker`.
- The new solver wrapper exposed the next real boundary cleanly: sparse or incomplete encoder
  artifacts are enough for query and checkpoint tests, but not for truthful MCCFR training or
  exploitability proofs. Those upstream panics now cross the crate boundary as typed errors
  instead of tearing down the process.
- Registry wiring turned out to be cheaper than the old plan implied. Because `myosu-games-poker`
  already depends on `myosu-games`, the registry cannot construct poker engines directly without a
  cycle. The honest registry surface is a built-in game descriptor map that resolves on-chain game
  type bytes into known variants and metadata, leaving engine instantiation to downstream crates.
- The next useful artifact step was not a miner bootstrap script or a clustering binary inside this
  crate. It was a trust boundary: loading a directory with `manifest.json`, verifying per-file and
  total SHA-256 hashes, and only then merging the encoded lookup tables into one `NlheEncoder`.
  That turns “some bytes” into a named abstraction artifact that later plans can depend on.

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

### M1: Create `crates/myosu-games-poker/` in the main workspace
Build the crate in the active repo. Historical worktrees can be consulted for
reference, but the implementation must be re-reviewed from scratch before use.
Add to workspace `Cargo.toml`. Verify `cargo check -p myosu-games-poker`
passes.

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
  now exists in active workspace
  currently contains: NLHE actions, snapshot state, TUI renderer,
  wire-safe `NlheInfoKey`, profile-backed robopoker query bridge,
  `bincode` binary transport helpers for key/query/response payloads,
  request-side inference lowering into `Partial` / `NlheInfo`,
  manifest-backed encoder artifact verification and loading,
  and a `PokerSolver` wrapper with checkpoint/query/training entrypoints
  `crates/myosu-games/` now also exports a built-in `GameRegistry`
  still missing: full `CfrGame` integration and proof against a complete encoder artifact
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

The `myosu-games` crate re-exports the `GameConfig` and trait types. `myosu-games-poker`
implements poker-specific state and TUI rendering now, and will add the robopoker thin-wrap next.
The chain's game-solver pallet eventually queries `myosu-games` via the
`StrategyQuery`/`StrategyResponse` types.

---

## Plan of Work

1. Create `myosu-games-poker` in the active repo
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
