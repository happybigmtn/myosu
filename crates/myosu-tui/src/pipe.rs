//! Pipe mode for agent protocol.
//!
//! When `--pipe` flag is set:
//! - Disable ratatui alternate screen (no TUI rendering)
//! - On each game state change, print `GameRenderer::pipe_output()` to stdout
//! - Read lines from stdin as input commands
//! - No color codes, no box-drawing, no cursor manipulation
//! - Flush stdout after every write
//!
//! This enables: `agent_a | myosu-play --pipe | agent_b`

use crate::renderer::GameRenderer;
use std::io::{self, BufRead, Write};

/// Pipe mode driver.
///
/// Reads lines from stdin, parses them via the game renderer,
/// and outputs structured state via `pipe_output()` to stdout.
pub struct PipeMode<'a> {
    renderer: &'a dyn GameRenderer,
    output: io::Stdout,
}

impl<'a> PipeMode<'a> {
    /// Create a new pipe mode instance with the given game renderer.
    pub fn new(renderer: &'a dyn GameRenderer) -> Self {
        Self {
            renderer,
            output: io::stdout(),
        }
    }

    /// Output current game state to stdout.
    ///
    /// This method writes a metadata line plus `GameRenderer::pipe_output()`
    /// to stdout and flushes immediately to ensure the agent receives timely
    /// updates.
    pub fn output_state(&mut self) -> io::Result<()> {
        for line in self.frame_lines() {
            writeln!(self.output, "{line}")?;
        }
        self.output.flush()?;
        Ok(())
    }

    /// Render the current shell/game frame as plain-text protocol lines.
    pub fn frame_lines(&self) -> Vec<String> {
        vec![self.meta_line(), self.renderer.pipe_output()]
    }

    /// Render shell-level metadata for the current frame.
    pub fn meta_line(&self) -> String {
        format!(
            "META game={:?} context={:?} declaration={:?}",
            self.renderer.game_label(),
            self.renderer.context_label(),
            self.renderer.declaration()
        )
    }

    /// Read a line from stdin and return the parsed action text.
    ///
    /// Returns `None` only if stdin is closed. Blank lines are returned as an
    /// empty string so the caller can decide whether to ignore them.
    pub fn read_input(&self) -> Option<String> {
        let stdin = io::stdin();
        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line).ok()?;
        if bytes_read == 0 {
            None
        } else {
            Some(line.trim().to_string())
        }
    }

    /// Check if the given input contains ANSI escape codes.
    ///
    /// Pipe mode output must never contain ANSI codes.
    pub fn has_ansi_codes(s: &str) -> bool {
        s.contains("\x1b[")
    }

    /// Run the pipe mode loop.
    ///
    /// This is a convenience method that outputs initial state,
    /// then processes stdin lines until EOF or error.
    /// The caller is responsible for updating game state and
    /// calling this method again after each state change.
    pub fn run_once(&mut self) -> io::Result<Option<String>> {
        self.output_state()?;
        Ok(self.read_input())
    }
}

/// Check if a string contains any ANSI escape sequences.
///
/// Pipe mode output must be plain text without any formatting.
pub fn is_plain_text(s: &str) -> bool {
    !PipeMode::has_ansi_codes(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    struct MockRenderer {
        hand_active: bool,
        state_text: &'static str,
    }

    impl GameRenderer for MockRenderer {
        fn render_state(&self, _area: Rect, _buf: &mut Buffer) {}

        fn desired_height(&self, _width: u16) -> u16 {
            if self.hand_active { 4 } else { 0 }
        }

        fn declaration(&self) -> &str {
            if self.hand_active {
                "THE SYSTEM AWAITS YOUR DECISION"
            } else {
                "NO ACTIVE HAND"
            }
        }

        fn completions(&self) -> Vec<String> {
            vec!["fold".into(), "call".into(), "raise".into()]
        }

        fn parse_input(&self, input: &str) -> Option<String> {
            match input.trim().to_lowercase().as_str() {
                "f" | "fold" => Some("fold".into()),
                "c" | "call" => Some("call".into()),
                "r" | "raise" => Some("raise".into()),
                _ => None,
            }
        }

        fn clarify(&self, _input: &str) -> Option<String> {
            None
        }

        fn pipe_output(&self) -> String {
            self.state_text.to_string()
        }

        fn game_label(&self) -> &str {
            "TEST"
        }

        fn context_label(&self) -> String {
            "TEST HAND".to_string()
        }
    }

    #[test]
    fn pipe_output_no_ansi() {
        let plain = "STATE hand=47 pot=12 hero=AcKh";
        assert!(!PipeMode::has_ansi_codes(plain));
        assert!(is_plain_text(plain));

        let with_ansi = "\x1b[32mSTATE\x1b[0m hand=47";
        assert!(PipeMode::has_ansi_codes(with_ansi));
        assert!(!is_plain_text(with_ansi));
    }

    #[test]
    fn pipe_output_matches_design_md() {
        let renderer = MockRenderer {
            hand_active: true,
            state_text: "STATE flop Ts7h2c pot=12bb hero=AcKh stack=88bb to_call=4bb actions=fold,call,raise",
        };

        let output = renderer.pipe_output();
        assert!(output.starts_with("STATE "));
        assert!(output.contains("pot="));
        assert!(output.contains("hero="));
        assert!(output.contains("actions="));
    }

    #[test]
    fn frame_lines_include_meta_and_state() {
        let renderer = MockRenderer {
            hand_active: true,
            state_text: "STATE hand=47 pot=12 hero=AcKh board=Ts7h2c",
        };
        let pipe = PipeMode::new(&renderer);

        assert_eq!(
            pipe.frame_lines(),
            vec![
                "META game=\"TEST\" context=\"TEST HAND\" declaration=\"THE SYSTEM AWAITS YOUR DECISION\""
                    .to_string(),
                "STATE hand=47 pot=12 hero=AcKh board=Ts7h2c".to_string(),
            ]
        );
    }

    #[test]
    fn pipe_output_idle_state() {
        let renderer = MockRenderer {
            hand_active: false,
            state_text: "STATE idle",
        };

        let output = renderer.pipe_output();
        assert_eq!(output, "STATE idle");
    }

    #[test]
    fn mock_renderer_parse_input() {
        let renderer = MockRenderer {
            hand_active: true,
            state_text: "test",
        };

        assert_eq!(renderer.parse_input("f"), Some("fold".to_string()));
        assert_eq!(renderer.parse_input("c"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("r"), Some("raise".to_string()));

        assert_eq!(renderer.parse_input("fold"), Some("fold".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("raise"), Some("raise".to_string()));

        assert_eq!(renderer.parse_input("invalid"), None);
    }

    #[test]
    fn is_plain_text_detects_ansi() {
        assert!(!is_plain_text("\x1b[31mred\x1b[0m"));
        assert!(!is_plain_text("\x1b[1mbold\x1b[0m"));
        assert!(!is_plain_text("\x1b[32;1mgreen bold\x1b[0m"));

        assert!(is_plain_text("plain text"));
        assert!(is_plain_text("STATE hand=47"));
        assert!(is_plain_text("fold call raise"));
    }
}
