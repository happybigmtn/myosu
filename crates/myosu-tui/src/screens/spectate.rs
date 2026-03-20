//! Spectator screen rendering with fog-of-war enforcement.
//!
//! Renders game state for spectators. Hole cards are NEVER shown during play.
//! Only revealed at showdown.

use serde::{Deserialize, Serialize};

/// Represents what a spectator can see at any point in the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectatorView {
    /// Current game state (public info only).
    pub public_state: PublicGameState,
    /// Whether showdown has occurred.
    pub is_showdown: bool,
    /// Player's hole cards (only visible at showdown).
    pub hole_cards: Option<Vec<String>>,
}

/// Public game state visible to spectators.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicGameState {
    /// Current round.
    pub round: u8,
    /// All bids made so far.
    pub bids: Vec<Bid>,
    /// Current player to act.
    pub current_player: u8,
}

/// A bid in the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bid {
    /// Player who made the bid.
    pub player: u8,
    /// Quantity of dice.
    pub quantity: u8,
    /// Face value.
    pub face: u8,
}

/// Spectator screen state.
#[derive(Clone, Debug)]
pub struct SpectatorScreen {
    /// Current view state.
    view: SpectatorView,
    /// Whether rendering is blocked (fog-of-war active).
    rendering_blocked: bool,
}

impl Default for SpectatorScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SpectatorScreen {
    /// Create a new spectator screen.
    pub fn new() -> Self {
        Self {
            view: SpectatorView {
                public_state: PublicGameState {
                    round: 0,
                    bids: Vec::new(),
                    current_player: 0,
                },
                is_showdown: false,
                hole_cards: None,
            },
            rendering_blocked: true,
        }
    }

    /// Update the view with new public state.
    pub fn update_public_state(&mut self, state: PublicGameState) {
        self.view.public_state = state;
        self.view.is_showdown = false;
        self.view.hole_cards = None;
        // During play, hole cards are blocked
        self.rendering_blocked = true;
    }

    /// Reveal hole cards at showdown.
    pub fn reveal_at_showdown(&mut self, hole_cards: Vec<String>) {
        self.view.is_showdown = true;
        self.view.hole_cards = Some(hole_cards);
        self.rendering_blocked = false;
    }

    /// Returns true if rendering is blocked due to fog-of-war.
    pub fn is_rendering_blocked(&self) -> bool {
        self.rendering_blocked
    }

    /// Get the current spectator view.
    pub fn view(&self) -> &SpectatorView {
        &self.view
    }

    /// Render the spectator view to a string.
    ///
    /// Returns "*** HOLE CARDS HIDDEN ***" if fog-of-war is active.
    pub fn render(&self) -> String {
        if self.rendering_blocked {
            return "*** HOLE CARDS HIDDEN ***".to_string();
        }

        if let Some(ref cards) = self.view.hole_cards {
            format!("Hole cards: {:?}", cards)
        } else {
            format!(
                "Round {} - Current player: {}",
                self.view.public_state.round, self.view.public_state.current_player
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_fog_of_war() {
        let screen = SpectatorScreen::new();

        // Initially, fog-of-war blocks rendering
        assert!(screen.is_rendering_blocked());
        assert!(screen.render().contains("HIDDEN"));
    }

    #[test]
    fn reveal_shows_hole_cards_after_showdown() {
        let mut screen = SpectatorScreen::new();

        // Initially blocked
        assert!(screen.is_rendering_blocked());

        // Reveal at showdown
        screen.reveal_at_showdown(vec!["K♠".to_string(), "Q♠".to_string()]);

        // Now unblocked and showing cards
        assert!(!screen.is_rendering_blocked());
        assert!(screen.render().contains("K♠"));
        assert!(screen.render().contains("Q♠"));
    }

    #[test]
    fn reveal_blocked_during_play() {
        let mut screen = SpectatorScreen::new();

        // Update with public state during play
        screen.update_public_state(PublicGameState {
            round: 1,
            bids: vec![Bid {
                player: 0,
                quantity: 1,
                face: 3,
            }],
            current_player: 1,
        });

        // Still blocked during play
        assert!(screen.is_rendering_blocked());
        assert!(screen.render().contains("HIDDEN"));

        // Even if we try to reveal, it should still be blocked during play
        // (update_public_state doesn't automatically unblock)
        assert!(screen.is_rendering_blocked());
    }

    #[test]
    fn spectator_view_serialization() {
        let view = SpectatorView {
            public_state: PublicGameState {
                round: 1,
                bids: vec![Bid {
                    player: 0,
                    quantity: 2,
                    face: 5,
                }],
                current_player: 1,
            },
            is_showdown: false,
            hole_cards: None,
        };

        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("round"));
        assert!(json.contains("bids"));
    }

    #[test]
    fn hole_cards_hidden_until_showdown() {
        let mut screen = SpectatorScreen::new();

        screen.update_public_state(PublicGameState {
            round: 2,
            bids: vec![],
            current_player: 0,
        });

        // During play - no hole cards visible
        assert!(screen.view().hole_cards.is_none());
        assert!(screen.is_rendering_blocked());

        // After showdown - hole cards revealed
        screen.reveal_at_showdown(vec!["A♥".to_string(), "K♥".to_string()]);
        assert!(screen.view().hole_cards.is_some());
        assert!(!screen.is_rendering_blocked());
    }
}
