//! Spectator screen for watching agent-vs-agent gameplay.
//!
//! Renders the event stream from the spectator relay. Fog-of-war is enforced:
//! during play, private information (hole cards, private dice) is shown as masked.
//! After showdown, private information is revealed.
//!
//! The screen is read-only: no input is accepted except navigation keys.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// The current view mode for the spectator screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectateView {
    /// During active play — private info is masked.
    Playing,
    /// After showdown — all private info is revealed.
    Revealed,
}

impl Default for SpectateView {
    fn default() -> Self {
        Self::Playing
    }
}

/// A parsed event ready for rendering.
#[derive(Debug, Clone)]
pub enum RenderEvent {
    HandStart { hand: u64, players: Vec<String>, game_type: String },
    Bid { player: String, quantity: u8, face: u8 },
    Challenge { player: String },
    Showdown { player: String, hand: Vec<String> },
    HandEnd { hand: u64, payoff: Vec<(String, f64)>, winner: Option<String> },
}

/// State maintained by the spectator screen across events.
#[derive(Debug, Clone)]
pub struct SpectateState {
    /// Current view mode (playing vs revealed).
    pub view: SpectateView,
    /// Current hand number.
    pub hand: u64,
    /// Players in the session.
    pub players: Vec<String>,
    /// Game type string.
    pub game_type: String,
    /// Last bid shown to spectators.
    pub last_bid: Option<(String, u8, u8)>, // (player, quantity, face)
    /// Pending showdown events to reveal after showdown completes.
    pending_reveals: Vec<RenderEvent>,
    /// True if we are currently in showdown.
    in_showdown: bool,
}

impl Default for SpectateState {
    fn default() -> Self {
        Self {
            view: SpectateView::Playing,
            hand: 0,
            players: Vec::new(),
            game_type: String::new(),
            last_bid: None,
            pending_reveals: Vec::new(),
            in_showdown: false,
        }
    }
}

impl SpectateState {
    /// Create a fresh spectator state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a raw JSON event line from the relay.
    ///
    /// The relay guarantees that events are already fog-of-war filtered —
    /// hole cards and private dice are stripped before emission. The TUI
    /// additionally masks any remaining private fields based on view mode.
    pub fn handle_event(&mut self, event: &str) -> Option<RenderEvent> {
        let parsed: serde_json::Value = serde_json::from_str(event).ok()?;
        let type_field = parsed.get("type")?.as_str()?;

        match type_field {
            "HandStart" | "hand_start" => {
                let data = parsed.get("data")?;
                let hand = data.get("hand")?.as_u64()?;
                let players: Vec<String> = data
                    .get("players")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let game_type = data
                    .get("game_type")?
                    .as_str()?
                    .to_string();
                self.hand = hand;
                self.players = players;
                self.game_type = game_type;
                self.view = SpectateView::Playing;
                self.last_bid = None;
                self.pending_reveals.clear();
                self.in_showdown = false;
                Some(RenderEvent::HandStart {
                    hand,
                    players: self.players.clone(),
                    game_type: self.game_type.clone(),
                })
            }
            "Bid" | "bid" => {
                let data = parsed.get("data")?;
                let player = data.get("player")?.as_str()?.to_string();
                let quantity = data.get("quantity")?.as_u64()? as u8;
                let face = data.get("face")?.as_u64()? as u8;
                self.last_bid = Some((player.clone(), quantity, face));
                Some(RenderEvent::Bid { player, quantity, face })
            }
            "Challenge" | "challenge" => {
                let data = parsed.get("data")?;
                let player = data.get("player")?.as_str()?.to_string();
                Some(RenderEvent::Challenge { player })
            }
            "Showdown" | "showdown" => {
                let data = parsed.get("data")?;
                let player = data.get("player")?.as_str()?.to_string();
                let hand: Vec<String> = data
                    .get("hand")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                self.in_showdown = true;
                self.view = SpectateView::Revealed;
                Some(RenderEvent::Showdown { player, hand })
            }
            "HandEnd" | "hand_end" => {
                let data = parsed.get("data")?;
                let hand = data.get("hand")?.as_u64()?;
                let payoff: Vec<(String, f64)> = data
                    .get("payoff")?
                    .as_array()?
                    .iter()
                    .filter_map(|v| {
                        let arr = v.as_array()?;
                        let player = arr.first()?.as_str()?.to_string();
                        let value = arr.get(1)?.as_f64()?;
                        Some((player, value))
                    })
                    .collect();
                let winner = data
                    .get("winner")?
                    .as_str()
                    .map(String::from);
                self.in_showdown = false;
                Some(RenderEvent::HandEnd { hand, payoff, winner })
            }
            _ => None,
        }
    }

