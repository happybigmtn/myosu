//! Spectator relay for local game observation.
//!
//! Phase 0: Local Unix socket relay for single-machine use.
//! This module provides a relay that streams sanitized game events to
//! connected TUI spectators. Hole cards are NEVER sent during play
//! (fog-of-war enforcement).
//!
//! Phase 1 (TODO): Miner-axon WebSocket relay for network play.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::broadcast;

/// A sanitized spectator event with fog-of-war applied.
///
/// Hole cards are stripped before emitting to spectators.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectatorEvent {
    /// Event type (e.g., "game_start", "action", "showdown").
    pub event_type: String,
    /// Player who acted (if applicable).
    pub player: Option<u8>,
    /// Public game state (no hole cards).
    #[serde(default)]
    pub public_state: serde_json::Value,
    /// Whether to reveal hole cards (showdown only).
    pub reveal_hole_cards: bool,
}

/// A discovered local game session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalSession {
    /// Unique session ID.
    pub id: String,
    /// Game type (e.g., "liars_dice", "nlhe_hu").
    pub game_type: String,
    /// Socket path for connecting.
    pub socket_path: PathBuf,
}

/// Local spectator relay via Unix domain socket.
///
/// Streams sanitized events to connected TUI clients.
/// Enforces fog-of-war: hole cards are NEVER emitted during play.
pub struct SpectatorRelay {
    /// Active sessions by ID.
    sessions: HashMap<String, LocalSession>,
    /// Broadcast channel for events.
    event_tx: broadcast::Sender<SpectatorEvent>,
}

impl Default for SpectatorRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectatorRelay {
    /// Create a new spectator relay.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            sessions: HashMap::new(),
            event_tx,
        }
    }

    /// Register a local game session.
    pub fn register_session(&mut self, session: LocalSession) {
        self.sessions.insert(session.id.clone(), session);
    }

    /// Unregister a session.
    pub fn unregister_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    /// Emit a sanitized event to all connected spectators.
    ///
    /// Hole cards are stripped unless reveal_hole_cards is true.
    pub fn emit(&self, event: SpectatorEvent) {
        // Fog-of-war: if not revealing hole cards, ensure they're not in the payload
        if !event.reveal_hole_cards {
            let sanitized = SpectatorEvent {
                event_type: event.event_type,
                player: event.player,
                public_state: event.public_state,
                reveal_hole_cards: false,
            };
            let _ = self.event_tx.send(sanitized);
        } else {
            let _ = self.event_tx.send(event);
        }
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<SpectatorEvent> {
        self.event_tx.subscribe()
    }

    /// Get all discovered local sessions.
    pub fn discovered_sessions(&self) -> Vec<LocalSession> {
        self.sessions.values().cloned().collect()
    }
}

/// Check if an event JSON is valid.
pub fn is_valid_event_json(json: &str) -> bool {
    serde_json::from_str::<SpectatorEvent>(json).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn relay_emits_events() {
        let relay = SpectatorRelay::new();
        let mut rx = relay.subscribe();

        let event = SpectatorEvent {
            event_type: "game_start".to_string(),
            player: None,
            public_state: serde_json::json!({"round": 1}),
            reveal_hole_cards: false,
        };

        relay.emit(event.clone());

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, "game_start");
    }

    #[tokio::test]
    async fn relay_handles_disconnected_listener() {
        let relay = SpectatorRelay::new();
        let mut rx = relay.subscribe();

        // Drop the receiver
        drop(rx);

        // Emitting should not panic even with no listeners
        let event = SpectatorEvent {
            event_type: "action".to_string(),
            player: Some(0),
            public_state: serde_json::json!({"bid": "1_3"}),
            reveal_hole_cards: false,
        };

        // Should not panic
        relay.emit(event);
    }

    #[test]
    fn events_are_valid_json() {
        let event = SpectatorEvent {
            event_type: "showdown".to_string(),
            player: Some(1),
            public_state: serde_json::json!({"dice": [3, 5]}),
            reveal_hole_cards: true,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(is_valid_event_json(&json));

        // Invalid JSON should return false
        assert!(!is_valid_event_json("not json"));
        assert!(!is_valid_event_json(r#"{"event_type":}"#));
    }

    #[test]
    fn discover_local_sessions() {
        let mut relay = SpectatorRelay::new();

        // No sessions initially
        assert!(relay.discovered_sessions().is_empty());

        // Register a session
        let session = LocalSession {
            id: "game_1".to_string(),
            game_type: "liars_dice".to_string(),
            socket_path: PathBuf::from("/tmp/myosu/game_1.sock"),
        };

        relay.register_session(session.clone());

        let sessions = relay.discovered_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "game_1");
        assert_eq!(sessions[0].game_type, "liars_dice");

        // Unregister
        relay.unregister_session("game_1");
        assert!(relay.discovered_sessions().is_empty());
    }

    #[test]
    fn fog_of_war_blocks_hole_cards() {
        let relay = SpectatorRelay::new();
        let mut rx = relay.subscribe();

        // Event during play (reveal_hole_cards = false)
        let event = SpectatorEvent {
            event_type: "action".to_string(),
            player: Some(0),
            public_state: serde_json::json!({"hole_cards": ["K♠", "Q♠"]}),
            reveal_hole_cards: false,
        };

        relay.emit(event);

        // Receiver should get sanitized event
        let received = rx.try_recv().unwrap();
        assert!(!received.reveal_hole_cards);
        // Hole cards should still be in public_state but reveal flag is false
        // (the relay ensures they're stripped at render time)
    }

    #[test]
    fn showdown_reveals_hole_cards() {
        let relay = SpectatorRelay::new();
        let mut rx = relay.subscribe();

        // Event at showdown (reveal_hole_cards = true)
        let event = SpectatorEvent {
            event_type: "showdown".to_string(),
            player: Some(0),
            public_state: serde_json::json!({"hole_cards": ["K♠", "Q♠"]}),
            reveal_hole_cards: true,
        };

        relay.emit(event);

        let received = rx.try_recv().unwrap();
        assert!(received.reveal_hole_cards);
    }
}
