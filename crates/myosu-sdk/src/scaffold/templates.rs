//! Project templates for scaffold-generated game crates.

/// Generate the Cargo.toml content for a new game crate.
pub fn cargo_toml(crate_name: &str, sdk_dependency_path: &str) -> String {
    format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"
license = "MIT"
description = "Game engine implementation for myosu"

[features]
default = []
tui = ["myosu-sdk/tui"]

[dependencies]
myosu-sdk = {{ path = "{sdk_dependency_path}", default-features = false }}
"#,
    )
}

/// Generate the src/lib.rs content.
pub fn lib_rs(game_name: &str) -> String {
    format!(
        r#"//! `myosu-games-{game_name}`: game engine implementation scaffold for myosu.
//!
//! Replace the stub types in `game.rs`, `encoder.rs`, and `renderer.rs`
//! with your concrete game implementation.

pub mod game;
pub mod encoder;

#[cfg(feature = "tui")]
pub mod renderer;

#[cfg(test)]
mod tests;

pub use encoder::GameEncoder;
pub use game::{{Game, GameAction, GameInfo, GamePublicInfo, GameSecretInfo, GameTurn}};

#[cfg(feature = "tui")]
pub use renderer::GameRendererImpl;
"#,
    )
}

/// Generate the src/game.rs content with compileable stub types.
pub fn game_rs(game_name: &str) -> String {
    format!(
        r#"//! Game state and CFR implementation scaffold for `{game_name}`.
//!
//! The template intentionally uses fixed-size arrays plus a length field so
//! the game state stays `Copy`. For games with variable-length histories,
//! keep this pattern and choose a capacity that fits your rules.

use myosu_sdk::{{
    CfrEdge, CfrGame, CfrInfo, CfrPublic, CfrSecret, CfrTurn, Support, Utility,
}};

const MAX_HISTORY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameAction {{
    Placeholder,
}}

impl Support for GameAction {{}}
impl CfrEdge for GameAction {{}}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameTurn {{
    Chance,
    PlayerOne,
    PlayerTwo,
    Terminal,
}}

impl From<usize> for GameTurn {{
    fn from(player: usize) -> Self {{
        match player {{
            0 => Self::PlayerOne,
            1 => Self::PlayerTwo,
            _ => panic!("scaffold only models two players by default"),
        }}
    }}
}}

impl Support for GameTurn {{}}

impl CfrTurn for GameTurn {{
    fn chance() -> Self {{
        Self::Chance
    }}

    fn terminal() -> Self {{
        Self::Terminal
    }}
}}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GamePublicInfo {{
    pub turn: GameTurn,
    pub history: [Option<GameAction>; MAX_HISTORY],
    pub history_len: u8,
}}

impl CfrPublic for GamePublicInfo {{
    type E = GameAction;
    type T = GameTurn;

    fn choices(&self) -> Vec<Self::E> {{
        todo!("define the legal actions visible from this public state")
    }}

    fn history(&self) -> Vec<Self::E> {{
        self.history
            .into_iter()
            .take(self.history_len as usize)
            .flatten()
            .collect()
    }}
}}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameSecretInfo {{
    pub bucket: u8,
}}

impl Support for GameSecretInfo {{}}
impl CfrSecret for GameSecretInfo {{}}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameInfo {{
    pub public: GamePublicInfo,
    pub secret: GameSecretInfo,
}}

impl CfrInfo for GameInfo {{
    type E = GameAction;
    type T = GameTurn;
    type X = GamePublicInfo;
    type Y = GameSecretInfo;

    fn public(&self) -> Self::X {{
        self.public
    }}

    fn secret(&self) -> Self::Y {{
        self.secret
    }}
}}

/// Game state for `{game_name}`.
///
/// Keep the state compact and `Copy`. If you need variable-length data
/// such as a bid history, prefer fixed-size arrays with sentinel slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Game {{
    pub turn: GameTurn,
    pub history: [Option<GameAction>; MAX_HISTORY],
    pub history_len: u8,
}}

impl CfrGame for Game {{
    type E = GameAction;
    type T = GameTurn;

    fn root() -> Self {{
        Self {{
            turn: GameTurn::Chance,
            history: [None; MAX_HISTORY],
            history_len: 0,
        }}
    }}

    fn turn(&self) -> Self::T {{
        self.turn
    }}

    fn apply(&self, edge: Self::E) -> Self {{
        let _ = edge;
        todo!("advance the game state and append the chosen action")
    }}