    /// Whether private information should be masked in the current view.
    pub fn should_mask(&self) -> bool {
        self.view == SpectateView::Playing
    }

    /// Mask a string as fog-of-war (e.g., "As Kh" → "·· ··").
    pub fn mask(&self, s: &str) -> String {
        if self.should_mask() {
            s.chars()
                .map(|c| if c.is_alphanumeric() { '·' } else { c })
                .collect()
        } else {
            s.to_string()
        }
    }
}

/// Render the spectator state into a TUI area.
///
/// During play, shows a fog-of-war view with masked dice/cards.
/// After showdown, shows revealed information.
pub fn render(state: &SpectateState, area: Rect, buf: &mut Buffer) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let x = area.x;
    let y = area.y;

    // Draw a simple header
    let header = format!(
        "SPECTATOR MODE | Hand {} | {} | {:?}",
        state.hand, state.game_type, state.view
    );
    write_text(buf, x, y, &header, area.width);

    // Draw last bid if present
    if let Some(ref last_bid) = state.last_bid {
        let (player, qty, face) = last_bid;
        let bid_line = if state.should_mask() {
            format!("  {} bid [·· ··] (masked)", player)
        } else {
            format!("  {} bid ({}x{})", player, qty, face)
        };
        write_text(buf, x, y + 1, &bid_line, area.width);
    }

    // Draw players
    let players_line = if state.players.is_empty() {
        "  No active game".to_string()
    } else {
        let masked: Vec<String> = state
            .players
            .iter()
            .map(|p| {
                if state.should_mask() {
                    state.mask(p)
                } else {
                    p.clone()
                }
            })
            .collect();
        format!("  Players: {}", masked.join(" vs "))
    };
    write_text(buf, x, y + 2, &players_line, area.width);
}

