# Specification: Gameplay CLI — Human vs Bot Poker

Source: Master spec AC-GP-01
Status: Draft
Date: 2026-03-16
Depends-on: GS-01..09 (chain for miner discovery), PE-01..04 (poker engine), MN-01..05 (miners serve strategies)

## Purpose

Build the `myosu-play` binary — a terminal application where a human plays
No-Limit Hold'em heads-up against the best available miner's strategy. This
is the consumer-facing product: solver quality directly translates into a
challenging, enjoyable opponent.

The player connects to the chain to discover the highest-ranked miner (by
Yuma incentive score), queries that miner's axon for action distributions,
and samples bot actions from those distributions.

**Key design constraint**: gameplay must feel responsive. Bot decisions should
appear within 500ms. If the miner is unreachable, fall back to a random
strategy rather than blocking.

## Whole-System Goal

Current state:
- Chain pallet tracks miner incentive scores (GS-05, GS-06)
- Miners serve strategy queries via HTTP axon (MN-03)
- Poker engine provides game lifecycle (PE-01)
- No gameplay interface exists

This spec adds:
- `myosu-play` CLI binary
- Best-miner discovery from chain incentive scores
- Interactive game loop (human decisions via stdin)
- Bot play via miner axon strategy queries
- Hand history recording
- Session statistics (hands played, win rate, profit)

If all ACs land:
- `myosu-play --chain ws://localhost:9944 --subnet 1` starts a poker session
- Human and bot alternate decisions in a proper NLHE heads-up game
- Bot actions are sampled from the best miner's trained strategy
- Each completed hand is recorded and stats are displayed

Still not solved here:
- TUI (graphical terminal interface) — text prompts only
- Multiplayer (human vs human)
- Web interface
- Real money or token wagering
- Multiple game variants

12-month direction:
- Rich TUI with card graphics (ratatui)
- Web-based gameplay interface
- Multi-table support
- Tournament mode with increasing blinds

## Why This Spec Exists As One Unit

- Best-miner discovery, game loop, bot integration, and hand recording are
  all needed for one outcome: "a human plays a complete hand of poker"
- The game loop is inherently sequential (deal → bet → flop → bet → ... → showdown)
- Splitting would create specs that can't demonstrate a playable experience

## Scope

In scope:
- CLI binary with chain connection
- Best-miner selection from incentive scores
- Interactive text-based game loop
- Bot action sampling from miner strategy
- Hand history JSON recording
- Session statistics display
- Fallback to random strategy on miner disconnection

Out of scope:
- TUI with card graphics — text prompts are sufficient
- Real token wagering — play money only
- Multiplayer (human vs human)
- Tournament mode
- Coaching/suggestion display
- Sound or animations

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| CLI + main | New | crates/myosu-play/src/main.rs |
| Chain client | New (shared) | crates/myosu-play/src/chain.rs |
| Best miner selector | New | crates/myosu-play/src/discovery.rs |
| Game loop | New | crates/myosu-play/src/game_loop.rs |
| Bot player | New | crates/myosu-play/src/bot.rs |
| Hand recorder | New | crates/myosu-play/src/recorder.rs |

---

### AC-GP-01: Best Miner Discovery

- Where: `crates/myosu-play/src/discovery.rs (new)`
- How: Query the chain for the miner with the highest `Incentive` score on the
  target subnet. Read their `Axons` entry for IP:port. Verify the miner is
  responsive (`GET /health`). If the best miner is unreachable, try the
  second-best, etc. Cache the selection for the session duration (re-check
  every 100 hands in case a better miner appears).
- Required tests:
  - `cargo test -p myosu-play discovery::tests::finds_best_miner`
  - `cargo test -p myosu-play discovery::tests::fallback_to_second_best`
  - `cargo test -p myosu-play discovery::tests::no_miners_uses_random`
- Pass/fail:
  - Returns miner with highest incentive score
  - Unreachable best miner → falls back to next best
  - No responsive miners → returns None (game uses random bot)
