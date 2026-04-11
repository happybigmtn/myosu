# Specification: Gameplay Surface

Source: myosu-play binary, myosu-tui library crate
Status: Draft
Depends-on: none

## Objective

The `myosu-play` binary is the single entry point for humans and agents to play
games on the myosu network. It provides three modes — smoke test, interactive
TUI training, and line-oriented pipe protocol — unified by the principle that
any action a human can take, an agent can take identically. This spec defines
the expected behavior, protocol contract, and verification criteria for the
gameplay surface.

## Evidence Status

All facts below are verified against source code unless noted otherwise.

**Binary and crate structure**: `myosu-play` lives at `crates/myosu-play/` with
a 1780-line `main.rs`. The TUI framework is the `myosu-tui` library crate.
INV-004 enforces in CI that `myosu-play` has no dependency on `myosu-miner`.

**Three modes verified in code**:

| Mode | Entry | Purpose |
|---|---|---|
| Smoke test | `--smoke-test` | Scripted hand progression through all streets and all games |
| Train | `train` subcommand | Interactive ratatui TUI with solver advisor overlay |
| Pipe | `pipe` subcommand | Line-oriented text protocol for agent consumption |

**Game support**: Poker (NLHE), Kuhn Poker, Liar's Dice, and the
`research/game-rules` portfolio games. Poker, Kuhn, and Liar's Dice use
dedicated renderers; portfolio-routed research games use `PortfolioRenderer`.
Each game implements the `GameRenderer` trait providing TUI rendering within
the five-panel shell.

**Solver advisor**: ON by default in train mode. Background poll at 250ms
interval. Displays action distribution, recommended action, and miner liveness
status (Fresh / Stale / Offline).

**Miner discovery**: When `--chain` and `--subnet` are provided, discovers the
highest-incentive miner on-chain and queries its axon endpoint.

**CLI flags**: `--game`, `--chain`, `--subnet`, `--checkpoint`, `--encoder-dir`,
`--require-artifact`, `--require-discovery`, `--require-live-query`.

**Test coverage**: Pipe responses, smoke reports, discovery flow, live advice,
and portfolio CLI inventory plus accepted-slug alignment with the Rust-owned
research manifest all have existing tests.

## Scope

In scope:
- Smoke test mode: scripted progression for all supported games
- Train mode: TUI screen flow, solver advisor overlay, slash commands
- Pipe mode: structured output protocol, agent-consumable format
- Game renderer trait contract and per-game renderer behavior
- Miner discovery and live query integration
- CLI flag behavior and fail-fast semantics

Out of scope:
- Miner or validator internals (INV-004 boundary)
- Solver training or MCCFR algorithm quality
- Chain runtime or pallet behavior
- Adding new games (see third-game extensibility spec)
- Network transport layer or axon protocol internals

## Current State

The play binary at `crates/myosu-play/src/main.rs` implements all three modes.
The TUI library at `crates/myosu-tui/` provides the ratatui-based rendering
shell with five panels.

Game renderers:
- `NlheRenderer`: Street progression (PREFLOP, FLOP, TURN, RIVER), community
  cards, pot, stacks, action history
- `KuhnRenderer`: Card display, bet/check actions
- `LiarsDiceRenderer`: Dice display, bid/challenge actions
- `PortfolioRenderer`: Research portfolio rule-aware typed challenge
  recommendations with scoped artifact-backed checkpoints for portfolio-routed
  games

TUI screen flow: Loading -> Lobby -> Table -> (hand completes) -> Lobby.
An onboarding screen displays if no artifacts are found.

Train mode supports `/deal` (new hand) and `/quit` slash commands. The solver
advisor overlay shows action distributions and recommended actions with live
miner staleness indicators.

## Behaviors

### Smoke Test

The `--smoke-test` flag runs a scripted progression for each supported game
without user interaction. Poker progresses through PREFLOP -> FLOP -> TURN ->
RIVER -> complete. Kuhn and Liar's Dice complete their respective full game
cycles. The smoke test exits with status 0 on success, non-zero on failure,
and produces a structured smoke report.