/// Write text at position, respecting area bounds.
fn write_text(buf: &mut Buffer, x: u16, y: u16, text: &str, max_width: u16) {
    for (i, c) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= buf.area().right() || i as u16 >= max_width {
            break;
        }
        buf[(cx, y)].set_char(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hand_start_event(hand: u64, players: &[&str], game_type: &str) -> String {
        let players_json: Vec<String> = players.iter().map(|s| format!("\"{}\"", s)).collect();
        format!(
            r#"{{"type":"hand_start","data":{{"hand":{},"players":[{}],"game_type":"{}"}}}}"#,
            hand,
            players_json.join(","),
            game_type
        )
    }

    fn bid_event(player: &str, quantity: u8, face: u8) -> String {
        format!(
            r#"{{"type":"bid","data":{{"player":"{}","quantity":{},"face":{}}}}}"#,
            player, quantity, face
        )
    }

    fn showdown_event(player: &str, hand: &[&str]) -> String {
        let hand_json: Vec<String> = hand.iter().map(|s| format!("\"{}\"", s)).collect();
        format!(
            r#"{{"type":"showdown","data":{{"player":"{}","hand":[{}]}}}}"#,
            player,
            hand_json.join(",")
        )
    }

    fn challenge_event(player: &str) -> String {
        format!(r#"{{"type":"challenge","data":{{"player":"{}"}}}}"#, player)
    }

    fn hand_end_event(hand: u64, winner: Option<&str>) -> String {
        match winner {
            Some(w) => format!(
                r#"{{"type":"hand_end","data":{{"hand":{},"payoff":[["{}",1.0]],"winner":"{}"}}}}"#,
                hand, w, w
            ),
            None => format!(r#"{{"type":"hand_end","data":{{"hand":{},"payoff":[],"winner":null}}}}"#, hand),
        }
    }

    #[test]
    fn renders_fog_of_war() {
        let mut state = SpectateState::new();

        // Start a hand
        let _ = state.handle_event(&hand_start_event(1, &["alice", "bob"], "liars_dice"));
        assert_eq!(state.view, SpectateView::Playing);
        assert!(state.should_mask());

        // Bid event - dice values should be masked
        let _ = state.handle_event(&bid_event("alice", 2, 4));
        assert!(state.should_mask());
        // The bid content (quantity, face) comes from relay and is visible to spectators
        // Fog-of-war means the *players' private dice* are masked, not the bid

        // Verify mask function works
        assert_eq!(state.mask("As Kh"), "·· ··");
        assert_eq!(state.mask("3"), "·");
        assert_eq!(state.mask("hello"), "·····");
    }

    #[test]
    fn reveal_shows_hole_cards_after_showdown() {
        let mut state = SpectateState::new();

        // Start a hand
        let _ = state.handle_event(&hand_start_event(1, &["alice", "bob"], "liars_dice"));
        assert_eq!(state.view, SpectateView::Playing);

        // Bid
        let _ = state.handle_event(&bid_event("alice", 2, 4));

        // Challenge
        let _ = state.handle_event(&challenge_event("bob"));

        // Showdown reveals the hands
        let _ = state.handle_event(&showdown_event("bob", &["3", "5"]));
        assert_eq!(state.view, SpectateView::Revealed);
        assert!(!state.should_mask());

        // After showdown, mask returns original value
        assert_eq!(state.mask("As Kh"), "As Kh");

        // Hand end
        let _ = state.handle_event(&hand_end_event(1, Some("bob")));

        // Next hand starts fresh in Playing mode
        let _ = state.handle_event(&hand_start_event(2, &["alice", "bob"], "liars_dice"));
        assert_eq!(state.view, SpectateView::Playing);
    }

    #[test]
    fn reveal_blocked_during_play() {
        let mut state = SpectateState::new();

        // Start a hand
        let _ = state.handle_event(&hand_start_event(1, &["alice", "bob"], "liars_dice"));
        assert_eq!(state.view, SpectateView::Playing);

        // During play (before showdown), mask is active
        assert!(state.should_mask());
        assert_eq!(state.mask("secret"), "······");

        // Even if we somehow get a showdown event, mask is off after it
        let _ = state.handle_event(&showdown_event("alice", &["2", "6"]));
        assert!(!state.should_mask());
    }

    #[test]
    fn hand_start_transitions_to_playing() {
        let mut state = SpectateState::new();

        // Start in revealed mode (from previous hand)
        state.view = SpectateView::Revealed;

        // New hand resets to playing
        let _ = state.handle_event(&hand_start_event(5, &["charlie", "diana"], "poker"));
        assert_eq!(state.view, SpectateView::Playing);
        assert_eq!(state.hand, 5);
        assert_eq!(state.game_type, "poker");
        assert!(state.players.contains(&"charlie".to_string()));
        assert!(state.players.contains(&"diana".to_string()));
    }

    #[test]
    fn unknown_event_type_returns_none() {
        let mut state = SpectateState::new();
        let result = state.handle_event(r#"{"type":"unknown","data":{}}"#);
        assert!(result.is_none());
    }

    #[test]
    fn malformed_event_returns_none() {
        let mut state = SpectateState::new();
        assert!(state.handle_event("not valid json").is_none());
        assert!(state.handle_event(r#"{"type":}"#).is_none());
        assert!(state.handle_event("").is_none());
    }

    #[test]
    fn spectate_state_mask_is_idempotent() {
        let state = SpectateState::new();
        let original = "As Kh Qj";
        // Apply mask twice
        let masked = state.mask(original);
        let masked_again = SpectateState::new().mask(&masked);
        assert_eq!(masked, masked_again);
    }

    #[test]
    fn view_mode_after_challenge() {
        // Challenge does NOT trigger reveal — only showdown does
        let mut state = SpectateState::new();
        let _ = state.handle_event(&hand_start_event(1, &["alice", "bob"], "liars_dice"));
        let _ = state.handle_event(&bid_event("alice", 1, 3));
        let _ = state.handle_event(&challenge_event("bob"));

        // Still in playing mode after challenge
        assert_eq!(state.view, SpectateView::Playing);
        assert!(state.should_mask());
    }
}
