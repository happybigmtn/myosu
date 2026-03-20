//! `myosu-sdk` — Developer SDK for the myosu game-solving platform
//!
//! This crate provides everything a game developer needs to implement
//! and register a new game on myosu:
//!
//! - **Traits** — re-exports `CfrGame`, `Encoder`, `Profile`, etc. from `myosu-games`
//! - **Test harness** — `testing::assert_game_valid`, `assert_solver_converges`
//! - **Scaffold** — `scaffold::ScaffoldGenerator` for generating new game crates
//! - **Registration** — `register::register_game` for on-chain game registration
//!
//! # Example
//!
//! ```rust
//! use myosu_sdk::CfrGame;
//! ```

pub use myosu_games::{
    CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, GameConfig, GameParams, GameType, Probability,
    Profile, StrategyQuery, StrategyResponse, Utility,
};
pub use rbp_mccfr::{Branch, CfrPublic, CfrSecret, Node, Tree};
pub use rbp_transport::Support;

// Test harness (AC-SDK-03)
pub mod testing;

// Scaffold tool (AC-SDK-02)
pub mod scaffold;

// Registration CLI (AC-SDK-04)
pub mod register;

#[cfg(feature = "tui")]
pub use myosu_tui::GameRenderer;

#[cfg(feature = "tui")]
pub use ratatui::{buffer::Buffer, layout::Rect};
