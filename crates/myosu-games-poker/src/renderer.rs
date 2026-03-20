//! NLHE poker state renderer for the myosu TUI.
//!
//! Renders the game-specific state panel using design.md's field-label format.
//! The shell handles header, declaration, log, and input; this module only
//! draws the poker state panel.

use myosu_tui::GameRenderer;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

/// Column-aligned field-label rendering style.
fn label_style() -> Style {
    Style::default().fg(Color::Rgb(192, 192, 192))
}

fn bright_style() -> Style {
    Style::default().fg(Color::White)
}

fn dim_style() -> Style {
    Style::default().fg(Color::Rgb(96, 96, 96))
}

/// NlheRenderer implements GameRenderer for No-Limit Hold'em heads-up poker.
///
/// This is the reference GameRenderer implementation. It renders the poker
/// state panel using design.md's field-label format with column alignment.
///
/// The renderer operates in two modes:
/// - Stub mode: hardcoded pre-set game states (used in Slice 2)
/// - Live mode: connected to robopoker's Game engine (used in Slice 3+)
#[derive(Debug, Clone)]
pub struct NlheRenderer {
    /// Current game state for rendering
    state: NlheState,
    /// Derived context label (cached to avoid allocation on each call)
    context_label: String,
}

/// Hardcoded NLHE game states for the stub implementation.
#[derive(Debug, Clone, PartialEq)]
pub enum NlheState {
    /// Pre-flop state: no board cards yet
    Preflop {
        hero_hole: String,
        hero_stack: u32,
        hero_position: &'static str,
        opponent_stack: u32,
        opponent_position: &'static str,
        pot: u32,
        to_call: u32,
        hand_num: u32,
        /// Whether the hero has a pending decision (always true preflop in training)
        has_decision: bool,
    },
    /// Flop state: three board cards
    Flop {
        hero_hole: String,
        hero_stack: u32,
        hero_position: &'static str,
        opponent_stack: u32,
        opponent_position: &'static str,
        pot: u32,
        to_call: u32,
        board: [String; 3],
        hand_num: u32,
        has_decision: bool,
    },
    /// Bot is thinking
    BotThinking { hand_num: u32 },
    /// Hand complete / showdown
    Showdown {
        hand_num: u32,
        hero_hole: String,
        opponent_hole: String,
        board: [String; 5],
        pot: u32,
        result: String,
    },
    /// No active hand
    Idle,
}

impl Default for NlheState {
    fn default() -> Self {
        Self::Idle
    }
}

impl NlheRenderer {
    /// Create a new renderer with the given state.
    pub fn new(state: NlheState) -> Self {
        let context_label = Self::context_label_for_state(&state);
        Self {
            state,
            context_label,
        }
    }

    /// Create a preflop stub state for testing.
    pub fn preflop(hand_num: u32) -> Self {
        let state = NlheState::Preflop {
            hero_hole: "A♠ K♥".to_string(),
            hero_stack: 950,
            hero_position: "BB",
            opponent_stack: 1050,
            opponent_position: "SB",
            pot: 3,
            to_call: 1,
            hand_num,
            has_decision: true,
        };
        let context_label = Self::context_label_for_state(&state);
        Self {
            state,
            context_label,
        }
    }

    /// Create a flop stub state for testing.
    pub fn flop(hand_num: u32) -> Self {
        let state = NlheState::Flop {
            hero_hole: "A♠ K♥".to_string(),
            hero_stack: 950,
            hero_position: "BB",
            opponent_stack: 1050,
            opponent_position: "SB",
            pot: 12,
            to_call: 4,
            board: ["T♠".to_string(), "7♥".to_string(), "2♣".to_string()],
            hand_num,
            has_decision: true,
        };
        let context_label = Self::context_label_for_state(&state);
        Self {
            state,
            context_label,
        }
    }

