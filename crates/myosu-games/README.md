# myosu-games

Game engine trait abstraction for the myosu game-solving chain.

This crate provides:

- **CFR Trait Re-exports**: Thin wrapper around robopoker's game-agnostic CFR traits
  (`CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Profile`, `Encoder`)

- **Game Configuration**: Typed `GameConfig` with `GameParams` enum for NLHE,
  Liar's Dice, and custom game types

- **Strategy Communication**: `StrategyQuery` and `StrategyResponse` types for
  miner-validator network communication

- **Game Registry**: `GameType` enum for runtime game selection by subnet

## Usage

```rust
use myosu_games::{GameConfig, GameType, StrategyQuery, StrategyResponse};

// Create a NLHE heads-up configuration
let config = GameConfig::nlhe_heads_up(100);

// Parse game type from on-chain bytes
let game_type = GameType::from_bytes(b"nlhe_hu").expect("valid game type");
assert_eq!(game_type.num_players(), 2);
```

## Architecture

This crate is the foundation of myosu's multi-game architecture. Games implement
the CFR traits from `rbp-mccfr`, and this crate adds serialization and runtime
discovery layers needed for the decentralized solver market.