    fn payoff(&self, turn: Self::T) -> Utility {{
        let _ = turn;
        todo!("return terminal utility for the requested player")
    }}
}}
"#,
    )
}

/// Generate the src/encoder.rs content.
pub fn encoder_rs(game_name: &str) -> String {
    format!(
        r#"//! Encoder implementation scaffold for `{game_name}`.
//!
//! The encoder maps game states onto information sets. For simple games this
//! can be direct; for larger games it usually encodes abstractions.

use myosu_sdk::{{Branch, Encoder, Tree}};

use crate::{{Game, GameAction, GameInfo, GameTurn}};

#[derive(Debug, Clone, Copy, Default)]
pub struct GameEncoder;

impl Encoder for GameEncoder {{
    type T = GameTurn;
    type E = GameAction;
    type G = Game;
    type I = GameInfo;

    fn seed(&self, game: &Self::G) -> Self::I {{
        let _ = game;
        todo!("build the root information set")
    }}

    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: Branch<Self::E, Self::G>,
    ) -> Self::I {{
        let _ = (tree, leaf);
        todo!("derive an information set for a child branch")
    }}

    fn resume(&self, past: &[Self::E], game: &Self::G) -> Self::I {{
        let _ = (past, game);
        todo!("rebuild an information set from a path and game state")
    }}
}}
"#,
    )
}

/// Generate the src/renderer.rs content.
pub fn renderer_rs(game_name: &str) -> String {
    format!(
        r#"//! TUI renderer scaffold for `{game_name}`.
//!
//! This module is only compiled when the generated crate enables its `tui`
//! feature, which forwards to `myosu-sdk/tui`.

use myosu_sdk::{{Buffer, GameRenderer, Rect}};

#[derive(Debug, Clone, Copy, Default)]
pub struct GameRendererImpl;

impl GameRenderer for GameRendererImpl {{
    fn render_state(&self, area: Rect, buf: &mut Buffer) {{
        let _ = (area, buf);
        todo!("draw the game-specific state panel")
    }}

    fn desired_height(&self, width: u16) -> u16 {{
        let _ = width;
        8
    }}

    fn declaration(&self) -> &str {{
        "IMPLEMENT YOUR GAME DECLARATION"
    }}

    fn completions(&self) -> Vec<String> {{
        vec![]
    }}

    fn parse_input(&self, input: &str) -> Option<String> {{
        let _ = input;
        None
    }}

    fn clarify(&self, input: &str) -> Option<String> {{
        let _ = input;
        None
    }}

    fn pipe_output(&self) -> String {{
        todo!("render a pipe-friendly view of the game state")
    }}

    fn game_label(&self) -> &str {{
        "{game_name}"
    }}

    fn context_label(&self) -> &str {{
        "SETUP"
    }}
}}
"#,
    )
}

/// Generate the src/tests.rs content with a pre-written failing compliance test.
pub fn tests_rs(game_name: &str) -> String {
    format!(
        r#"//! Compliance tests for the `{game_name}` scaffold.
//!
//! This test is expected to fail until you replace the `todo!()` with a real
//! call to the SDK harness after implementing your game.

use crate::Game;

#[test]
fn game_passes_all_compliance_checks() {{
    let _game_type = std::any::type_name::<Game>();
    // Replace this todo with:
    // myosu_sdk::testing::assert_game_valid::<Game>();
    todo!("implement `{game_name}` and replace this todo with a real compliance assertion");
}}
"#,
    )
}

/// Generate the README.md content.
pub fn readme_md(game_name: &str) -> String {
    format!(
        r#"# myosu-games-{game_name}

Scaffolded game engine crate for the myosu platform.

## What to Fill In

1. Implement `Game`, `GameAction`, `GameTurn`, and the information-set types in `src/game.rs`.
2. Implement `GameEncoder` in `src/encoder.rs`.
3. Replace the scaffolded compliance-test `todo!()` in `src/tests.rs` with `assert_game_valid::<Game>();`.
4. If you want TUI support, enable `--features tui` and implement `GameRendererImpl` in `src/renderer.rs`.

## Commands

```bash
cargo check
cargo test
cargo check --features tui
```

## Copy Constraint Reminder

`CfrGame` is `Copy`, so store variable-length data in fixed-size arrays plus
an explicit length field instead of heap-backed containers like `Vec`.
"#,
    )
}
