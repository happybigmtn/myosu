# Myosu Design

Date: 2026-04-05

This document describes the user-facing surfaces of myosu: the gameplay
interface, the operator CLI surfaces, and the agent-native text protocol.

---

## User-Facing Surfaces

Myosu has four user-facing surfaces:

1. **TUI gameplay** -- interactive terminal poker/game session with solver advisor
2. **Pipe protocol** -- line-oriented text protocol for agent consumption
3. **Operator CLI** -- key management, node operation, miner/validator setup
4. **Chain RPC** -- WebSocket JSON-RPC for programmatic chain interaction

---

## 1. TUI Gameplay (myosu-play train)

### Screen Flow

```
Launch → Loading → Lobby → Table → (hand completes) → Lobby
                     ↓
              Onboarding (if no artifacts found)
```

### Screens

| Screen | Content | Interaction |
|--------|---------|-------------|
| Loading | Spinner + status message | Automatic transition |
| Onboarding | First-run guidance | Read-only |
| Lobby | Session info, log messages | Type `1` to enter table |
| Table | Game state + solver advisor | Legal actions, `/quit`, `/deal` |

### Solver Advisor Overlay

When artifact-backed strategy is available, the TUI shows:
- Action distribution for the current information set
- Recommended action (highest probability)
- Live miner advice with staleness indicator (Fresh/Stale/Offline)

The advisor is ON by default in train mode. This is the core value proposition:
learning GTO through play with transparent strategy guidance.

### Game Renderers

Each game implements `GameRenderer`:
- **Poker (NlheRenderer)**: Street progression (PREFLOP→FLOP→TURN→RIVER),
  community cards, pot size, player stacks, action history, bet sizing
- **Kuhn (KuhnRenderer)**: Card display, bet/check actions
- **Liar's Dice (LiarsDiceRenderer)**: Dice display, bid/challenge actions

### Live Advice Refresh

When chain discovery finds a miner:
- Background task polls miner axon every 250ms
- Displays connectivity state: Fresh (green), Stale (yellow), Offline (red)
- Shows age of last successful refresh
- Emits `UpdateEvent::Status` and `UpdateEvent::Message` on transitions

---

## 2. Pipe Protocol (myosu-play pipe)

### Message Types

| Direction | Prefix | Meaning |
|-----------|--------|---------|
| Output | `STATUS` | Startup status with metadata |
| Output | `STATE` | Current game state |
| Output | `ACTION` | Accepted action |
| Output | `CLARIFY` | Ambiguous input with legal actions |
| Output | `ERROR` | Invalid input with legal actions |
| Output | `QUIT` | Session ended |
| Input | action text | e.g. `fold`, `call`, `raise 6`, `bet`, `bid 1x4` |
| Input | `/quit` or `quit` | End session |

### State Output Format

Poker example:
```
STATE street=PREFLOP pot=3 hero_stack=99 villain_stack=98 actions=fold|call|raise 6|/quit recommend=call advisor=blueprint
```

Kuhn example:
```
STATE game=kuhn_poker hero_card=K phase=betting actions=bet|check|/quit
```

Liar's Dice example:
```
STATE game=liars_dice hero_dice=2,5 total_dice=4 actions=bid 1x4|bid 1x5|challenge|/quit
```

### Live Query Metadata

When chain discovery is active, state output includes:
```
live_query=live_http live_miner_advertised_endpoint=0.0.0.0:8080 live_miner_connect_endpoint=127.0.0.1:8080 live_miner_action_count=3 live_miner_recommended_edge=Call live_miner_recommended_action=call
```

### Agent Integration Pattern

An agent consumes myosu-play pipe mode by:
1. Launching: `printf 'quit\n' | cargo run -p myosu-play -- pipe`
2. Parsing `STATE` lines for game state
3. Sending action text on stdin
4. Parsing `ACTION` responses to confirm execution
5. Using `recommend=` field for solver-backed advice
6. Sending `quit` to cleanly exit

