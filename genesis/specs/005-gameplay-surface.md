# Specification: Gameplay Surface

Source: Reverse-engineered from crates/myosu-play (main.rs, blueprint.rs, discovery.rs, live.rs)
Status: Draft
Depends-on: 001-game-trait-framework, 004-terminal-ui-framework

## Purpose

The gameplay surface provides interactive human play against trained
imperfect-information game strategies. It orchestrates blueprint loading,
miner discovery, live strategy queries, and the TUI/pipe rendering pipeline
into a unified gameplay experience. A human player can play poker or Liar's
Dice hands against the best available strategy, receiving real-time advice from
either a local blueprint artifact or a live miner.

The primary consumer is a human player or an automated agent connecting via the
pipe protocol.

## Whole-System Goal

Current state: The gameplay binary is fully implemented with three modes (train,
pipe, smoke-test), a four-tier artifact discovery system, chain-based miner
discovery, and live HTTP strategy queries with freshness tracking.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: A human or agent can play imperfect-information games against
the best available strategy, with graceful degradation when artifacts, chain, or
live miners are unavailable.

Still not solved here: Strategy training (miner responsibility), quality scoring
(validator responsibility), and on-chain coordination (pallet responsibility)
are separate concerns.

## Scope

In scope:
- Three operational modes: train (interactive TUI), pipe (agent protocol),
  smoke-test (deterministic validation)
- Blueprint resolution with four-tier fallback hierarchy
- Chain-based miner discovery selecting the highest-incentive miner
- Live HTTP strategy queries with health checking
- Background live advice refresh with freshness tracking (Fresh/Stale/Offline)
- Startup status reporting with structured metadata
- Smoke test validation with configurable requirement flags
- Multi-game support (poker, Liar's Dice) selected at startup

Out of scope:
- Strategy training or MCCFR computation
- Validator scoring or weight submission
- Chain state management or registration
- The TUI framework itself (handled by myosu-tui)
- Wire protocol encoding details (handled by game crates)

## Current State

The binary exists at crates/myosu-play. It supports two game types at startup:
Poker (with blueprint/live advice) and Liar's Dice (demo renderer only).

Train mode initializes the TUI shell at Screen::Lobby, loads the game surface
via blueprint resolution, and optionally spawns a background live advice refresh
task that polls a discovered miner every 250ms with a 5-second timeout per
query. The refresh task tracks connectivity state transitions (Fresh, Stale,
Offline) and emits UI status updates only on transitions.

Pipe mode outputs a STATUS line with startup metadata (advice source, selection
strategy, origin, discovery state, live query state), an INFO line with protocol
version and detailed source information, then a STATE line with the current game
state. It enters a read-eval-print loop: each input line produces an ACTION
(advances state, prints new STATE), CLARIFY (ambiguous input with legal
actions), ERROR (invalid input with legal actions), or QUIT response. Pipe mode
refreshes live queries before each STATE output.

Smoke test mode runs deterministic game progressions without TUI rendering. For
poker: PREFLOP through call/call/call/check to completion. For Liar's Dice:
state output and a bid demonstration. Three requirement flags
(--require-artifact, --require-discovery, --require-live-query) allow CI to
validate specific integration points.

Blueprint resolution follows a four-tier hierarchy: (1) explicit --checkpoint
and --encoder-dir arguments, (2) auto-discovery from the MYOSU_BLUEPRINT_DIR
environment variable or default codexpoker home directory, (3) fallback to a
generated demo renderer on load failure, (4) error if no renderer can be
constructed. Each tier records its source, selection strategy, origin, and
reason for metadata reporting.

Miner discovery queries the chain via JSON-RPC for all chain-visible miners on
the specified subnet, sorts by incentive descending then UID ascending, and
selects the first miner with nonzero incentive and a valid endpoint.

Live query sends an HTTP GET to /health (expecting {"status":"ok"}) then POSTs a
wire-encoded strategy query to /strategy, decoding the response and extracting
the recommended action. Unspecified IP addresses (0.0.0.0) are resolved to
loopback.

Startup state cascades: Empty advice state is terminal. Partial advice state or
any discovery/live-query issue yields Partial. Otherwise the advice state
propagates.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Train mode | run_train() with TUI shell and live refresh | Reuse | Interactive gameplay proven |
| Pipe mode | run_pipe() with structured text protocol | Reuse | Agent integration proven |
| Smoke test | run_smoke_test() with poker and dice progressions | Reuse | CI validation proven |
| Blueprint resolution | Four-tier hierarchy in blueprint.rs | Reuse | Graceful degradation proven |
| Miner discovery | Chain-based discovery via discover_best_chain_visible_miner | Reuse | Incentive-ranked selection |
| Live queries | HTTP health+strategy queries in live.rs | Reuse | Background refresh with freshness |

## Non-goals

- Training strategies or running MCCFR iterations.
- Scoring or validating miner strategies.
- Registering on-chain or submitting transactions.
- Implementing the TUI layout or rendering framework.
- Providing a web or graphical gameplay interface.

## Behaviors

On startup, the binary parses CLI arguments to determine the mode (train, pipe,
or smoke-test), game type (Poker or LiarsDice), and optional chain/discovery
configuration.

Blueprint resolution runs first, constructing a game renderer from the best
available source. The startup metadata (source, selection, origin, reason) is
recorded regardless of the chosen tier.

If chain endpoint and subnet are provided, miner discovery connects to the chain
and selects the best available miner. If the miner has a valid endpoint and the
game is poker, a live query attempts an HTTP health check followed by a strategy
query. All three resolution steps (blueprint, discovery, live query) produce
structured metadata for startup reporting.

In train mode, the shell renders the game in TUI mode. If a live miner was
discovered and a poker renderer is active, a background task polls the miner
every 250ms. On successful query, the renderer receives the recommended action
with age 0 (Fresh). If the last success was more than the poll interval ago,
the recommendation is marked Stale with the age in seconds. If no query has
ever succeeded, the advice is marked Offline. Status and Message events are
emitted to the TUI only on connectivity state transitions.

In pipe mode, each input line routes through the renderer's parse/clarify
pipeline. Only Action responses advance game state. Before each STATE output,
the live query is refreshed on-demand to include the latest recommendation. Pipe
output is validated to contain no ANSI escape codes.

In smoke test mode, the binary runs fixed game progressions and outputs a SMOKE
report with all metadata lines. If any --require flag is set and the
corresponding integration point failed, the binary exits with an error.

## Acceptance Criteria

- Train mode renders the TUI shell with the selected game and responds to
  keyboard input.
- Pipe mode produces STATUS, INFO, and STATE lines on startup, then responds
  to each input with ACTION, CLARIFY, ERROR, or QUIT.
- Pipe mode output contains no ANSI escape codes.
- Smoke test mode completes poker progression from PREFLOP through RIVER
  without error.
- Smoke test mode completes Liar's Dice state output and bid demonstration.
- Blueprint resolution falls back gracefully through all four tiers when
  higher-priority sources are unavailable.
- Miner discovery selects the highest-incentive miner with a valid endpoint.
- Live strategy queries health-check the miner before sending a strategy
  request.
- Background live advice refresh tracks Fresh, Stale, and Offline states and
  emits UI updates only on transitions.
- Startup state correctly cascades: Empty advice is terminal, Partial advice or
  discovery/live issues yield Partial.
- Smoke test requirement flags cause exit with error when the corresponding
  integration point is unavailable.
