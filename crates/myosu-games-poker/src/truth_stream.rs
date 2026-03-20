//! Truth stream emitter for NLHE poker action log.
//!
//! Processes game events into prose log lines with right-aligned pot running total.
//! Follows design.md's visual grammar: prose verbs, two-space indent,
//! `───` street separators, and no icons.

use ratatui::style::{Color, Style};

/// Log line type determines styling.
#[derive(Debug, Clone, PartialEq)]
pub enum LogLineType {
    /// Action by a player (e.g., "solver raises to 6bb")
    Action,
    /// Fold (rendered in dim style)
    Fold,
    /// Street transition (e.g., "─── flop: T♠ 7♥ 2♣")
    StreetTransition,
    /// Showdown reveal
    Showdown,
    /// Hand result (win/loss)
    Result,
    /// Error message
    Error,
    /// Fallback notice
    Fallback,
    /// Blank separator line
    Blank,
}

/// A single log line with metadata for rendering.
#[derive(Debug, Clone)]
pub struct LogLine {
    /// The rendered text of this line.
    pub text: String,
    /// The type of line (determines style).
    pub line_type: LogLineType,
    /// Pot size at the time this line was emitted.
    pub pot_at_line: u32,
}

impl LogLine {
    /// Create a new action log line.
    pub fn action(actor: &str, verb: &str, amount: Option<u32>, pot: u32) -> Self {
        let text = if let Some(amt) = amount {
            format!("{} {} to {}bb", actor, verb, amt)
        } else {
            format!("{} {}", actor, verb)
        };
        Self {
            text,
            line_type: LogLineType::Action,
            pot_at_line: pot,
        }
    }

    /// Create a fold log line.
    pub fn fold(actor: &str, pot: u32) -> Self {
        Self {
            text: format!("{} folds", actor),
            line_type: LogLineType::Fold,
            pot_at_line: pot,
        }
    }

    /// Create a check log line.
    pub fn check(actor: &str, pot: u32) -> Self {
        Self {
            text: format!("{} checks", actor),
            line_type: LogLineType::Action,
            pot_at_line: pot,
        }
    }

    /// Create a street transition log line.
    pub fn street(street: &str, cards: &[String], pot: u32) -> Self {
        let card_str = cards
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        Self {
            text: format!("─── {}: {}", street, card_str),
            line_type: LogLineType::StreetTransition,
            pot_at_line: pot,
        }
    }

    /// Create a showdown line.
    pub fn showdown(actor: &str, hole: &str, hand_desc: &str, pot: u32) -> Self {
        Self {
            text: format!("{} shows {}  {}", actor, hole, hand_desc),
            line_type: LogLineType::Showdown,
            pot_at_line: pot,
        }
    }

    /// Create a result line.
    pub fn result(actor: &str, amount_str: &str, pot: u32) -> Self {
        Self {
            text: format!("{} {}", actor, amount_str),
            line_type: LogLineType::Result,
            pot_at_line: pot,
        }
    }

    /// Create a blank separator line.
    pub fn blank() -> Self {
        Self {
            text: String::new(),
            line_type: LogLineType::Blank,
            pot_at_line: 0,
        }
    }

    /// Get the style for this log line.
    pub fn style(&self) -> Style {
        match self.line_type {
            LogLineType::Fold | LogLineType::Fallback => {
                Style::default().fg(Color::Rgb(96, 96, 96))
            }
            LogLineType::Result => {
                // Win = green, but we don't know direction here
                Style::default().fg(Color::Rgb(192, 192, 192))
            }
            LogLineType::StreetTransition => Style::default().fg(Color::Rgb(96, 96, 96)),
            LogLineType::Error => Style::default().fg(Color::Rgb(204, 51, 51)),
            _ => Style::default().fg(Color::Rgb(192, 192, 192)),
        }
    }

    /// Format the line with right-aligned pot for display.
    pub fn formatted_line(&self, terminal_width: u16) -> String {
        let pot_str = format!("pot {}", self.pot_at_line);
        let pot_len = pot_str.len();
        let text_len = self.text.len();
        let min_width = text_len + pot_len + 1; // 1 for space between

        if min_width as u16 >= terminal_width {
            // Line is too wide, just return the text
            return self.text.clone();
        }

        let spaces = (terminal_width as usize - text_len - pot_len - 1).max(2);
        format!("{}{}{}", self.text, " ".repeat(spaces), pot_str)
    }
}

