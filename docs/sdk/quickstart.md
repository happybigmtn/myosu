# Quickstart: Add a New Game to myosu in 30 Minutes

This guide walks you through implementing a new game (Kuhn Poker) on myosu using the SDK.

## Prerequisites

- Rust 1.75+
- `cargo` installed
- Access to the myosu repository

## Step 1: Scaffold Your Game Crate

```bash
myosu init --game kuhn-poker
```

This creates `myosu-games-kuhn-poker/` with:

```
myosu-games-kuhn-poker/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── game.rs      # CfrGame impl (fill in)
    ├── encoder.rs   # Encoder impl (fill in)
    ├── renderer.rs  # GameRenderer impl (fill in)
    └── tests.rs     # Compliance tests
```

## Step 2: Implement the Game

### 2a. Define Game State (`src/game.rs`)

Your game state must be `Copy` for CFR to work. Use fixed-size arrays for variable-length data.

```rust
use myosu_sdk::{CfrGame, CfrTurn, CfrInfo, Profile, Encoder, Probability, Utility};

#[derive(Debug, Clone)]
pub struct Game {
    pub player: u8,
    pub deck: [u8; 3],  // Fixed-size: 3 cards
    pub history: u8,     // Bid history as bitfield
}
```

### 2b. Implement `CfrGame` Trait

Fill in each method:

```rust
impl CfrGame for Game {
    type Action = u8;   // 0=fold, 1=bet, 2=call
    type Info = (u8, u8);  // (player, card)

    fn cfr_turn(&self) -> CfrTurn { ... }
    fn legal_actions(&self) -> Vec<Self::Action> { ... }
    fn apply_action(&mut self, action: Self::Action) { ... }
    fn current_player(&self) -> u8 { ... }
    fn terminal_utility(&self, player: u8) -> Utility { ... }
    fn info_sets(&self, player: u8) -> Vec<CfrInfo<Self>> { ... }
    fn encode_info(&self, info: &CfrInfo<Self>, encoder: &mut impl Encoder<Self>) { ... }
    fn is_chance_node(&self) -> bool { ... }
    fn chance_outcomes(&self) -> Vec<(Self::Action, Probability)> { ... }
    fn apply_chance(&mut self, action: Self::Action) { ... }
}
```

### 2c. Implement `Encoder` (`src/encoder.rs`)

```rust
use myosu_sdk::Encoder;

pub struct GameEncoder;

impl Encoder<Game> for GameEncoder {
    fn encode_action(&mut self, action: &u8) -> Vec<u8> {
        vec![*action]
    }

    fn encode_info(&mut self, info: &(u8, u8)) -> Vec<u8> {
        vec![info.0, info.1]
    }
}
```

### 2d. (Optional) Implement `GameRenderer` (`src/renderer.rs`)

```rust
#[cfg(feature = "tui")]
impl GameRenderer for GameRendererImpl {
    type Game = Game;

    fn render(&self, game: &Self::Game) -> String {
        format!("Player {} with card {:?}", game.player, game.deck)
    }
}
```

## Step 3: Run Compliance Tests

```bash
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
```

This validates your implementation satisfies CFR invariants.

## Step 4: Train Your Game

```bash
myosu train --game kuhn-poker --iters 10000
```

## Step 5: Register on Chain

```bash
myosu register \
    --chain ws://localhost:9944 \
    --game-type kuhn-poker \
    --players 2
```

## Next Steps

- See [trait-reference.md](./trait-reference.md) for detailed trait documentation
- See [registration.md](./registration.md) for chain registration details
