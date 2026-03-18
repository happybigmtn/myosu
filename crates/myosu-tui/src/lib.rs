//! Myosu TUI — Terminal user interface for the myosu game-solving platform.
//!
//! This crate provides the visual shell for all myosu games. It implements:
//! - A five-panel layout (header, transcript, state, declaration, input)
//! - Event loop for async updates and keyboard input
//! - Pipe mode for agent protocol (non-interactive)
//! - Game-agnostic rendering traits
//!
//! Games implement the `GameRenderer` trait to draw their state panel.
//! The shell handles all cross-cutting concerns: layout, input, history,
//! logging, and mode switching.

pub mod events;
pub mod input;
pub mod pipe;
pub mod renderer;
pub mod schema;
pub mod screens;
pub mod shell;
pub mod theme;

pub use renderer::{GameRenderer, Renderable};
pub use theme::Theme;
