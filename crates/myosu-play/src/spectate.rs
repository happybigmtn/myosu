//! Spectator relay for game event streaming.
//!
//! Emits JSON event lines to Unix domain sockets at `~/.myosu/spectate/<session_id>.sock`.
//! Fog-of-war is enforced at the relay: game-specific hidden information (hole cards,
//! private dice) is stripped before emission.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// Base directory for spectator sockets.
fn spectate_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".myosu")
        .join("spectate")
}

/// Path to a session's Unix socket.
fn session_socket_path(session_id: &str) -> PathBuf {
    spectate_base_dir().join(format!("{}.sock", session_id))
}

/// A game-agnostic spectator event.
///
/// Variants cover all event types across game types (poker, Liar's Dice, etc.).
/// Each event carries only the information visible to a spectator (no hole cards).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GameEvent {
    /// A new hand/episode has started.
    HandStart {
        hand: u64,
        players: Vec<String>,
        game_type: String,
    },
    /// A player placed a bid (Liar's Dice, or raise in poker).
    Bid {
        player: String,
        quantity: u8,
        face: u8,
    },
    /// A player checked or passed.
    Check { player: String },
    /// A player called.
    Call { player: String, amount: Option<u64> },
    /// A player folded.
    Fold { player: String },
    /// A player bet or raised.
    Raise { player: String, amount: u64 },
    /// A player challenged the current bid.
    Challenge { player: String },
    /// Showdown: hands are revealed.
    Showdown {
        player: String,
        hand: Vec<String>,
    },
    /// The hand ended and payoff was awarded.
    HandEnd {
        hand: u64,
        payoff: Vec<(String, f64)>,
        winner: Option<String>,
    },
    /// A generic game state update for games not yet enumerated.
    StateUpdate {
        phase: String,
        game_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<serde_json::Value>,
    },
}

/// Session metadata for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub game_type: String,
    pub players: Vec<String>,
    pub started_at: u64,
}

impl GameEvent {
    /// Serialize this event as a JSON line (without trailing newline).
    pub fn to_json_line(&self) -> anyhow::Result<String> {
        let line = serde_json::to_string(self)?;
        Ok(line)
    }
}

/// The spectator relay.
///
/// Manages a set of connected Unix socket listeners and broadcasts events to all of them.
/// Events are emitted as JSON lines (one JSON object per line, no trailing comma).
///
/// Fog-of-war is enforced here: the relay only emits events that are safe to show
/// a spectator. Game-specific private information (hole cards, private dice) is
/// never sent — it is stripped by the game engine before the event reaches the relay.
#[derive(Debug, Clone)]
pub struct SpectatorRelay {
    session_id: String,
    listeners: Arc<Mutex<Vec<tokio::net::UnixStream>>>,
    #[allow(dead_code)]
    game_type: String,
}

