//! NLHE poker GameRenderer implementation for the myosu TUI.
//!
//! This crate provides the reference `GameRenderer` implementation for
//! No-Limit Hold'em Heads-Up poker. It renders the game state panel,
//! processes action log entries, and formats solver advisor distributions.

pub mod renderer;
pub mod truth_stream;

pub use renderer::NlheRenderer;
