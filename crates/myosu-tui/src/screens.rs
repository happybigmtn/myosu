//! Screen state machine for TUI navigation.
//!
//! Manages transitions between application screens:
//! - Onboarding: first-run setup
//! - Lobby: game/subnet selection
//! - Game: active gameplay
//! - Stats: session summary
//! - Coaching: /analyze output
//! - History: /history output
//! - Wallet: account + staking
//! - Spectate: watch agent vs agent

use std::path::Path;

/// Application screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// First-run setup (DESIGN.md 8.0a-8.0c).
    Onboarding,
    /// Game/subnet selection (DESIGN.md 9.23).
    Lobby,
    /// Active gameplay (DESIGN.md 9.1-9.20, all 20 games).
    Game,
    /// Session summary (DESIGN.md 10.4).
    Stats,
    /// /analyze output (DESIGN.md 9.22).
    Coaching,
    /// /history output.
    History,
    /// Account + staking (DESIGN.md 8.0d).
    Wallet,
    /// Watch agent vs agent (DESIGN.md 9.24).
    Spectate,
}

impl Screen {
    /// Returns true if this screen is an overlay that returns to Game on any key.
    pub fn is_game_overlay(self) -> bool {
        matches!(self, Screen::Coaching | Screen::History)
    }

    /// Returns true if this screen supports /back navigation.
    pub fn supports_back(self) -> bool {
        self == Screen::Wallet
    }

    /// Default declaration text for each screen (ALLCAPS).
    pub fn default_declaration(self) -> &'static str {
        match self {
            Screen::Onboarding => "WELCOME TO MYOSU",
            Screen::Lobby => "SELECT A GAME",
            Screen::Game => "THE SYSTEM AWAITS YOUR DECISION",
            Screen::Stats => "SESSION SUMMARY",
            Screen::Coaching => "ANALYSIS",
            Screen::History => "HAND HISTORY",
            Screen::Wallet => "ACCOUNT",
            Screen::Spectate => "SPECTATOR MODE",
        }
    }
}

/// Manages screen state and navigation history.
#[derive(Debug)]
pub struct ScreenManager {
    current: Screen,
    history: Vec<Screen>,
    key_file_path: String,
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenManager {
    /// Create a new screen manager with default initial screen.
    ///
    /// Starts at Onboarding if no key file exists, otherwise goes straight to Lobby.
    pub fn new() -> Self {
        let key_path = Self::default_key_path();

        let initial = if Path::new(&key_path).exists() {
            Screen::Lobby
        } else {
            Screen::Onboarding
        };

        Self {
            current: initial,
            history: Vec::new(),
            key_file_path: key_path,
        }
    }

    fn default_key_path() -> String {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(|h| format!("{}/.myosu/key", h))
            .unwrap_or_else(|_| "~/.myosu/key".to_string())
    }

    /// Create a screen manager with a specific starting screen (for testing).
    #[must_use]
    pub fn with_screen(screen: Screen) -> Self {
        Self {
            current: screen,
            history: Vec::new(),
            key_file_path: "~/.myosu/key".to_string(),
        }
    }

    /// Get the current screen.
    pub fn current(&self) -> Screen {
        self.current
    }

    /// Get the navigation history.
    pub fn history(&self) -> &[Screen] {
        &self.history
    }

    /// Check if a key file exists at the configured path.
    pub fn has_key_file(&self) -> bool {
        Path::new(&self.key_file_path).exists()
    }

    /// Transition to a new screen.
    ///
    /// Pushes current screen to history before transitioning (for screens
    /// that support back navigation).
    pub fn transition(&mut self, to: Screen) {
        // Save current screen to history if the new screen supports back
        // or if we're going to an overlay
        if to.supports_back() || self.current.is_game_overlay() {
            self.history.push(self.current);
        }
        self.current = to;
    }

    /// Navigate back to the previous screen.
    ///
    /// Returns true if navigation was successful, false if no history.
    pub fn back(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.current = previous;
            true
        } else {
            false
        }
    }

    /// Handle any key press on overlay screens (Coaching, History).
    ///
    /// Returns true if the screen was changed (overlay closed).
    pub fn handle_overlay_key(&mut self) -> bool {
        if self.current.is_game_overlay() {
            self.back()
        } else {
            false
        }
    }