/// Truth stream emitter — collects and formats NLHE action log entries.
#[derive(Debug, Default)]
pub struct TruthStreamEmitter {
    /// Log lines in order.
    lines: Vec<LogLine>,
    /// Current pot size.
    current_pot: u32,
    /// Current street name.
    current_street: String,
}

impl TruthStreamEmitter {
    /// Create a new emitter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a pot update.
    pub fn update_pot(&mut self, pot: u32) {
        self.current_pot = pot;
    }

    /// Record a street change.
    pub fn set_street(&mut self, street: &str) {
        self.current_street = street.to_string();
    }

    /// Emit an action line.
    pub fn emit_action(&mut self, actor: &str, verb: &str, amount: Option<u32>) {
        let line = LogLine::action(actor, verb, amount, self.current_pot);
        self.lines.push(line);
    }

    /// Emit a fold line.
    pub fn emit_fold(&mut self, actor: &str) {
        let line = LogLine::fold(actor, self.current_pot);
        self.lines.push(line);
    }

    /// Emit a check line.
    pub fn emit_check(&mut self, actor: &str) {
        let line = LogLine::check(actor, self.current_pot);
        self.lines.push(line);
    }

    /// Emit a street transition.
    pub fn emit_street(&mut self, street: &str, cards: &[String]) {
        let line = LogLine::street(street, cards, self.current_pot);
        self.lines.push(line);
        self.current_street = street.to_string();
    }

    /// Emit a showdown line.
    pub fn emit_showdown(&mut self, actor: &str, hole: &str, hand_desc: &str) {
        let line = LogLine::showdown(actor, hole, hand_desc, self.current_pot);
        self.lines.push(line);
    }

    /// Emit a result line.
    pub fn emit_result(&mut self, actor: &str, amount_str: &str) {
        let line = LogLine::result(actor, amount_str, self.current_pot);
        self.lines.push(line);
    }

    /// Emit a blank line.
    pub fn emit_blank(&mut self) {
        self.lines.push(LogLine::blank());
    }

    /// Get all log lines.
    pub fn lines(&self) -> &[LogLine] {
        &self.lines
    }

    /// Clear all lines (for new hand).
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Reset state for new hand.
    pub fn reset(&mut self) {
        self.clear();
        self.current_pot = 0;
        self.current_street = "preflop".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_produces_log_line() {
        let mut emitter = TruthStreamEmitter::new();
        emitter.update_pot(9);
        emitter.emit_action("solver", "raises", Some(6));
        assert_eq!(emitter.lines.len(), 1);
        assert!(emitter.lines[0].text.contains("raises to 6bb"));
    }

    #[test]
    fn fold_produces_dim_line() {
        let mut emitter = TruthStreamEmitter::new();
        emitter.emit_fold("solver");
        assert_eq!(emitter.lines[0].line_type, LogLineType::Fold);
    }

    #[test]
    fn street_transition_format() {
        let line = LogLine::street(
            "flop",
            &["T♠".to_string(), "7♥".to_string(), "2♣".to_string()],
            12,
        );
        assert!(line.text.starts_with("─── flop:"));
        assert!(line.text.contains("T♠"));
    }

    #[test]
    fn showdown_shows_cards() {
        let mut emitter = TruthStreamEmitter::new();
        emitter.update_pot(100);
        emitter.emit_showdown("solver", "Q♣ J♣", "two pair");
        let line = &emitter.lines[0];
        assert!(line.text.contains("Q♣ J♣"));
        assert!(line.text.contains("two pair"));
    }

    #[test]
    fn emitter_collects_multiple_lines() {
        let mut emitter = TruthStreamEmitter::new();
        emitter.emit_action("solver", "raises", Some(6));
        emitter.emit_action("hero", "calls", None);
        emitter.emit_street(
            "flop",
            &["T♠".to_string(), "7♥".to_string(), "2♣".to_string()],
        );
        assert_eq!(emitter.lines.len(), 3);
    }

    #[test]
    fn reset_clears_state() {
        let mut emitter = TruthStreamEmitter::new();
        emitter.update_pot(50);
        emitter.emit_action("solver", "raises", Some(10));
        emitter.reset();
        assert!(emitter.lines.is_empty());
        assert_eq!(emitter.current_pot, 0);
    }

    #[test]
    fn blank_line() {
        let line = LogLine::blank();
        assert_eq!(line.text, "");
        assert_eq!(line.line_type, LogLineType::Blank);
    }

    #[test]
    fn formatted_line_with_pot() {
        let line = LogLine::action("hero", "bets", Some(8), 21);
        let formatted = line.formatted_line(60);
        assert!(formatted.contains("pot 21"));
    }
}