---

## 3. Operator CLI Surfaces

### myosu-keys

Key management for operator identity:

| Command | Purpose |
|---------|---------|
| `create` | Generate new keypair |
| `show-active` | Display active key address |
| `list` | List all keys in keystore |
| `switch-active` | Change active key |
| `import-keyfile` | Import from JSON keyfile |
| `import-mnemonic` | Import from 12-word mnemonic |
| `import-raw-seed` | Import from raw seed hex |
| `export-active-keyfile` | Export active key to JSON |
| `print-bootstrap` | Print bootstrap metadata for subnet |
| `change-password` | Change keystore password |

All password inputs via environment variables (`MYOSU_KEY_PASSWORD`), never
command-line arguments.

### myosu-miner

| Flag | Purpose |
|------|---------|
| `--chain` | RPC endpoint |
| `--subnet` | Target subnet ID |
| `--key` / `--key-config-dir` | Identity source |
| `--register` | Register neuron on-chain |
| `--serve-axon` | Publish axon endpoint |
| `--port` | HTTP axon port |
| `--data-dir` | Training artifact storage |
| `--iterations` | MCCFR training iterations |
| `--checkpoint` | Pre-trained checkpoint path |
| `--game` | Game type (poker, liars-dice) |

### myosu-validator

| Flag | Purpose |
|------|---------|
| `--chain` | RPC endpoint |
| `--subnet` | Target subnet ID |
| `--key` / `--key-config-dir` | Identity source |
| `--register` | Register validator on-chain |
| `--stake-amount` | Amount to stake |
| `--submit-weights` | Submit weights after scoring |
| `--encoder-dir` | Encoder artifact directory |
| `--checkpoint` | Solver checkpoint path |
| `--query-file` / `--response-file` | Wire-encoded artifacts to score |
| `--game` | Game type (poker, liars-dice) |

### myosu-play

| Flag | Purpose |
|------|---------|
| `--smoke-test` | Run scripted proof |
| `--game` | Game selection (poker, kuhn, liars-dice) |
| `--chain` / `--subnet` | Enable miner discovery |
| `--checkpoint` / `--encoder-dir` | Artifact-backed strategy |
| `--require-artifact` | Fail if no artifacts |
| `--require-discovery` | Fail if no chain miner |
| `--require-live-query` | Fail if miner unreachable |
| `train` | Interactive TUI mode |
| `pipe` | Agent-native text protocol |

---

## 4. Chain RPC Surface

Stage-0 chain exposes standard Substrate JSON-RPC plus game-solver custom RPCs:

| Method | Purpose |
|--------|---------|
| `gameSolver_getSubnetInfo` | Subnet metadata |
| `gameSolver_getNeuronInfo` | Neuron state for a subnet |
| `gameSolver_getWeights` | Current weight matrix |
| `gameSolver_getStakeInfo` | Staking state |

The shared `myosu-chain-client` crate wraps these with typed helpers:
- `probe_chain()`: Connectivity and subnet state check
- `ensure_registered()`: Register neuron if not already registered
- `ensure_serving()`: Publish axon endpoint
- `submit_weights()`: Commit-reveal weight submission

---

## Design Principles

1. **Agent = Human**: The pipe protocol and TUI share the same game renderer.
   Any action a human can take in the TUI, an agent can take via pipe.

2. **Solver advisor ON by default**: Training mode shows action distributions
   because learning GTO is the value proposition. Advisor is OFF by default
   in chain/miner mode (miner privacy).

3. **Structured output**: Pipe mode uses key=value structured output parseable
   by simple text tools. No JSON wrapping, no binary protocols for the
   agent surface.

4. **Progressive disclosure**: Smoke test → pipe mode → TUI. Each surface
   is independently useful and independently testable.

5. **Multi-game through composition**: New games plug in via GameRenderer +
   CFR traits. The play surface, miner, and validator are game-agnostic at
   the orchestration layer.
