//! Myosu Play — Gameplay CLI and spectator relay.
//!
//! This crate provides:
//! - Local Unix socket relay for spectator mode (Phase 0)
//! - Event streaming to connected TUI spectators
//!
//! Phase 0: Local relay via Unix domain socket.
//! Phase 1 (TODO): Miner-axon WebSocket relay for network play.

pub mod spectate;

pub use spectate::{LocalSession, SpectatorRelay, SpectatorEvent};