    /// Update the renderer to a new state.
    pub fn set_state(&mut self, state: NlheState) {
        self.context_label = Self::context_label_for_state(&state);
        self.state = state;
    }

    fn game_label_static() -> &'static str {
        "NLHE-HU"
    }

    fn context_label_for_state(state: &NlheState) -> String {
        match state {
            NlheState::Idle => "".to_string(),
            NlheState::Preflop { hand_num, .. } => format!("HAND {}", hand_num),
            NlheState::Flop { hand_num, .. } => format!("HAND {} · FLOP", hand_num),
            NlheState::BotThinking { hand_num } => format!("HAND {} · FLOP", hand_num),
            NlheState::Showdown { hand_num, .. } => format!("HAND {}", hand_num),
        }
    }

    fn declaration_for_state(state: &NlheState) -> &'static str {
        match state {
            NlheState::Idle => "NO ACTIVE HAND",
            NlheState::Preflop { has_decision, .. } => {
                if *has_decision {
                    "THE SYSTEM AWAITS YOUR DECISION"
                } else {
                    "BOT THINKING"
                }
            }
            NlheState::Flop { has_decision, .. } => {
                if *has_decision {
                    "THE SYSTEM AWAITS YOUR DECISION"
                } else {
                    "BOT THINKING"
                }
            }
            NlheState::BotThinking { .. } => "BOT THINKING",
            NlheState::Showdown { .. } => "SHOWDOWN",
        }
    }

    fn completions_for_state(state: &NlheState) -> Vec<String> {
        match state {
            NlheState::Idle => vec!["new".to_string(), "quit".to_string()],
            NlheState::Preflop {
                has_decision: true, ..
            } => {
                vec![
                    "fold".to_string(),
                    "check".to_string(),
                    "call".to_string(),
                    "raise".to_string(),
                    "shove".to_string(),
                    "/deal".to_string(),
                    "/board".to_string(),
                    "/advisor".to_string(),
                ]
            }
            NlheState::Flop {
                has_decision: true, ..
            } => {
                vec![
                    "fold".to_string(),
                    "check".to_string(),
                    "call".to_string(),
                    "raise".to_string(),
                    "shove".to_string(),
                    "/deal".to_string(),
                    "/board".to_string(),
                    "/advisor".to_string(),
                ]
            }
            NlheState::Preflop {
                has_decision: false,
                ..
            }
            | NlheState::Flop {
                has_decision: false,
                ..
            }
            | NlheState::BotThinking { .. } => vec![],
            NlheState::Showdown { .. } => vec!["new".to_string(), "/quit".to_string()],
        }
    }

    /// Render the preflop state panel (4 lines).
    fn render_preflop_state(&self, area: Rect, buf: &mut Buffer) {
        let NlheState::Preflop {
            ref hero_hole,
            hero_stack,
            hero_position,
            opponent_stack,
            opponent_position,
            pot,
            to_call,
            ..
        } = self.state
        else {
            return;
        };

        let dim = dim_style();
        let bright = bright_style();
        let label = label_style();

        // Line 1: board (empty preflop slots)
        let y = area.y;
        write_label_value(buf, area.x, y, "board", "", &dim, &label);

        // Line 2: you (hero) + opponent
        let y = area.y + 1;
        // "you" row
        write_label_value(buf, area.x, y, "you", hero_hole.as_str(), &dim, &bright);
        // opponent row
        write_label_value(buf, area.x + 30, y, "solver", "·· ··", &dim, &dim);
        // stacks on the right
        let stack_text = format!("{}bb", hero_stack);
        let opp_stack_text = format!("{}bb", opponent_stack);
        let stack_x = area.right().saturating_sub(stack_text.len() as u16 + 1);
        let opp_stack_x = area.right().saturating_sub(opp_stack_text.len() as u16 + 1);
        write_at(buf, stack_x, y, &stack_text, &bright);
        write_at(buf, opp_stack_x, y, &opp_stack_text, &dim);
        write_at(
            buf,
            area.right().saturating_sub(3),
            y,
            hero_position,
            &label,
        );
        write_at(
            buf,
            area.right().saturating_sub(6),
            y,
            opponent_position,
            &label,
        );

        // Line 3: pot + to_call
        let y = area.y + 2;
        let pot_text = format!("{}bb", pot);
        let call_text = if to_call > 0 {
            format!("call {}bb", to_call)
        } else {
            "check".to_string()
        };
        write_label_value(buf, area.x, y, "pot", &pot_text, &dim, &bright);
        if to_call > 0 {
            write_at(buf, area.x + 15, y, &call_text, &label);
        }

        // Line 4: empty (for spacing / potential raise range)
    }

    /// Render the flop state panel.
    fn render_flop_state(&self, area: Rect, buf: &mut Buffer) {
        let NlheState::Flop {
            ref hero_hole,
            hero_stack,
            hero_position,
            opponent_stack,
            opponent_position,
            pot,
            to_call,
            ref board,
            has_decision,
            ..
        } = self.state
        else {
            return;
        };

        let dim = dim_style();
        let bright = bright_style();
        let label = label_style();

        // Line 1: board
        let y = area.y;
        let board_str = format!("{}  {}  {}", board[0], board[1], board[2]);
        write_label_value(buf, area.x, y, "board", &board_str, &dim, &bright);

        // Line 2: you + solver with stacks
        let y = area.y + 1;
        write_label_value(buf, area.x, y, "you", hero_hole.as_str(), &dim, &bright);
        write_label_value(buf, area.x + 30, y, "solver", "·· ··", &dim, &dim);
        let stack_text = format!("{}bb", hero_stack);
        let opp_stack_text = format!("{}bb", opponent_stack);
        write_at(
            buf,
            area.right().saturating_sub(stack_text.len() as u16 + 1),
            y,
            &stack_text,
            &bright,
        );
        write_at(
            buf,
            area.right().saturating_sub(opp_stack_text.len() as u16 + 1),
            y,
            &opp_stack_text,
            &dim,
        );
        write_at(
            buf,
            area.right().saturating_sub(3),
            y,
            hero_position,
            &label,
        );
        write_at(
            buf,
            area.right().saturating_sub(6),
            y,
            opponent_position,
            &label,
        );

        // Line 3: pot + decision context
        let y = area.y + 2;
        let pot_text = format!("{}bb", pot);
        write_label_value(buf, area.x, y, "pot", &pot_text, &dim, &bright);
        if has_decision && to_call > 0 {
            write_at(buf, area.x + 15, y, &format!("call {}bb", to_call), &label);
        }
    }

    /// Render the bot thinking state.
    fn render_bot_thinking_state(&self, _area: Rect, _buf: &mut Buffer) {
        // Bot thinking is indicated by the declaration; state panel stays the same
    }

    /// Render the showdown state.
    fn render_showdown_state(&self, area: Rect, buf: &mut Buffer) {
        let NlheState::Showdown {
            ref hero_hole,
            ref opponent_hole,
            ref board,
            pot,
            ref result,
            ..
        } = self.state
        else {
            return;
        };

        let dim = dim_style();
        let bright = bright_style();
        // Line 1: board
        let y = area.y;
        let board_str = format!("{}  {}  {}  ·  ·", board[0], board[1], board[2]);
        write_label_value(buf, area.x, y, "board", &board_str, &dim, &bright);

        // Line 2: showdown results
        let y = area.y + 1;
        write_label_value(buf, area.x, y, "you", hero_hole.as_str(), &dim, &bright);
        write_label_value(
            buf,
            area.x + 30,
            y,
            "solver",
            opponent_hole.as_str(),
            &dim,
            &dim,
        );
        let pot_text = format!("{}bb", pot);
        write_at(
            buf,
            area.right().saturating_sub(pot_text.len() as u16 + 1),
            y,
            &pot_text,
            &bright,
        );

        // Line 3: result
        let y = area.y + 2;
        write_label_value(buf, area.x, y, "result", result.as_str(), &dim, &bright);
    }
}

