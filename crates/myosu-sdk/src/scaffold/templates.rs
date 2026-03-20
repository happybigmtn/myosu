//! Project templates for scaffold-generated game crates.

/// Generate the Cargo.toml content for a new game crate.
pub fn cargo_toml(crate_name: &str) -> String {
    format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Game engine implementation for myosu"

[dependencies]
myosu-sdk = {{ path = "../../crates/myosu-sdk", features = ["tui"] }}

[dev-dependencies]
tokio-test = "0.4"
"#,
        crate_name = crate_name
    )
}

/// Generate the src/lib.rs content.
pub fn lib_rs(crate_name: &str) -> String {
    format!(
        r#"//! {crate_name}: Game engine implementation for myosu
//!
//! This crate implements the `CfrGame` trait for the game.

pub mod game;
pub mod encoder;
pub mod renderer;

pub use game::Game;
"#,
        crate_name = crate_name
    )
}

/// Generate the src/game.rs content with todo!() stubs.
pub fn game_rs(game_name: &str) -> String {
    format!(
        r#"//! Game state and CFR implementation.
//!
//! Fill in the `CfrGame` implementation for your game.
//! The `{{todo!("implement {{game_name}} game logic")}}` stubs
//! guide you through the required methods.

use myosu_sdk::{{CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder, Probability, Utility}};

/// Game state for {game_name}
///
/// Your game state must implement `Copy` — use fixed-size arrays
/// for variable-length data (bid history, card sequences).
#[derive(Debug, Clone)]
pub struct Game {{
    // TODO: Add your game state fields here.
    // IMPORTANT: Game must be Copy for CFR to work.
    // Use fixed-size arrays with sentinel values for variable-length data.
}}

impl CfrGame for Game {{
    type Action = (); // TODO: Define your action type
    type Info = (); // TODO: Define your information set type

    fn cfr_turn(&self) -> CfrTurn {{
        todo!("implement CFR turn determination")
    }}

    fn legal_actions(&self) -> Vec<Self::Action> {{
        todo!("return legal actions from this state")
    }}

    fn apply_action(&mut self, action: Self::Action) {{
        todo!("apply the given action to advance game state")
    }}

    fn current_player(&self) -> u8 {{
        todo!("return the acting player index (0, 1, ...)")
    }}

    fn terminal_utility(&self, player: u8) -> Utility {{
        todo!("return utility for `player` at terminal state")
    }}

    fn info_sets(&self, player: u8) -> Vec<CfrInfo<Self>> {{
        todo!("return information sets for the given player")
    }}

    fn encode_info(&self, info: &CfrInfo<Self>, encoder: &mut impl Encoder<Self>) {{
        todo!("encode the information set using the encoder")
    }}

    fn is_chance_node(&self) -> bool {{
        todo!("return true if this is a chance node")
    }}

    fn chance_outcomes(&self) -> Vec<(Self::Action, Probability)> {{
        todo!("return (action, probability) pairs for chance node")
    }}

    fn apply_chance(&mut self, action: Self::Action) {{
        todo!("apply the chance action to advance game state")
    }}
}}
"#
    )
}

/// Generate the src/encoder.rs content.
pub fn encoder_rs() -> String {
    r#"//! Encoder implementation for game information sets.
//!
//! The encoder serializes game information sets for storage
//! and retrieval of CFR profiles.

use myosu_sdk::Encoder;

/// Encoder for game information sets.
#[derive(Debug)]
pub struct GameEncoder;

impl Encoder<super::Game> for GameEncoder {
    fn encode_action(&mut self, action: &<super::Game as myosu_sdk::CfrGame>::Action) -> Vec<u8> {
        todo!("encode the given action")
    }

    fn encode_info(&mut self, info: &<super::Game as myosu_sdk::CfrGame>::Info) -> Vec<u8> {
        todo!("encode the information set")
    }
}
"#
    .to_string()
}

/// Generate the src/renderer.rs content.
pub fn renderer_rs() -> String {
    r#"//! TUI renderer for the game.
//!
//! This module provides `GameRenderer` implementation for displaying
//! the game state in the terminal. Only compiled when the `tui` feature
//! is enabled.

#[cfg(feature = "tui")]
use myosu_sdk::myosu_tui::GameRenderer;

#[cfg(feature = "tui")]
/// Renderer for displaying game state in the terminal.
pub struct GameRendererImpl;

#[cfg(feature = "tui")]
impl GameRenderer for GameRendererImpl {
    type Game = super::Game;

    fn render(&self, game: &Self::Game) -> String {
        todo!("render game state to string")
    }
}
"#
    .to_string()
}

/// Generate the src/tests.rs content with pre-written compliance tests.
pub fn tests_rs() -> String {
    r#"//! Trait compliance tests for the game.
//!
//! These tests validate that the game implementation satisfies
//! CFR invariants. They fail until you implement the game.

use myosu_sdk::testing::assert_game_valid;
use crate::Game;

#[test]
fn game_passes_all_compliance_checks() {
    // This test will pass once the game is fully implemented.
    // For now it serves as a reminder of what needs to be done.
    todo!("implement game and uncomment the assertion");
    // assert_game_valid::<Game>();
}
"#
    .to_string()
}

/// Generate the README.md content.
pub fn readme_md(game_name: &str) -> String {
    format!(
        r#"# {game_name} — myosu Game Engine

Game engine implementation for [myosu](https://github.com/happybigmtn/myosu).

## Status

🚧 **Implementing** — This crate is a scaffold generated by `myosu init --game {game_name}`.

## Implementation Guide

1. **Implement `Game`** in `src/game.rs` — the game state and CFR logic
2. **Implement `Encoder`** in `src/encoder.rs` — information set serialization
3. **Implement `GameRenderer`** in `src/renderer.rs` — TUI rendering (optional, requires `tui` feature)
4. **Run tests** with `cargo test`

## Compliance

Once implemented, run the compliance harness:

```bash
cargo test -p myosu-sdk testing::assert_game_valid
```

This validates that your game satisfies all CFR invariants.
"#
    )
}
