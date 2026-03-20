//! myosu-play: game session runner with spectator relay.
//!
//! Provides the game session runner and the spectator relay for observing
//! agent-vs-agent gameplay in real-time via Unix domain sockets.

pub mod spectate;

pub use spectate::{discover_local_sessions, GameEvent, SessionInfo, SpectatorRelay};