/// Write a label: value pair at the given position.
fn write_label_value(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    value: &str,
    label_style: &Style,
    value_style: &Style,
) {
    write_at(buf, x, y, label, label_style);
    write_at(buf, x + 8, y, value, value_style);
}

/// Write styled text at a specific buffer position.
fn write_at(buf: &mut Buffer, x: u16, y: u16, text: &str, style: &Style) {
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx < buf.area.right() {
            buf[(cx, y)].set_char(ch);
            buf[(cx, y)].set_style(*style);
        }
    }
}

impl GameRenderer for NlheRenderer {
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        match &self.state {
            NlheState::Idle => {}
            NlheState::Preflop { .. } => self.render_preflop_state(area, buf),
            NlheState::Flop { .. } => self.render_flop_state(area, buf),
            NlheState::BotThinking { .. } => self.render_bot_thinking_state(area, buf),
            NlheState::Showdown { .. } => self.render_showdown_state(area, buf),
        }
    }

    fn desired_height(&self, _width: u16) -> u16 {
        match self.state {
            NlheState::Idle => 0,
            NlheState::Preflop { .. } => 4,
            NlheState::Flop { .. } => 4,
            NlheState::BotThinking { .. } => 4,
            NlheState::Showdown { .. } => 4,
        }
    }

    fn declaration(&self) -> &str {
        Self::declaration_for_state(&self.state)
    }

    fn completions(&self) -> Vec<String> {
        Self::completions_for_state(&self.state)
    }

    fn parse_input(&self, input: &str) -> Option<String> {
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "f" | "fold" => Some("fold".to_string()),
            "c" | "call" | "x" | "check" => Some("check".to_string()),
            "r" | "raise" => Some("raise".to_string()),
            "s" | "shove" | "all-in" => Some("shove".to_string()),
            _ if input.starts_with("r ") => Some(input.clone()),
            _ if input.parse::<u32>().is_ok() => Some(format!("raise {}", input)),
            _ => None,
        }
    }

    fn clarify(&self, input: &str) -> Option<String> {
        let input = input.trim();
        if input.starts_with('r') && input != "raise" && !input.starts_with("r ") {
            Some("raise to how much? (e.g., raise 15)".to_string())
        } else {
            None
        }
    }

    fn pipe_output(&self) -> String {
        match &self.state {
            NlheState::Idle => "STATE idle".to_string(),
            NlheState::Preflop {
                hero_hole,
                pot,
                hand_num,
                ..
            } => {
                format!("STATE hand={} pot={} hero={}", hand_num, pot, hero_hole)
            }
            NlheState::Flop {
                hero_hole,
                pot,
                hand_num,
                board,
                has_decision,
                ..
            } => {
                let board_str = format!("{} {} {}", board[0], board[1], board[2]);
                format!(
                    "STATE hand={} pot={} hero={} board={} decision={}",
                    hand_num, pot, hero_hole, board_str, has_decision
                )
            }
            NlheState::BotThinking { hand_num } => {
                format!("STATE hand={} bot_thinking=true", hand_num)
            }
            NlheState::Showdown {
                hand_num,
                hero_hole,
                opponent_hole,
                board,
                pot,
                result,
            } => {
                let board_str = format!("{} {} {} · ·", board[0], board[1], board[2]);
                format!(
                    "STATE hand={} showdown=true hero={} opponent={} board={} pot={} result={}",
                    hand_num, hero_hole, opponent_hole, board_str, pot, result
                )
            }
        }
    }

    fn game_label(&self) -> &str {
        Self::game_label_static()
    }

    fn context_label(&self) -> &str {
        &self.context_label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn render_preflop_state() {
        let renderer = NlheRenderer::preflop(1);
        let area = Rect::new(0, 0, 80, 4);
        let mut buf = Buffer::empty(area);
        renderer.render_state(area, &mut buf);
        // Verify board row is rendered (empty slots)
        assert_eq!(buf[(0, 0)].symbol(), "b");
    }

    #[test]
    fn render_flop_with_board() {
        let renderer = NlheRenderer::flop(1);
        assert_eq!(renderer.desired_height(80), 4);
        let area = Rect::new(0, 0, 80, 4);
        let mut buf = Buffer::empty(area);
        renderer.render_state(area, &mut buf);
        // Board should contain card characters
        let content: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(content.contains('T') || content.contains('·'));
    }

    #[test]
    fn trait_is_object_safe() {
        let renderer = NlheRenderer::preflop(1);
        let _: Box<dyn GameRenderer> = Box::new(renderer);
    }

    #[test]
    fn parse_input_accepts_shorthands() {
        let renderer = NlheRenderer::preflop(1);
        assert_eq!(renderer.parse_input("f"), Some("fold".to_string()));
        assert_eq!(renderer.parse_input("c"), Some("check".to_string()));
        assert_eq!(renderer.parse_input("r 15"), Some("r 15".to_string()));
        assert_eq!(renderer.parse_input("50"), Some("raise 50".to_string()));
        assert_eq!(renderer.parse_input("xyz"), None);
    }

    #[test]
    fn pipe_output_returns_structured_text() {
        let renderer = NlheRenderer::preflop(47);
        let output = renderer.pipe_output();
        assert!(output.starts_with("STATE "));
        assert!(output.contains("hand=47"));
        assert!(output.contains("hero=A♠ K♥"));
    }

    #[test]
    fn desired_height_4_when_active() {
        let renderer = NlheRenderer::preflop(1);
        assert_eq!(renderer.desired_height(80), 4);
    }

    #[test]
    fn desired_height_0_when_idle() {
        let renderer = NlheRenderer::new(NlheState::Idle);
        assert_eq!(renderer.desired_height(80), 0);
    }

    #[test]
    fn declaration_for_preflop_decision() {
        let renderer = NlheRenderer::preflop(1);
        assert_eq!(renderer.declaration(), "THE SYSTEM AWAITS YOUR DECISION");
    }

    #[test]
    fn game_label_is_nlhe_hu() {
        let renderer = NlheRenderer::preflop(1);
        assert_eq!(renderer.game_label(), "NLHE-HU");
    }

    #[test]
    fn context_label_shows_hand_number() {
        let renderer = NlheRenderer::flop(47);
        assert!(renderer.context_label().contains("47"));
    }

    #[test]
    fn bot_thinking_declaration() {
        let renderer = NlheRenderer::new(NlheState::BotThinking { hand_num: 1 });
        assert_eq!(renderer.declaration(), "BOT THINKING");
    }

    #[test]
    fn showdown_declaration() {
        let renderer = NlheRenderer::new(NlheState::Showdown {
            hand_num: 1,
            hero_hole: "A♠ K♥".to_string(),
            opponent_hole: "Q♣ J♣".to_string(),
            board: [
                "T♠".to_string(),
                "7♥".to_string(),
                "2♣".to_string(),
                "·".to_string(),
                "·".to_string(),
            ],
            pot: 100,
            result: "you win 14bb".to_string(),
        });
        assert_eq!(renderer.declaration(), "SHOWDOWN");
    }

    #[test]
    fn completions_non_empty_when_decision() {
        let renderer = NlheRenderer::flop(1);
        let completions = renderer.completions();
        assert!(!completions.is_empty());
        assert!(completions.contains(&"fold".to_string()));
        assert!(completions.contains(&"raise".to_string()));
    }

    #[test]
    fn clarify_returns_prompt_for_ambiguous() {
        let renderer = NlheRenderer::preflop(1);
        assert!(renderer.clarify("r").is_some());
        assert!(renderer.clarify("raise").is_none());
    }
}