- Blocking note: players should face the strongest bot available.
- Rollback condition: incentive scores not yet populated (first epoch hasn't run).

### AC-GP-02: Interactive Game Loop

- Where: `crates/myosu-play/src/game_loop.rs (new)`
- How: Text-based game loop using robopoker's `Game` engine:

  ```
  === Hand #1 ===
  Your cards: A♠ K♥
  Pot: 3 BB | Your stack: 98.5 BB | Bot stack: 98.5 BB

  Bot posts small blind (0.5 BB)
  You post big blind (1 BB)

  Your action? [f]old [c]heck [r]aise [amount]: r 3
  You raise to 3 BB.
  Bot calls 3 BB.

  --- Flop: T♠ 7♥ 2♣ ---
  Pot: 6 BB
  Your action? [f]old [c]heck [r]aise [amount]:
  ```

  Use `Game::root()` to start each hand. Human decisions read from stdin.
  Bot decisions from the miner's strategy (GP-03). Apply actions via
  `Game::apply()`. At showdown, evaluate hands and display results.

  Between hands: display running stats (hands played, win rate, BB/hand).
- Required tests:
  - `cargo test -p myosu-play game_loop::tests::hand_completes_fold`
  - `cargo test -p myosu-play game_loop::tests::hand_completes_showdown`
  - `cargo test -p myosu-play game_loop::tests::invalid_input_reprompts`
  - `cargo test -p myosu-play game_loop::tests::all_in_resolves`
- Pass/fail:
  - Fold at preflop → hand ends, pot awarded to opponent
  - Full hand to showdown → best hand wins pot
  - Invalid input ("xyz") → reprompt without crash
  - All-in → remaining streets dealt, showdown resolves
  - Stats correctly track wins, losses, and BB/hand
- Blocking note: this is the user-facing product.
- Rollback condition: robopoker's Game struct can't represent the game states needed for display.

### AC-GP-03: Bot Strategy Integration

- Where: `crates/myosu-play/src/bot.rs (new)`
- How: When it's the bot's turn:
  1. Convert current `Game` to `NlheInfo` via the encoder
  2. Query the miner's axon: `POST /strategy` with serialized info
  3. Receive action distribution: `[(Fold, 0.1), (Call, 0.3), (Raise(1/2), 0.6)]`
  4. Sample one action from the distribution using thread RNG
  5. Apply the sampled action to the game

  Fallback: if miner query fails, sample uniformly from legal actions.

  Bot actions should feel natural — add a small delay (200-500ms) before
  acting to simulate "thinking time."
- Required tests:
  - `cargo test -p myosu-play bot::tests::query_and_sample_action`
  - `cargo test -p myosu-play bot::tests::fallback_on_timeout`
  - `cargo test -p myosu-play bot::tests::sampled_action_is_legal`
- Pass/fail:
  - Bot queries miner and returns a legal action
  - Miner timeout → bot plays random legal action
  - Sampled action is always in the legal action set
  - Action distribution with 0.0 probabilities → those actions never sampled
- Blocking note: the bot is what makes the game challenging.
- Rollback condition: miner query latency > 500ms makes gameplay unresponsive.

### AC-GP-04: Hand History Recording

- Where: `crates/myosu-play/src/recorder.rs (new)`
- How: Record each completed hand as JSON:
  ```json
  {
    "hand_number": 1,
    "hero_cards": "A♠ K♥",
    "board": "T♠ 7♥ 2♣ 9♦ J♠",
    "actions": [
      {"player": "hero", "action": "raise", "amount": 3.0},
      {"player": "bot", "action": "call", "amount": 3.0},
      ...
    ],
    "result": {"winner": "hero", "pot": 12.0, "hero_profit": 6.0},
    "miner_uid": 0,
    "subnet_id": 1
  }
  ```
  Save to `{data-dir}/hands/hand_{N}.json`. Also maintain a session summary
  file with running statistics.
- Required tests:
  - `cargo test -p myosu-play recorder::tests::record_hand`
  - `cargo test -p myosu-play recorder::tests::session_stats_update`
- Pass/fail:
  - Hand JSON file created after hand completes
  - All actions recorded in order
  - Session stats file shows correct totals
  - Handles disk full gracefully (log warning, continue playing)
- Blocking note: hand history is essential for player review and debugging.
- Rollback condition: N/A — recording failures shouldn't block gameplay.

---

## Decision Log

- 2026-03-16: Text prompts, not TUI — minimal implementation that proves
  the product thesis. TUI is a separate enhancement spec.
- 2026-03-16: 200-500ms bot delay — feels more natural than instant response.
- 2026-03-16: Best-miner by incentive score — directly ties gameplay quality
  to the Yuma Consensus output.
- 2026-03-16: JSON hand history — human-readable, easy to parse for analysis.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Discovers best miner from chain | Discovery | GP-01 |
| 2 | Human folds preflop → hand ends correctly | Game loop | GP-02 |
| 3 | Complete hand to showdown with bot | Bot integration | GP-02, GP-03 |
| 4 | Bot plays from miner strategy (not random) | Strategy quality | GP-03 |
| 5 | Hand history JSON file written | Recording | GP-04 |
| 6 | 10 hands played without crash | Stability | All |
