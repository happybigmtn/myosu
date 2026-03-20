# Trait Reference

This document describes the core traits that game implementations must satisfy.

## `CfrGame`

The central trait for CFR-based game solving.

```rust
pub trait CfrGame {
    type Action;
    type Info;

    fn cfr_turn(&self) -> CfrTurn;
    fn legal_actions(&self) -> Vec<Self::Action>;
    fn apply_action(&mut self, action: Self::Action);
    fn current_player(&self) -> u8;
    fn terminal_utility(&self, player: u8) -> Utility;
    fn info_sets(&self, player: u8) -> Vec<CfrInfo<Self>>;
    fn encode_info(&self, info: &CfrInfo<Self>, encoder: &mut impl Encoder<Self>);
    fn is_chance_node(&self) -> bool;
    fn chance_outcomes(&self) -> Vec<(Self::Action, Probability)>;
    fn apply_chance(&mut self, action: Self::Action);
}
```

### Required Methods

| Method | Description |
|--------|-------------|
| `cfr_turn()` | Returns `CfrTurn::Player(i)` or `CfrTurn::Chance` |
| `legal_actions()` | Returns valid actions (empty = terminal) |
| `apply_action()` | Mutates state with the given action |
| `current_player()` | Returns acting player index |
| `terminal_utility()` | Returns payoff for player at terminal state |
| `info_sets()` | Returns all information sets for player |
| `encode_info()` | Serializes info set to bytes |
| `is_chance_node()` | True if chance determines outcome |
| `chance_outcomes()` | `(action, probability)` pairs at chance node |
| `apply_chance()` | Applies chance action |

### Important Constraints

1. **Game must be `Copy`** — CFR maintains multiple strategy profiles in memory
2. **Zero-sum** — Sum of utilities across all players must equal zero
3. **Perfect recall** — Information sets must not reveal future information

## `Encoder`

Serializes game actions and information sets.

```rust
pub trait Encoder {
    fn encode_action(&mut self, action: &Self::Action) -> Vec<u8>;
    fn encode_info(&mut self, info: &Self::Info) -> Vec<u8>;
}
```

## `GameRenderer` (TUI only)

Renders game state for terminal display.

```rust
#[cfg(feature = "tui")]
pub trait GameRenderer {
    type Game;

    fn render(&self, game: &Self::Game) -> String;
}
```

## `NPlayerGame`

Extension for games with more than 2 players.

## Types

| Type | Description |
|------|-------------|
| `CfrTurn` | `Player(u8)` or `Chance` |
| `CfrInfo<G>` | Information set with strategy profile |
| `Profile` | Action probability distribution |
| `Probability` | f64 alias for action probabilities |
| `Utility` | f64 alias for game payoffs |