    /// Process a slash command and potentially change screens.
    ///
    /// Returns the target screen if a transition should occur.
    pub fn handle_command(&mut self, cmd: &str) -> Option<Screen> {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/quit" => match self.current {
                Screen::Game | Screen::Stats => Some(Screen::Lobby),
                Screen::Spectate => Some(Screen::Lobby),
                _ => None,
            },
            "/stats" => match self.current {
                Screen::Game => Some(Screen::Stats),
                _ => None,
            },
            "/analyze" => match self.current {
                Screen::Game => Some(Screen::Coaching),
                _ => None,
            },
            "/history" => match self.current {
                Screen::Game => Some(Screen::History),
                _ => None,
            },
            "/wallet" => match self.current {
                Screen::Lobby | Screen::Game => Some(Screen::Wallet),
                _ => None,
            },
            "/back" => {
                if self.current.supports_back() {
                    self.back();
                }
                None
            }
            "/spectate" => match self.current {
                Screen::Lobby => Some(Screen::Spectate),
                _ => None,
            },
            _ => {
                // Check for numeric input in lobby (subnet selection)
                if self.current == Screen::Lobby {
                    if let Ok(_subnet_id) = parts[0].parse::<u32>() {
                        return Some(Screen::Game);
                    }
                }
                None
            }
        }
    }

    /// Apply a command, transitioning screens if needed.
    ///
    /// Returns true if the screen was changed.
    pub fn apply_command(&mut self, cmd: &str) -> bool {
        if let Some(target) = self.handle_command(cmd) {
            self.transition(target);
            true
        } else {
            false
        }
    }

    /// Complete onboarding and transition to lobby.
    ///
    /// Should be called when onboarding setup is complete.
    pub fn complete_onboarding(&mut self) {
        if self.current == Screen::Onboarding {
            self.current = Screen::Lobby;
        }
    }

    /// Start a new game session from stats screen.
    pub fn new_game_from_stats(&mut self) {
        if self.current == Screen::Stats {
            self.current = Screen::Game;
            self.history.clear();
        }
    }

    /// Quit spectate mode and return to lobby.
    pub fn quit_spectate(&mut self) {
        if self.current == Screen::Spectate {
            self.current = Screen::Lobby;
            self.history.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lobby_to_game() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);
        assert_eq!(mgr.current(), Screen::Lobby);

        // Typing "1" in lobby transitions to Game
        assert!(mgr.apply_command("1"));
        assert_eq!(mgr.current(), Screen::Game);
    }

    #[test]
    fn game_to_stats() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // /stats during game shows Stats screen
        assert!(mgr.apply_command("/stats"));
        assert_eq!(mgr.current(), Screen::Stats);
    }

    #[test]
    fn slash_analyze_to_coaching() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // /analyze from game transitions to Coaching
        assert!(mgr.apply_command("/analyze"));
        assert_eq!(mgr.current(), Screen::Coaching);
    }

    #[test]
    fn slash_history_to_history() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // /history from game transitions to History
        assert!(mgr.apply_command("/history"));
        assert_eq!(mgr.current(), Screen::History);
    }

    #[test]
    fn onboarding_to_lobby() {
        let mut mgr = ScreenManager::with_screen(Screen::Onboarding);
        assert_eq!(mgr.current(), Screen::Onboarding);

        // Onboarding completion transitions to Lobby
        mgr.complete_onboarding();
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn wallet_back_navigation() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // /wallet from Lobby transitions to Wallet
        assert!(mgr.apply_command("/wallet"));
        assert_eq!(mgr.current(), Screen::Wallet);
        // History should contain Lobby for back navigation
        assert!(mgr.history.contains(&Screen::Lobby));

        // /back from Wallet returns to previous screen
        assert!(mgr.back());
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn spectate_from_lobby() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // /spectate from Lobby transitions to Spectate
        assert!(mgr.apply_command("/spectate"));
        assert_eq!(mgr.current(), Screen::Spectate);
    }

    #[test]
    fn quit_spectate_returns_to_lobby() {
        let mut mgr = ScreenManager::with_screen(Screen::Spectate);

        // Quit spectate returns to lobby
        mgr.quit_spectate();
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn quit_from_stats_returns_to_lobby() {
        let mut mgr = ScreenManager::with_screen(Screen::Stats);

        // /quit from Stats returns to Lobby
        assert!(mgr.apply_command("/quit"));
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn quit_from_game_returns_to_lobby() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // /quit from Game returns to Lobby
        assert!(mgr.apply_command("/quit"));
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn overlay_any_key_returns_to_game() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // Enter Coaching overlay
        assert!(mgr.apply_command("/analyze"));
        assert_eq!(mgr.current(), Screen::Coaching);

        // Any key from Coaching returns to Game
        assert!(mgr.handle_overlay_key());
        assert_eq!(mgr.current(), Screen::Game);

        // Enter History overlay
        assert!(mgr.apply_command("/history"));
        assert_eq!(mgr.current(), Screen::History);

        // Any key from History returns to Game
        assert!(mgr.handle_overlay_key());
        assert_eq!(mgr.current(), Screen::Game);
    }

    #[test]
    fn wallet_from_game() {
        let mut mgr = ScreenManager::with_screen(Screen::Game);

        // /wallet from Game transitions to Wallet
        assert!(mgr.apply_command("/wallet"));
        assert_eq!(mgr.current(), Screen::Wallet);
        // History should contain Game for back navigation
        assert!(mgr.history.contains(&Screen::Game));
    }

    #[test]
    fn new_game_from_stats() {
        let mut mgr = ScreenManager::with_screen(Screen::Stats);

        // Start new game from Stats
        mgr.new_game_from_stats();
        assert_eq!(mgr.current(), Screen::Game);
    }

    #[test]
    fn screen_variants() {
        // Test all screen variants exist and are distinct
        let screens = [
            Screen::Onboarding,
            Screen::Lobby,
            Screen::Game,
            Screen::Stats,
            Screen::Coaching,
            Screen::History,
            Screen::Wallet,
            Screen::Spectate,
        ];

        for (i, s1) in screens.iter().enumerate() {
            for (j, s2) in screens.iter().enumerate() {
                if i != j {
                    assert_ne!(s1, s2, "Screens at index {} and {} should differ", i, j);
                }
            }
        }
    }

    #[test]
    fn screen_is_game_overlay() {
        assert!(!Screen::Onboarding.is_game_overlay());
        assert!(!Screen::Lobby.is_game_overlay());
        assert!(!Screen::Game.is_game_overlay());
        assert!(!Screen::Stats.is_game_overlay());
        assert!(Screen::Coaching.is_game_overlay());
        assert!(Screen::History.is_game_overlay());
        assert!(!Screen::Wallet.is_game_overlay());
        assert!(!Screen::Spectate.is_game_overlay());
    }

    #[test]
    fn screen_supports_back() {
        assert!(!Screen::Onboarding.supports_back());
        assert!(!Screen::Lobby.supports_back());
        assert!(!Screen::Game.supports_back());
        assert!(!Screen::Stats.supports_back());
        assert!(!Screen::Coaching.supports_back());
        assert!(!Screen::History.supports_back());
        assert!(Screen::Wallet.supports_back());
        assert!(!Screen::Spectate.supports_back());
    }

    #[test]
    fn default_declarations() {
        assert_eq!(Screen::Onboarding.default_declaration(), "WELCOME TO MYOSU");
        assert_eq!(Screen::Lobby.default_declaration(), "SELECT A GAME");
        assert_eq!(
            Screen::Game.default_declaration(),
            "THE SYSTEM AWAITS YOUR DECISION"
        );
        assert_eq!(Screen::Stats.default_declaration(), "SESSION SUMMARY");
        assert_eq!(Screen::Coaching.default_declaration(), "ANALYSIS");
        assert_eq!(Screen::History.default_declaration(), "HAND HISTORY");
        assert_eq!(Screen::Wallet.default_declaration(), "ACCOUNT");
        assert_eq!(Screen::Spectate.default_declaration(), "SPECTATOR MODE");
    }

    #[test]
    fn handle_command_invalid_commands() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // Invalid commands return None
        assert_eq!(mgr.handle_command("/invalid"), None);
        assert_eq!(mgr.handle_command(""), None);
        assert_eq!(mgr.handle_command("   "), None);

        // /stats from lobby is invalid
        assert_eq!(mgr.handle_command("/stats"), None);

        // /quit from lobby is invalid
        assert_eq!(mgr.handle_command("/quit"), None);
    }

    #[test]
    fn history_tracking() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // Transition to Wallet should push Lobby to history
        mgr.transition(Screen::Wallet);
        assert_eq!(mgr.history(), &[Screen::Lobby]);

        // Transition to Game (from Wallet) - Wallet is overlay-supporting
        mgr.transition(Screen::Game);
        // History should have both
        assert!(mgr.history().contains(&Screen::Lobby));
        assert!(mgr.history().contains(&Screen::Wallet));
    }

    #[test]
    fn back_with_empty_history() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // Back with empty history returns false
        assert!(!mgr.back());
        assert_eq!(mgr.current(), Screen::Lobby);
    }

    #[test]
    fn multiple_subnet_selections() {
        let mut mgr = ScreenManager::with_screen(Screen::Lobby);

        // Can select different subnet IDs
        assert!(mgr.apply_command("1"));
        assert_eq!(mgr.current(), Screen::Game);

        // Go back and select another
        mgr.back();
        assert_eq!(mgr.current(), Screen::Lobby);

        assert!(mgr.apply_command("5"));
        assert_eq!(mgr.current(), Screen::Game);
    }
}