### Train Mode (TUI)

The `train` subcommand launches an interactive ratatui terminal session. The
solver advisor is ON by default, polling every 250ms for miner advice. The
overlay displays per-action probability distributions, a recommended action,
and the advisor source (blueprint or live). Miner liveness is shown as Fresh
(recent response), Stale (aged response), or Offline (no response).

Users interact via legal game actions and slash commands (`/deal`, `/quit`).
The TUI renders game state through the `GameRenderer` trait, with each game
providing its own renderer implementation.

### Pipe Mode (Agent Protocol)

The `pipe` subcommand emits structured key=value lines on stdout and reads
actions on stdin. Output message types:

- `STATE`: Game state with street, pot, stacks, legal actions, recommendation.
  Example: `STATE street=PREFLOP pot=3 hero_stack=99 ... actions=fold|call|raise 6|/quit recommend=call advisor=blueprint`
- `ACTION`: Confirmation of applied action
- `CLARIFY`: Request for disambiguation when input is ambiguous
- `ERROR`: Error with context
- `QUIT`: Session termination

No JSON, no binary — plain key=value text. Live query metadata is included when
discovery is active: `live_query=live_http live_miner_advertised_endpoint=...`

The pipe and TUI modes share the same game renderer, enforcing the agent=human
principle: every action available in TUI is available via pipe, with identical
game state transitions.

### Miner Discovery

When `--chain` and `--subnet` are provided, the play binary discovers the
highest-incentive miner on-chain and queries its axon endpoint for live solver
advice. The `--require-discovery` flag makes startup fail if discovery cannot
complete. The `--require-live-query` flag makes startup fail if the discovered
miner does not respond.

### Fail-Fast Flags

- `--require-artifact`: Fail at startup if checkpoint or encoder artifacts are
  not found at the specified paths.
- `--require-discovery`: Fail at startup if on-chain miner discovery fails.
- `--require-live-query`: Fail at startup if the discovered miner endpoint does
  not respond.

These flags convert soft degradation into hard failures for CI and production
environments.

## Acceptance Criteria

- Smoke test passes for the dedicated games (poker, Kuhn, Liar's Dice) and all
  research game slugs via
  `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`.
- Train mode launches, displays solver advisor overlay, and completes at least
  one hand per game without panics.
- Pipe mode emits correctly structured `STATE` lines containing all required
  fields (street, pot, stacks, actions, recommend, advisor) and accepts legal
  actions on stdin.
- Pipe and TUI modes produce identical game state transitions for the same
  action sequence.
- `--require-artifact`, `--require-discovery`, and `--require-live-query` each
  cause a non-zero exit when their respective preconditions are not met.
- Miner discovery resolves the highest-incentive miner and queries its endpoint
  when `--chain` and `--subnet` are provided.
- INV-004 holds: `myosu-play` has no dependency on `myosu-miner` (enforced in
  CI).
- No regressions in existing pipe response, smoke report, discovery, or live
  advice tests.

## Verification

Run the smoke test:

```
SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test
```

Run the existing test suite:

```
SKIP_WASM_BUILD=1 cargo test -p myosu-play
```

Verify the INV-004 dependency boundary:

```
cargo tree -p myosu-play | grep myosu-miner  # expect no output
```

Verify pipe protocol output structure manually or in CI:

```
echo "call" | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe --game poker 2>/dev/null | head -1
# expect: STATE street=PREFLOP pot=... actions=...
```

## Open Questions

- Should the pipe protocol be versioned (e.g., `PROTO_VERSION=1` header) to
  allow future format evolution without breaking agent consumers?
- Is the 250ms advisor poll interval appropriate for production, or should it be
  configurable via CLI flag?
- Should pipe mode support a batch/replay mode where a full action sequence is
  provided upfront for deterministic replay (useful for testing and
  reproducibility)?
- What is the expected behavior when the advisor transitions from Fresh to
  Offline mid-hand — should the TUI fall back to blueprint silently or notify
  the user?