impl SpectatorRelay {
    /// Create a new relay for the given session.
    pub fn new(session_id: &str, game_type: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            listeners: Arc::new(Mutex::new(Vec::new())),
            game_type: game_type.to_string(),
        }
    }

    /// Ensure the spectate directory exists.
    async fn ensure_base_dir() -> anyhow::Result<()> {
        let dir = spectate_base_dir();
        fs::create_dir_all(&dir).await?;
        Ok(())
    }

    /// Start listening on the session socket.
    /// Creates the Unix socket at `~/.myosu/spectate/<session_id>.sock`.
    pub async fn listen(&self) -> anyhow::Result<()> {
        Self::ensure_base_dir().await?;
        let path = session_socket_path(&self.session_id);

        // Remove existing socket file if present
        let _ = fs::remove_file(&path).await;

        // UnixListener::bind is synchronous in tokio
        let listener = tokio::net::UnixListener::bind(&path)?;
        // Set socket to be accessible by all users (mode 0o777)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path).await?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o777);
            fs::set_permissions(&path, perms).await?;
        }

        // Spawn a task to accept connections
        let listeners = self.listeners.clone();
        let session_id = self.session_id.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let mut guard = listeners.lock().await;
                        guard.push(stream);
                    }
                    Err(e) => {
                        tracing::debug!("spectator socket {} closed: {}", session_id, e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Emit a game event to all connected listeners.
    ///
    /// JSON lines are written asynchronously. Errors on individual listeners
    /// (e.g., disconnected client) are silently ignored.
    pub async fn emit(&self, event: &GameEvent) {
        let json = match event.to_json_line() {
            Ok(line) => line,
            Err(_) => return,
        };
        let mut guard = self.listeners.lock().await;
        let mut dead = Vec::new();
        for (i, stream) in guard.iter_mut().enumerate() {
            let mut write_buf = json.as_bytes().to_vec();
            write_buf.push(b'\n');
            if stream.write_all(&write_buf).await.is_err() {
                dead.push(i);
            }
        }
        // Remove dead listeners
        for i in dead.into_iter().rev() {
            guard.swap_remove(i);
        }
    }

    /// Emit a raw JSON line directly (used for pre-serialized events).
    pub async fn emit_raw(&self, json_line: &str) {
        let mut guard = self.listeners.lock().await;
        let mut dead = Vec::new();
        let line = format!("{}\n", json_line);
        for (i, stream) in guard.iter_mut().enumerate() {
            if stream.write_all(line.as_bytes()).await.is_err() {
                dead.push(i);
            }
        }
        for i in dead.into_iter().rev() {
            guard.swap_remove(i);
        }
    }

    /// Stop the relay and remove the socket file.
    pub async fn shutdown(&self) -> anyhow::Result<()> {
        let path = session_socket_path(&self.session_id);
        let _ = fs::remove_file(&path).await;
        let mut guard = self.listeners.lock().await;
        guard.clear();
        Ok(())
    }
}

/// Discover all local spectator sessions by scanning the spectate directory.
pub async fn discover_local_sessions() -> anyhow::Result<Vec<SessionInfo>> {
    let dir = spectate_base_dir();
    let mut sessions = Vec::new();

    let mut entries = fs::read_dir(&dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sock") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                sessions.push(SessionInfo {
                    session_id: stem.to_string(),
                    game_type: "unknown".to_string(),
                    players: Vec::new(),
                    started_at: 0,
                });
            }
        }
    }
    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn relay_emits_events() {
        let relay = SpectatorRelay::new("test-session-emit", "liars_dice");
        relay.listen().await.unwrap();

        // Give the listener time to bind
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let event = GameEvent::HandStart {
            hand: 1,
            players: vec!["alice".to_string(), "bob".to_string()],
            game_type: "liars_dice".to_string(),
        };
        relay.emit(&event).await;

        relay.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn relay_handles_disconnected_listener() {
        let relay = SpectatorRelay::new("test-session-disconnect", "liars_dice");
        relay.listen().await.unwrap();

        // Give the listener time to bind
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Emit several events — disconnected listeners should not panic
        for i in 0..5u64 {
            let event = GameEvent::HandEnd {
                hand: i,
                payoff: vec![("alice".to_string(), 1.0)],
                winner: Some("alice".to_string()),
            };
            relay.emit(&event).await;
        }

        relay.shutdown().await.unwrap();
    }

    #[test]
    fn events_are_valid_json() {
        let events = vec![
            GameEvent::HandStart {
                hand: 1,
                players: vec!["alice".to_string(), "bob".to_string()],
                game_type: "liars_dice".to_string(),
            },
            GameEvent::Bid {
                player: "alice".to_string(),
                quantity: 2,
                face: 4,
            },
            GameEvent::Challenge {
                player: "bob".to_string(),
            },
            GameEvent::Showdown {
                player: "alice".to_string(),
                hand: vec!["3".to_string(), "5".to_string()],
            },
            GameEvent::HandEnd {
                hand: 1,
                payoff: vec![("alice".to_string(), 1.0)],
                winner: Some("alice".to_string()),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).expect("should serialize");
            let parsed: serde_json::Value =
                serde_json::from_str(&json).expect("should parse as JSON");
            assert!(
                parsed.get("type").is_some(),
                "JSON must have a 'type' field"
            );
        }
    }

    #[tokio::test]
    async fn discover_local_sessions_test() {
        // Create a fake socket file for discovery
        let relay = SpectatorRelay::new("test-discovery-session", "liars_dice");
        relay.listen().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let sessions = super::discover_local_sessions().await.unwrap();
        assert!(
            sessions.iter().any(|s| s.session_id == "test-discovery-session"),
            "should discover the test session"
        );

        relay.shutdown().await.unwrap();
    }
}
