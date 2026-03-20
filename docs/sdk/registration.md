# Game Registration Guide

This document describes how to register a game on the myosu chain.

## Overview

Game registration creates a new game type on-chain, enabling miners to serve strategies and validators to score them.

## Prerequisites

- A running myosu chain node (or access to a public endpoint)
- Your game crate implemented and tested locally

## Registration Flow

### 1. Validate Your Game

Before registering, ensure your game passes compliance tests:

```bash
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
```

### 2. Connect to Chain

Test your chain connection:

```bash
# Using websocat or similar
websocat ws://localhost:9944
```

### 3. Register the Game

```bash
myosu register \
    --chain ws://localhost:9944 \
    --game-type kuhn-poker \
    --players 2 \
    --exploit-unit "exploit" \
    --exploit-baseline 1.0
```

### Registration Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--chain` | WebSocket endpoint | required |
| `--game-type` | Unique game identifier | required |
| `--players` | Number of players | 2 |
| `--exploit-unit` | Exploitability measurement unit | "exploit" |
| `--exploit-baseline` | Baseline exploitability | 1.0 |

## On-Chain Process

The registration extrinsic (`register_game_type`) performs:

1. **Validation** — Verifies game type doesn't already exist
2. **Subnet creation** — Creates a new subnet for the game
3. **Initial parameters** — Sets exploit baseline and unit

## Post-Registration

After registration:

1. **Miners** can serve strategies for the new game type
2. **Validators** can score miner strategies using exploitability
3. **Players** can request strategy recommendations

## Troubleshooting

### Connection Timeout

```
Error: connection timeout: failed to connect to ws://localhost:9944 within 5s
```

- Check the chain node is running
- Verify the WebSocket URL is correct
- Check firewall rules

### Registration Failed

```
Error: registration failed: game type already exists
```

The game type is already registered. Choose a different name or check existing registrations.

### Chain Error

```
Error: chain error: consensus error
```

The chain may be in a degraded state. Check chain logs and try again.

## Integration with Services

The registration creates on-chain state that:

- `services:miner` uses to discover game types to serve
- `services:validator-oracle` uses to score strategies
