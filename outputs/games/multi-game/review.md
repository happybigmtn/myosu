# `games:multi-game` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The lane is correctly scoped. All three source specs (031626-06, 031626-16, 031626-17) are coherent and internally consistent with each other and with the existing `games:traits` crate. The `GameType::LiarsDice` and `GameParams::LiarsDice` already exist in `myosu-games`, meaning the traits lane already prepared the hook — the multi-game lane just needs to implement the concrete game engine that plugs into it. No spec reopening is needed.

The lane has no critical design ambiguity: the Liar's Dice variant (1 die each, 6 faces, 2 players) is explicitly specified with exact state-space numbers (147,420 terminal states, ~24,576 info sets). The zero-change architectural claim is verifiable via `git diff`. The spectator protocol decision (local relay for Phase 0, miner axon for Phase 1) is explicitly documented with rationale.

---

## Proof Expectations

The following commands must all exit 0 before the lane is considered complete:

```bash
# Bootstrap gate (crate integrity)
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice

# Slice 2 — game engine (MG-01)
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied

# Slice 3 — solver and Nash verification (MG-02)
cargo test -p myosu-games-liars-dice solver::tests::train_to_nash
cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works

# Slice 4 — ExploitMetric registration (CS-01)
cargo test -p myosu-games registry::tests::all_game_types_have_metrics
cargo test -p myosu-games registry::tests::random_baseline_positive
cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline

# Slice 5 — spectator relay (SP-01)
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::discover_local_sessions

# Slice 6 — spectator TUI (SP-02)
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play

# Slice 7 — zero-change verification (MG-03)
cargo test -p myosu-games registry::tests::known_game_types --quiet
cargo test -p myosu-games-poker
cargo test -p myosu-games-liars-dice
# git diff crates/myosu-games/src/ crates/myosu-games-poker/src/  # must be empty
```

---

## Remaining Blockers

### Blocker 1: `myosu-games-liars-dice` Crate Is Entirely Greenfield (Critical)

**Location**: `crates/myosu-games-liars-dice/` does not exist.

**What must happen**: The implementation lane must create all source files. This is normal for a bootstrap lane — not a design problem.

**Risk if ignored**: Lane cannot progress at all.

### Blocker 2: `CfrGame: Copy` Constraint for Variable-Length Bid History (High)

**Location**: `crates/myosu-games-liars-dice/src/game.rs`

**What must happen**: `CfrGame` requires `Copy`. The bid history is variable-length. The spec calls for a fixed-size array (max 12 bids for 1-die Liar's Dice) with a sentinel value to distinguish empty slots. If this approach doesn't work, the zero-change claim may be broken — but the solution is an implementation detail, not a design change.

**Risk if ignored**: `CfrGame: Copy` is a hard constraint from robopoker. If `LiarsDiceGame` can't be `Copy`, the whole proof-of-architecture fails.

### Blocker 3: `ExploitMetric` Not in `myosu-games` (High)

**Location**: `crates/myosu-games/src/traits.rs` — no `ExploitMetric` type exists.

**What must happen**: Slice 4 must add `ExploitMetric` and `ExploitScale` to `myosu-games`. This is a backwards-compatible addition (adds new exports, no existing API changed).

**Risk if ignored**: Cross-game scoring cannot work. The validator oracle and lobby TUI cannot display game-specific exploitability units.

### Blocker 4: `SpectatorRelay` and Spectator TUI Are Both Missing (Medium)

**Location**: `crates/myosu-play/src/spectate.rs` and `crates/myosu-tui/src/screens/spectate.rs` do not exist.

**What must happen**: Both files must be created. The spectator relay requires careful fog-of-war enforcement — hole cards must never be sent during play.

**Risk if ignored**: Spectator mode is a key user-visible feature for agent-vs-agent viewing.

---

## Risks the Implementation Lane Must Preserve

1. **Zero-change property**: Adding `myosu-games-liars-dice` must not modify `crates/myosu-games/src/` or `crates/myosu-games-poker/src/`. This is the architectural claim. If robopoker's trait signatures change, the implementation lane must flag this as a breaking change.

2. **`CfrGame: Copy` constraint**: Liar's Dice game state must be `Copy`. The implementation must use a fixed-size array with sentinel for the bid history, not a `Vec`.

3. **Exploitability normalization contract**: The validator oracle formula `(1.0 - (exploit / metric.random_baseline).min(1.0)).clamp(0.0, 1.0)` must be preserved. Changing this formula breaks consensus weights.

4. **Fog-of-war enforcement at relay**: The `SpectatorRelay` must strip hole cards before emitting events. The TUI receives an already-sanitized stream.

---

## Risks the Implementation Lane Should Reduce

1. **Liar's Dice 1-die variant is a proof, not a product**: The 1-die variant converges in seconds and is trivially solvable. The 6-die variant (AC-MG-04) is the realistic game variant. Keep the 1-die proof self-contained so expansion to 6-die (future work) doesn't require re-architecting.

2. **No Liar's Dice CLI gameplay**: The multi-game spec explicitly excludes CLI gameplay. The implementation lane should not add it — it's out of scope.

3. **Spectator relay is Phase 0 only**: The local Unix socket relay is for single-machine use. Phase 1 (miner-axon WebSocket) requires agent experience APIs (AX-01..06). The implementation lane should implement Phase 0 only and leave Phase 1 markers in the code.

---

## Is the Lane Ready for an Implementation-Family Workflow Next?

**Yes — with conditions.**

The specification is stable. The implementation lane can begin with Slice 1 (crate skeleton) immediately. The `GameType::LiarsDice` and `GameParams::LiarsDice` already exist in `myosu-games`, meaning the traits hook is prepared. No further spec work is needed before the implementation lane starts.

The conditions for proceeding:

1. Slice 2 (game engine) must succeed before Slice 3 (solver) and Slice 7 (zero-change verification) can run
2. Slice 4 (ExploitMetric) must succeed before cross-game scoring can be integrated with the validator oracle
3. Slice 5 (spectator relay) must succeed before Slice 6 (spectator TUI)
4. All 22 test commands above must exit 0 before the lane is marked complete

If the `CfrGame: Copy` constraint (blocker 2) cannot be satisfied, the lane must be reopened — this would indicate a fundamental incompatibility between Liar's Dice and the robopoker trait system.

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | `games:multi-game` extends `games:traits` with `ExploitMetric` and `GameType::LiarsDice`. Relationship is additive. The `games:traits` lane is stable. |
| `games:poker-engine` | Both consume `myosu-games`. Poker-engine is the reference implementation; multi-game is the proof that the architecture generalizes. They are independent and can proceed in parallel. |
| `services:miner` | Will serve Liar's Dice queries once implemented. Not needed for the proof-of-architecture. |
| `services:validator-oracle` | Will use `ExploitMetric` for cross-game weight normalization (AC-CS-02). Slice 4 (metric registration) is the prerequisite. |
| `product:play-tui` | Spectator TUI is part of play-tui. Slice 5 and 6 integrate with the play TUI. |

The `services:miner` and `services:validator-oracle` lanes are blocked on `ExploitMetric` registration (slice 4) for cross-game scoring — not on Liar's Dice itself. The metric system is orthogonal to which game engine is running.
